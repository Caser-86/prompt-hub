use std::io::Write;
use std::process::{Command, Stdio};

use prompt_domain::{
    Actor, AuditAction, EffectivenessStatus, Prompt, PromptContent, PromptSource, PromptVariable,
    SourceKind, VariableKind,
};
use prompt_store::Database;
use tempfile::tempdir;
use time::macros::datetime;

#[test]
fn stdio_server_discovers_only_approved_tools() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_prompt-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n")
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let names = response["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "search_prompts",
            "get_prompt",
            "render_prompt",
            "save_prompt_draft"
        ]
    );
}

#[test]
fn stdio_server_returns_a_structured_database_unavailable_error() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_prompt-mcp"))
        .env_remove("PROMPT_HUB_DATABASE_PATH")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!(
            "../../../tests/fixtures/mcp/tools-call.json"
        ))
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let error: serde_json::Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();

    assert_eq!(error["code"], "database_unavailable");
    assert!(error["message"].as_str().unwrap().contains("database"));
}

#[test]
fn stdio_server_returns_a_structured_invalid_argument_error_from_fixture() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_prompt-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(include_bytes!(
            "../../../tests/fixtures/mcp/invalid-tools-call.json"
        ))
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let error: serde_json::Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();

    assert_eq!(error["code"], "invalid_argument");
}

#[test]
fn stdio_server_calls_read_tools_and_creates_only_an_inbox_draft() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("prompt-hub.db");
    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let mut prompt = Prompt::new_inbox(
        PromptContent::with_variables(
            "代码审查",
            "审查 {{language}} 代码",
            None,
            Some("开发".to_owned()),
            vec!["审查".to_owned()],
            vec![PromptVariable::new("language", VariableKind::Text, None, None, true).unwrap()],
        )
        .unwrap(),
        PromptSource::new(
            SourceKind::Manual,
            "手动录入",
            None,
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    );
    repository.save(&prompt, AuditAction::Created).unwrap();
    prompt
        .publish(
            EffectivenessStatus::Unverified,
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();
    repository.save(&prompt, AuditAction::Published).unwrap();
    drop(repository);

    let mut child = Command::new(env!("CARGO_BIN_EXE_prompt-mcp"))
        .env("PROMPT_HUB_DATABASE_PATH", &path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let request = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"search_prompts\",\"arguments\":{{\"query\":\"代码\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{{\"name\":\"get_prompt\",\"arguments\":{{\"id\":\"{}\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{{\"name\":\"render_prompt\",\"arguments\":{{\"id\":\"{}\",\"variables\":{{\"language\":\"Rust\"}}}}}}}}\n{{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{{\"name\":\"save_prompt_draft\",\"arguments\":{{\"title\":\"MCP 草稿\",\"body\":\"只进入收件箱\",\"source\":{{\"kind\":\"mcp\",\"name\":\"Codex\"}}}}}}}}\n",
        prompt.id().value(),
        prompt.id().value(),
    );
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(request.as_bytes())
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    let responses = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("代码审查")
    );
    assert!(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("代码审查"),
    );
    assert!(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Rust"),
    );
    assert!(
        responses[3]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"status\":\"inbox\""),
    );
}

#[test]
fn stdio_server_enforces_runtime_schema_and_published_only_reads() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("prompt-hub.db");
    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let inbox = prompt("Inbox secret");
    repository.save(&inbox, AuditAction::Created).unwrap();
    let mut published = prompt("Published prompt");
    published
        .publish(
            EffectivenessStatus::Unverified,
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();
    repository.save(&published, AuditAction::Published).unwrap();
    drop(repository);

    let request = format!(
        concat!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"search_prompts\",\"arguments\":{{\"query\":\"\"}}}}}}\n",
            "{{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{{\"name\":\"search_prompts\",\"arguments\":{{\"filters\":{{\"status\":\"inbox\"}}}}}}}}\n",
            "{{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{{\"name\":\"get_prompt\",\"arguments\":{{\"id\":\"{}\"}}}}}}\n",
            "{{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{{\"name\":\"get_prompt\",\"arguments\":{{\"id\":\"{}\"}}}}}}\n",
            "{{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{{\"name\":\"save_prompt_draft\",\"arguments\":{{\"title\":\"Escalation\",\"body\":\"Must remain inbox\",\"status\":\"published\",\"source\":{{\"kind\":\"mcp\",\"name\":\"Codex\"}}}}}}}}\n"
        ),
        inbox.id().value(),
        published.id().value(),
    );
    let responses = run_mcp(&path, request.as_bytes());

    let search: serde_json::Value = tool_payload(&responses[0]);
    assert_eq!(search["total"], 1);
    assert_eq!(search["hits"][0]["id"], published.id().value().to_string());
    let invalid_filter: serde_json::Value = tool_payload(&responses[1]);
    assert_eq!(invalid_filter["code"], "invalid_argument");
    let denied: serde_json::Value = tool_payload(&responses[2]);
    assert_eq!(denied["code"], "permission_denied");
    assert!(
        responses[3]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Published prompt")
    );
    let lifecycle_override: serde_json::Value = tool_payload(&responses[4]);
    assert_eq!(lifecycle_override["code"], "invalid_argument");
}

#[test]
fn stdio_server_rejects_oversized_requests_and_recovers_at_the_next_line() {
    let mut oversized = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"padding\":\"{}\"}}\n",
        "x".repeat(1024 * 1024)
    );
    oversized.push_str("{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n");

    let mut child = Command::new(env!("CARGO_BIN_EXE_prompt-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(oversized.as_bytes())
        .unwrap();
    drop(child.stdin.take());
    let responses = String::from_utf8(child.wait_with_output().unwrap().stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(responses[0]["error"]["code"], -32600);
    assert_eq!(responses[0]["error"]["message"], "request too large");
    assert_eq!(responses[1]["id"], 2);
    assert_eq!(responses[1]["result"]["tools"].as_array().unwrap().len(), 4);
}

fn prompt(title: &str) -> Prompt {
    Prompt::new_inbox(
        PromptContent::new(title, "Body", None, Some("security".to_owned()), Vec::new()).unwrap(),
        PromptSource::new(
            SourceKind::Manual,
            "manual",
            None,
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    )
}

fn run_mcp(path: &std::path::Path, request: &[u8]) -> Vec<serde_json::Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_prompt-mcp"))
        .env("PROMPT_HUB_DATABASE_PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(request).unwrap();
    drop(child.stdin.take());
    String::from_utf8(child.wait_with_output().unwrap().stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn tool_payload(response: &serde_json::Value) -> serde_json::Value {
    serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}
