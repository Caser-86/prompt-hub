use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use prompt_domain::{
    Actor, Prompt, PromptContent, PromptId, PromptSource, PromptVariable, SourceKind,
};
use prompt_mcp::{ToolOperation, approved_tools};
use prompt_store::{Database, PromptRepository, SearchQuery};
use serde_json::{Map, Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let response = handle_request(&line);
        if writeln!(stdout, "{response}").is_err() {
            break;
        }
    }
}

fn handle_request(line: &str) -> Value {
    let request: Value = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(_) => return error(Value::Null, -32700, "parse error"),
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        Some("tools/list") => {
            json!({"jsonrpc":"2.0", "id":id, "result":{"tools": approved_tools().into_iter().map(|tool| json!({
            "name": tool.name,
            "description": match tool.operation { ToolOperation::ReadOnly => "Prompt Hub local read-only tool", ToolOperation::InboxWrite => "Prompt Hub tool that creates an inbox draft" },
            "inputSchema":{"$ref":tool.input_schema_id}
        })).collect::<Vec<_>>()}})
        }
        Some("tools/call") => match call_tool(&request) {
            Ok(result) => {
                json!({"jsonrpc":"2.0", "id":id, "result":{"content":[{"type":"text", "text":result.to_string()}]}})
            }
            Err(message) => {
                json!({"jsonrpc":"2.0", "id":id, "result":{"isError":true, "content":[{"type":"text", "text":message}]}})
            }
        },
        Some(_) => error(id, -32601, "method not found"),
        None => error(id, -32600, "invalid request"),
    }
}

fn call_tool(request: &Value) -> Result<Value, String> {
    let params = request
        .get("params")
        .and_then(Value::as_object)
        .ok_or("invalid tool request")?;
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or("tool name is required")?;
    let arguments = params
        .get("arguments")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut repository = open_repository()?;
    match name {
        "search_prompts" => search(&repository, &arguments),
        "get_prompt" => get_prompt(&repository, &arguments),
        "render_prompt" => render_prompt(&repository, &arguments),
        "save_prompt_draft" => save_draft(&mut repository, &arguments),
        _ => Err("unknown tool".to_owned()),
    }
}

fn open_repository() -> Result<PromptRepository, String> {
    let path = env::var_os("PROMPT_HUB_DATABASE_PATH")
        .map(PathBuf::from)
        .ok_or("database_unavailable: PROMPT_HUB_DATABASE_PATH is not configured")?;
    Database::open(path)
        .map(Database::into_repository)
        .map_err(|_| "database_unavailable".to_owned())
}

fn search(repository: &PromptRepository, arguments: &Map<String, Value>) -> Result<Value, String> {
    let query = arguments.get("query").and_then(Value::as_str).unwrap_or("");
    let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(20) as u32;
    let offset = arguments.get("offset").and_then(Value::as_u64).unwrap_or(0) as u32;
    let page = repository
        .search(SearchQuery::new(query).with_page(limit, offset))
        .map_err(|_| "database_unavailable".to_owned())?;
    serde_json::to_value(page).map_err(|_| "serialization_failed".to_owned())
}

fn get_prompt(
    repository: &PromptRepository,
    arguments: &Map<String, Value>,
) -> Result<Value, String> {
    let prompt = load_prompt(repository, arguments)?;
    let content = prompt.current_version().content();
    Ok(
        json!({"id":prompt.id().value(), "title":content.title(), "body":content.body(), "description":content.description(), "category":content.category(), "tags":content.tags(), "status":prompt.status(), "effectiveness":prompt.effectiveness(), "version":prompt.current_version().number(), "sources":prompt.sources(), "variables":content.variables(), "compatibilities":prompt.compatibilities()}),
    )
}

fn render_prompt(
    repository: &PromptRepository,
    arguments: &Map<String, Value>,
) -> Result<Value, String> {
    let prompt = load_prompt(repository, arguments)?;
    let supplied = arguments
        .get("variables")
        .and_then(Value::as_object)
        .ok_or("variables is required")?;
    let mut text = prompt.current_version().content().body().to_owned();
    let mut missing = Vec::new();
    for variable in prompt.current_version().content().variables() {
        let value = supplied.get(variable.name()).cloned().or_else(|| {
            variable
                .default_value()
                .map(|value| Value::String(value.to_owned()))
        });
        match value.as_ref() {
            Some(value) => {
                text = text.replace(&format!("{{{{{}}}}}", variable.name()), &scalar(value)?)
            }
            None if variable.required() => missing.push(variable.name()),
            None => {}
        }
    }
    Ok(json!({"text":text, "missing_variables":missing}))
}

fn save_draft(
    repository: &mut PromptRepository,
    arguments: &Map<String, Value>,
) -> Result<Value, String> {
    let title = text(arguments, "title")?;
    let body = text(arguments, "body")?;
    let source = arguments
        .get("source")
        .and_then(Value::as_object)
        .ok_or("source is required")?;
    if source.get("kind").and_then(Value::as_str) != Some("mcp") {
        return Err("source kind must be mcp".to_owned());
    }
    let now = OffsetDateTime::now_utc();
    let content = PromptContent::with_variables(
        title,
        body,
        optional_text(arguments, "description"),
        optional_text(arguments, "category"),
        string_array(arguments, "tags"),
        parse_variables(arguments)?,
    )
    .map_err(|error| error.to_string())?;
    let provenance = PromptSource::new(
        SourceKind::Mcp,
        text(source, "name")?,
        optional_text(source, "location"),
        now,
    )
    .map_err(|error| error.to_string())?;
    let prompt = Prompt::new_inbox(content, provenance, Actor::Mcp, now);
    repository
        .save(&prompt, prompt_domain::AuditAction::Created)
        .map_err(|_| "database_write_failed".to_owned())?;
    Ok(json!({"draft_id":prompt.id().value(), "status":"inbox"}))
}

fn load_prompt(
    repository: &PromptRepository,
    arguments: &Map<String, Value>,
) -> Result<Prompt, String> {
    let id = Uuid::parse_str(&text(arguments, "id")?).map_err(|_| "invalid prompt id")?;
    repository
        .get(PromptId::from_uuid(id))
        .map_err(|_| "database_unavailable".to_owned())?
        .ok_or("prompt_not_found".to_owned())
}
fn parse_variables(arguments: &Map<String, Value>) -> Result<Vec<PromptVariable>, String> {
    arguments
        .get("variables")
        .and_then(Value::as_array)
        .map_or(Ok(Vec::new()), |items| {
            items
                .iter()
                .map(|item| {
                    let item = item.as_object().ok_or("invalid variable")?;
                    PromptVariable::new(
                        text(item, "name")?,
                        serde_json::from_value(
                            item.get("kind")
                                .cloned()
                                .ok_or("variable kind is required")?,
                        )
                        .map_err(|_| "invalid variable kind")?,
                        optional_text(item, "description"),
                        item.get("default_value")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        item.get("required")
                            .and_then(Value::as_bool)
                            .ok_or("variable required flag is required")?,
                    )
                    .map_err(|error| error.to_string())
                })
                .collect()
        })
}
fn text(values: &Map<String, Value>, name: &str) -> Result<String, String> {
    values
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{name} is required"))
}
fn optional_text(values: &Map<String, Value>, name: &str) -> Option<String> {
    values.get(name).and_then(Value::as_str).map(str::to_owned)
}
fn string_array(values: &Map<String, Value>, name: &str) -> Vec<String> {
    values
        .get(name)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
fn scalar(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        _ => Err("variable values must be scalar".to_owned()),
    }
}
fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":code,"message":message}})
}
