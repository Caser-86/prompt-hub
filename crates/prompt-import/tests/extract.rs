use prompt_import::extract_readable_text;

#[test]
fn extracts_readable_html_without_executing_or_retaining_scripts() {
    let extracted = extract_readable_text(
        "<html><head><title>提示词页面</title><script>window.secret = 'do not keep'</script></head><body><h1>代码审查</h1><p>审查当前变更。</p><img src='https://example.com/pixel'></body></html>",
    );

    assert_eq!(extracted.title.as_deref(), Some("提示词页面"));
    assert!(extracted.text.contains("代码审查"));
    assert!(extracted.text.contains("审查当前变更"));
    assert!(!extracted.text.contains("window.secret"));
    assert!(!extracted.text.contains("pixel"));
}
