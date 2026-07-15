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
