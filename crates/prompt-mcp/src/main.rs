use std::io::{self, BufRead, Write};

use prompt_mcp::{ToolOperation, approved_tools};
use serde_json::{Value, json};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        let response = handle_request(&line);
        if writeln!(stdout, "{response}").is_err() {
            break;
        }
    }
}

fn handle_request(line: &str) -> Value {
    let request: Value = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(_) => {
            return json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": "parse error" }
            });
        }
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        Some("tools/list") => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": approved_tools().into_iter().map(|tool| json!({
                "name": tool.name,
                "description": match tool.operation {
                    ToolOperation::ReadOnly => "Prompt Hub local read-only tool",
                    ToolOperation::InboxWrite => "Prompt Hub tool that creates an inbox draft",
                },
                "inputSchema": { "$ref": tool.input_schema_id }
            })).collect::<Vec<_>>() }
        }),
        Some(_) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": "method not found" }
        }),
        None => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32600, "message": "invalid request" }
        }),
    }
}
