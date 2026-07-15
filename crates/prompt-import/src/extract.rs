#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadableText {
    pub title: Option<String>,
    pub text: String,
    pub warnings: Vec<String>,
}

#[must_use]
pub fn extract_readable_text(html: &str) -> ReadableText {
    let title = extract_element(html, "title").map(|value| strip_tags(&value));
    let without_unsafe_blocks =
        remove_element_blocks(&remove_element_blocks(html, "script"), "style");
    let text = strip_tags(&without_unsafe_blocks);
    ReadableText {
        title: title.filter(|value| !value.is_empty()),
        text,
        warnings: Vec::new(),
    }
}

fn extract_element(input: &str, name: &str) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    let open_start = lower.find(&format!("<{name}"))?;
    let content_start = lower[open_start..].find('>')? + open_start + 1;
    let close_start = lower[content_start..].find(&format!("</{name}>"))? + content_start;
    Some(input[content_start..close_start].to_owned())
}

fn remove_element_blocks(input: &str, name: &str) -> String {
    let mut output = input.to_owned();
    loop {
        let lower = output.to_ascii_lowercase();
        let Some(start) = lower.find(&format!("<{name}")) else {
            return output;
        };
        let Some(open_end) = lower[start..].find('>') else {
            return output;
        };
        let content_start = start + open_end + 1;
        let Some(close_offset) = lower[content_start..].find(&format!("</{name}>")) else {
            output.truncate(start);
            return output;
        };
        let close_end = content_start + close_offset + name.len() + 3;
        output.replace_range(start..close_end, " ");
    }
}

fn strip_tags(input: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in input.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
