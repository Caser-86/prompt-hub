use std::io::Write;
use std::process::{Command, Stdio};

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
