use std::collections::{BTreeSet, HashSet};

use serde_json::{json, Map, Value};
use url::Url;

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

pub fn build_openapi_from_tree(root: &TreeNode) -> Result<String, String> {
    let mut entries = Vec::new();
    collect_endpoints(root, &mut entries);
    build_openapi_from_entries(root, entries)
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

fn build_openapi_from_entries(
    root: &TreeNode,
    entries: Vec<(String, TabContent)>,
) -> Result<String, String> {
    if entries.is_empty() {
        return Err("Nenhuma request para exportar.".to_string());
    }

    let mut servers = BTreeSet::new();
    let mut paths: Map<String, Value> = Map::new();
    let mut seen = HashSet::new();
    let mut invalid_urls = 0;

    for (label, content) in entries {
        let Some(url) = parse_request_url(&content.url) else {
            invalid_urls += 1;
            continue;
        };

        let key = (content.method.key().to_string(), content.url.clone());
        if !seen.insert(key) {
            continue;
        }

        let origin = url.origin().unicode_serialization();
        if !origin.is_empty() && origin != "null" {
            servers.insert(origin);
        }

        let path = url.path();
        let path_key = if path.is_empty() {
            "/".to_string()
        } else {
            path.to_string()
        };
        let method_key = content.method.key().to_string();

        let mut operation = Map::new();
        operation.insert("summary".to_string(), Value::String(label));

        let parameters = build_parameters(&content, &url);
        if !parameters.is_empty() {
            operation.insert("parameters".to_string(), Value::Array(parameters));
        }

        if let Some(request_body) = build_request_body(&content) {
            operation.insert("requestBody".to_string(), request_body);
        }

        operation.insert("responses".to_string(), json!({ "200": { "description": "OK" } }));

        let path_item = paths
            .entry(path_key)
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(map) = path_item {
            map.insert(method_key, Value::Object(operation));
        }
    }

    if paths.is_empty() {
        if invalid_urls > 0 {
            return Err(
                "Nenhuma request válida para exportar. Verifique se a URL começa com http:// ou https://."
                    .to_string(),
            );
        }
        return Err("Nenhuma request válida para exportar.".to_string());
    }

    let server_list: Vec<Value> = servers
        .into_iter()
        .map(|url| json!({ "url": url }))
        .collect();

    let title = if root.label == "Root" {
        "Rustman".to_string()
    } else {
        root.label.clone()
    };

    let doc = json!({
        "openapi": "3.0.3",
        "info": {
            "title": title,
            "version": "1.0.0",
        },
        "servers": server_list,
        "paths": Value::Object(paths),
    });

    serde_yaml::to_string(&doc).map_err(|err| err.to_string())
}

fn parse_request_url(value: &str) -> Option<Url> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(url) = Url::parse(trimmed) {
        return Some(url);
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return None;
    }

    let candidate = format!("https://{trimmed}");
    Url::parse(&candidate).ok()
}

fn collect_endpoints(node: &TreeNode, out: &mut Vec<(String, TabContent)>) {
    if let Some(content) = node.content.clone() {
        out.push((node.label.clone(), content));
    }

    for child in &node.children {
        collect_endpoints(child, out);
    }
}

fn build_parameters(content: &TabContent, url: &Url) -> Vec<Value> {
    let mut parameters = Vec::new();
    let mut seen = HashSet::new();

    let has_explicit_params = content
        .params
        .iter()
        .any(|param| param.enable && !param.key.trim().is_empty());

    if has_explicit_params {
        for param in content.params.iter().filter(|param| param.enable) {
            let key = param.key.trim();
            if key.is_empty() {
                continue;
            }
            push_parameter(
                &mut parameters,
                &mut seen,
                "query",
                key,
                Some(param.value.trim()),
            );
        }
    } else {
        for (key, value) in url.query_pairs() {
            push_parameter(
                &mut parameters,
                &mut seen,
                "query",
                &key,
                Some(value.as_ref()),
            );
        }
    }

    for header in content.headers.iter().filter(|header| header.enable) {
        let key = header.key.trim();
        if key.is_empty() {
            continue;
        }
        if matches_ignore_case(key, "content-type") || matches_ignore_case(key, "accept") {
            continue;
        }
        push_parameter(
            &mut parameters,
            &mut seen,
            "header",
            key,
            Some(header.value.trim()),
        );
    }

    parameters
}

fn push_parameter(
    parameters: &mut Vec<Value>,
    seen: &mut HashSet<(String, String)>,
    location: &str,
    name: &str,
    example: Option<&str>,
) {
    let key = (location.to_string(), name.to_string());
    if seen.contains(&key) {
        return;
    }
    seen.insert(key);

    let mut param = Map::new();
    param.insert("name".to_string(), Value::String(name.to_string()));
    param.insert("in".to_string(), Value::String(location.to_string()));
    param.insert("required".to_string(), Value::Bool(false));
    param.insert(
        "schema".to_string(),
        json!({
            "type": "string"
        }),
    );

    if let Some(example) = example {
        let value = example.trim();
        if !value.is_empty() {
            param.insert("example".to_string(), Value::String(value.to_string()));
        }
    }

    parameters.push(Value::Object(param));
}

fn build_request_body(content: &TabContent) -> Option<Value> {
    let body = content.body.trim();
    if body.is_empty() {
        return None;
    }

    if let Ok(value) = serde_json::from_str::<Value>(body) {
        Some(json!({
            "required": false,
            "content": {
                "application/json": {
                    "example": value
                }
            }
        }))
    } else {
        Some(json!({
            "required": false,
            "content": {
                "text/plain": {
                    "example": body
                }
            }
        }))
    }
}

fn matches_ignore_case(value: &str, other: &str) -> bool {
    value.eq_ignore_ascii_case(other)
}
