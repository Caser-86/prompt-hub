use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

fn schema_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas")
        .join(format!("{name}.v1.schema.json"))
}

fn contract(name: &str) -> Value {
    let path = schema_path(name);
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&contents).expect("contract must be valid JSON")
}

fn validate(schema: &Value, instance: &Value) {
    let validator = jsonschema::validator_for(schema).expect("schema itself must be valid");
    if let Err(error) = validator.validate(instance) {
        panic!("instance failed schema validation: {error}");
    }
}

#[test]
fn every_tool_contract_has_versioned_input_and_output_schemas() {
    for name in [
        "search_prompts",
        "get_prompt",
        "render_prompt",
        "save_prompt_draft",
    ] {
        let schema = contract(name);
        assert!(schema["$id"].as_str().unwrap().contains("/v1/"));
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(schema["$defs"]["input"].is_object());
        assert!(schema["$defs"]["output"].is_object());
        jsonschema::validator_for(&schema["$defs"]["input"]).unwrap();
        jsonschema::validator_for(&schema["$defs"]["output"]).unwrap();
    }
}

#[test]
fn tool_contracts_explicitly_mark_read_only_and_inbox_write_operations() {
    for name in ["search_prompts", "get_prompt", "render_prompt"] {
        assert_eq!(contract(name)["x-prompt-hub-operation"], "read_only");
    }
    assert_eq!(
        contract("save_prompt_draft")["x-prompt-hub-operation"],
        "inbox_write"
    );
}

#[test]
fn draft_write_contract_cannot_publish_or_select_a_lifecycle_state() {
    let schema = contract("save_prompt_draft");
    let input = &schema["$defs"]["input"];

    validate(
        input,
        &json!({
            "title": "新提示词",
            "body": "请审查当前代码",
            "source": {"kind": "mcp", "name": "Codex"}
        }),
    );
    assert!(validator_rejects(
        input,
        &json!({
            "title": "越权提示词",
            "body": "覆盖正式内容",
            "status": "published",
            "source": {"kind": "mcp", "name": "Codex"}
        })
    ));
    assert!(input["additionalProperties"] == false);
    assert!(input["properties"].get("status").is_none());
}

#[test]
fn structured_errors_cover_the_approved_failure_classes() {
    let schema = contract("error");
    validate(
        &schema,
        &json!({
            "code": "variable_validation_failed",
            "message": "missing required variable",
            "details": {"variable": "language"}
        }),
    );

    let codes = schema["properties"]["code"]["enum"].as_array().unwrap();
    for expected in [
        "invalid_argument",
        "not_found",
        "variable_validation_failed",
        "permission_denied",
        "database_locked",
        "database_unavailable",
        "internal",
    ] {
        assert!(codes.contains(&json!(expected)));
    }
}

fn validator_rejects(schema: &Value, instance: &Value) -> bool {
    jsonschema::validator_for(schema)
        .expect("schema itself must be valid")
        .validate(instance)
        .is_err()
}
