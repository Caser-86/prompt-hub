use prompt_mcp::approved_tools;

#[test]
fn exposes_only_the_four_approved_versioned_tools() {
    let tools = approved_tools();

    assert_eq!(
        tools.iter().map(|tool| tool.name).collect::<Vec<_>>(),
        [
            "search_prompts",
            "get_prompt",
            "render_prompt",
            "save_prompt_draft"
        ]
    );
    assert!(
        tools
            .iter()
            .all(|tool| tool.input_schema_id.contains("/v1/"))
    );
}
