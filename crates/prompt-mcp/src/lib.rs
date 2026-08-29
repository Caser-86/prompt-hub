#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolDescriptor {
    pub name: &'static str,
    pub input_schema_id: &'static str,
    pub operation: ToolOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolOperation {
    ReadOnly,
    InboxWrite,
}

#[must_use]
pub fn tool_input_schema(name: &str) -> Option<serde_json::Value> {
    let document = match name {
        "search_prompts" => include_str!("../schemas/search_prompts.v1.schema.json"),
        "get_prompt" => include_str!("../schemas/get_prompt.v1.schema.json"),
        "render_prompt" => include_str!("../schemas/render_prompt.v1.schema.json"),
        "save_prompt_draft" => include_str!("../schemas/save_prompt_draft.v1.schema.json"),
        _ => return None,
    };
    serde_json::from_str::<serde_json::Value>(document)
        .ok()?
        .pointer("/$defs/input")
        .cloned()
}

pub fn validate_tool_arguments(name: &str, arguments: &serde_json::Value) -> Result<(), String> {
    let schema = tool_input_schema(name).ok_or_else(|| "unknown tool".to_owned())?;
    let validator =
        jsonschema::validator_for(&schema).map_err(|_| "tool schema is unavailable".to_owned())?;
    validator
        .validate(arguments)
        .map_err(|_| "tool arguments do not match the published schema".to_owned())
}

#[must_use]
pub const fn approved_tools() -> [ToolDescriptor; 4] {
    [
        ToolDescriptor {
            name: "search_prompts",
            input_schema_id: "https://prompt-hub.local/schemas/v1/search_prompts",
            operation: ToolOperation::ReadOnly,
        },
        ToolDescriptor {
            name: "get_prompt",
            input_schema_id: "https://prompt-hub.local/schemas/v1/get_prompt",
            operation: ToolOperation::ReadOnly,
        },
        ToolDescriptor {
            name: "render_prompt",
            input_schema_id: "https://prompt-hub.local/schemas/v1/render_prompt",
            operation: ToolOperation::ReadOnly,
        },
        ToolDescriptor {
            name: "save_prompt_draft",
            input_schema_id: "https://prompt-hub.local/schemas/v1/save_prompt_draft",
            operation: ToolOperation::InboxWrite,
        },
    ]
}
