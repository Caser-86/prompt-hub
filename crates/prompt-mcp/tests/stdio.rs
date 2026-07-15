use std::io::Write;
use std::process::{Command, Stdio};

use prompt_domain::{
    Actor, AuditAction, Prompt, PromptContent, PromptSource, PromptVariable, SourceKind,
    VariableKind,
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
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"search_prompts\",\"arguments\":{}}}\n")
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
fn stdio_server_calls_read_tools_and_creates_only_an_inbox_draft() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("prompt-hub.db");
    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let prompt = Prompt::new_inbox(
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
