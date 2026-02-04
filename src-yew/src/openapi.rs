use serde_json::Value;

use crate::state::{MethodEnum, TabContent, TreeNode};

pub fn build_tree_from_openapi(text: &str) -> Result<TreeNode, String> {
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|err| format!("YAML parse error: {err}"))?;
    let json: Value = serde_json::to_value(yaml)
        .map_err(|err| format!("OpenAPI conversion error: {err}"))?;

    let title = json
        .pointer("/info/title")
        .and_then(|value| value.as_str())
        .unwrap_or("OpenAPI")
        .to_string();

    let servers = json
        .get("servers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let paths = json
        .get("paths")
        .and_then(|value| value.as_object())
        .cloned()
        .unwrap_or_default();

    let mut server_nodes = Vec::new();
    for server in servers {
        let Some(url) = server.get("url").and_then(|value| value.as_str()) else {
            continue;
        };

        let mut path_nodes = Vec::new();
        let mut path_entries: Vec<_> = paths.iter().collect();
        path_entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (path_key, path_value) in path_entries {
            let content = convert_content(url, path_key, path_value);
            path_nodes.push(TreeNode {
                label: path_key.clone(),
                content: Some(content),
                expanded: false,
                children: Vec::new(),
            });
        }

        server_nodes.push(TreeNode {
            label: url.to_string(),
            content: None,
            expanded: false,
            children: path_nodes,
        });
    }

    Ok(TreeNode {
        label: title,
        content: None,
        expanded: true,
        children: server_nodes,
    })
}

fn convert_content(server: &str, key: &str, path: &Value) -> TabContent {
    let (method, body) = find_method_and_body(path);
    let server = server.trim_end_matches('/');
    let path = if key.starts_with('/') {
        key.to_string()
    } else {
        format!("/{}", key)
    };

    TabContent {
        url: format!("{}{}", server, path),
        method,
        body,
        ..TabContent::default()
    }
}

fn find_method_and_body(path: &Value) -> (MethodEnum, String) {
    let methods = [
        MethodEnum::Get,
        MethodEnum::Post,
        MethodEnum::Put,
        MethodEnum::Patch,
        MethodEnum::Delete,
        MethodEnum::Options,
        MethodEnum::Head,
        MethodEnum::Trace,
    ];

    for method in methods {
        if path.get(method.key()).is_some() {
            let body = extract_body(path, method.key());
            return (method, body);
        }
    }

    (MethodEnum::Get, String::new())
}

fn extract_body(path: &Value, method_key: &str) -> String {
    let Some(method) = path.get(method_key) else {
        return String::new();
    };
    let Some(example) = method
        .get("requestBody")
        .and_then(|value| value.get("content"))
        .and_then(|value| value.get("application/json"))
        .and_then(|value| value.get("example"))
    else {
        return String::new();
    };

    serde_json::to_string_pretty(example).unwrap_or_default()
}
