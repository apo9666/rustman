use yew::prelude::*;

pub fn parse_json_value(data: &str) -> Option<serde_json::Value> {
    serde_json::from_str(normalize_json_input(data)).ok()
}

pub fn highlight_json(data: &str) -> Option<Html> {
    let value = parse_json_value(data)?;
    Some(render_json_value(&value, 0))
}

fn normalize_json_input(data: &str) -> &str {
    let data = data.trim_start_matches('\u{FEFF}');
    data.trim()
}

fn render_json_value(value: &serde_json::Value, indent: usize) -> Html {
    match value {
        serde_json::Value::Null => json_span("json-null", "null"),
        serde_json::Value::Bool(flag) => {
            json_span("json-bool", if *flag { "true" } else { "false" })
        }
        serde_json::Value::Number(number) => json_span("json-number", &number.to_string()),
        serde_json::Value::String(text) => {
            let escaped = serde_json::to_string(text).unwrap_or_else(|_| format!("\"{text}\""));
            json_span("json-string", &escaped)
        }
        serde_json::Value::Array(items) => render_json_array(items, indent),
        serde_json::Value::Object(map) => render_json_object(map, indent),
    }
}

fn render_json_array(items: &[serde_json::Value], indent: usize) -> Html {
    let mut nodes: Vec<Html> = Vec::new();
    nodes.push(json_span("json-punct", "["));

    if !items.is_empty() {
        nodes.push(text_node("\n"));
        let next_indent = indent + 1;
        let indent_str = "  ".repeat(next_indent);
        for (index, item) in items.iter().enumerate() {
            nodes.push(text_node(&indent_str));
            nodes.push(render_json_value(item, next_indent));
            if index + 1 < items.len() {
                nodes.push(json_span("json-punct", ","));
            }
            nodes.push(text_node("\n"));
        }
        nodes.push(text_node(&"  ".repeat(indent)));
    }

    nodes.push(json_span("json-punct", "]"));
    html! { <>{ for nodes }</> }
}

fn render_json_object(map: &serde_json::Map<String, serde_json::Value>, indent: usize) -> Html {
    let mut nodes: Vec<Html> = Vec::new();
    nodes.push(json_span("json-punct", "{"));

    if !map.is_empty() {
        nodes.push(text_node("\n"));
        let next_indent = indent + 1;
        let indent_str = "  ".repeat(next_indent);
        let len = map.len();
        for (index, (key, value)) in map.iter().enumerate() {
            nodes.push(text_node(&indent_str));
            let escaped = serde_json::to_string(key).unwrap_or_else(|_| format!("\"{key}\""));
            nodes.push(json_span("json-key", &escaped));
            nodes.push(text_node(": "));
            nodes.push(render_json_value(value, next_indent));
            if index + 1 < len {
                nodes.push(json_span("json-punct", ","));
            }
            nodes.push(text_node("\n"));
        }
        nodes.push(text_node(&"  ".repeat(indent)));
    }

    nodes.push(json_span("json-punct", "}"));
    html! { <>{ for nodes }</> }
}

fn json_span(class_name: &str, text: &str) -> Html {
    html! { <span class={class_name.to_string()}>{ text }</span> }
}

fn text_node(text: &str) -> Html {
    html! { { text } }
}
