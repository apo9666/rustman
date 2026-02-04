use std::collections::{BTreeSet, HashSet};

use serde_json::{json, Map, Value};
use url::Url;

use crate::state::{MethodEnum, Param, TabContent, TreeNode};

pub fn build_tree_from_openapi(text: &str) -> Result<(TreeNode, Vec<String>), String> {
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

    let server_list: Vec<String> = servers
        .iter()
        .filter_map(|server| server.get("url").and_then(|value| value.as_str()))
        .map(|value| value.to_string())
        .collect();

    let mut servers = server_list;
    if servers.is_empty() && !paths.is_empty() {
        servers.push("http://localhost".to_string());
    }

    let mut tag_map: std::collections::BTreeMap<String, Vec<TreeNode>> =
        std::collections::BTreeMap::new();
    let mut root_nodes: Vec<TreeNode> = Vec::new();
    let mut path_entries: Vec<_> = paths.iter().collect();
    path_entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (path_key, path_value) in path_entries {
        let Some(path_obj) = path_value.as_object() else {
            continue;
        };
        for method in MethodEnum::all() {
            let method_key = method.key();
            let Some(method_value) = path_obj.get(method_key) else {
                continue;
            };
            let content = convert_content(path_key, *method, method_value);
            let node = TreeNode {
                label: path_key.clone(),
                content: Some(content),
                expanded: false,
                children: Vec::new(),
            };
            let tag_label = method_value
                .get("tags")
                .and_then(|value| value.as_array())
                .and_then(|tags| tags.first())
                .and_then(|value| value.as_str())
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());

            if let Some(tag_label) = tag_label {
                tag_map.entry(tag_label.to_string()).or_default().push(node);
            } else {
                root_nodes.push(node);
            }
        }
    }

    let mut tag_nodes: Vec<TreeNode> = tag_map
        .into_iter()
        .map(|(label, children)| TreeNode {
            label,
            content: None,
            expanded: true,
            children,
        })
        .collect();

    let mut children = Vec::new();
    children.append(&mut root_nodes);
    children.append(&mut tag_nodes);

    Ok((
        TreeNode {
            label: title,
            content: None,
            expanded: true,
            children,
        },
        servers,
    ))
}

pub fn build_openapi_from_tree(root: &TreeNode, servers: &[String]) -> Result<String, String> {
    build_openapi_from_tree_nodes(root, servers)
}

fn convert_content(path_key: &str, method: MethodEnum, method_value: &Value) -> TabContent {
    let path = normalize_path(path_key);
    let body = extract_body(method_value);
    let path_params = extract_path_params(path_key);
    TabContent {
        url: path,
        method,
        body,
        path_params,
        ..TabContent::default()
    }
}

fn extract_body(method_value: &Value) -> String {
    let Some(example) = method_value
        .get("requestBody")
        .and_then(|value| value.get("content"))
        .and_then(|value| value.get("application/json"))
        .and_then(|value| value.get("example"))
    else {
        return String::new();
    };

    serde_json::to_string_pretty(example).unwrap_or_default()
}

fn extract_path_params(path: &str) -> Vec<Param> {
    let mut params = Vec::new();
    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut key = String::new();
            while let Some(next) = chars.next() {
                if next == '}' {
                    break;
                }
                key.push(next);
            }
            let trimmed = key.trim();
            if !trimmed.is_empty() {
                params.push(Param {
                    enable: true,
                    key: trimmed.to_string(),
                    value: String::new(),
                });
            }
        }
    }
    if params.is_empty() {
        params.push(Param {
            enable: true,
            key: String::new(),
            value: String::new(),
        });
    }
    params
}

fn build_openapi_from_tree_nodes(root: &TreeNode, servers: &[String]) -> Result<String, String> {
    if root.children.is_empty() {
        return Err("Nenhuma request para exportar.".to_string());
    }

    let mut servers_set = BTreeSet::new();
    let mut tag_names = BTreeSet::new();
    let mut paths: Map<String, Value> = Map::new();
    let mut seen = HashSet::new();

    let mut push_operation =
        |node: &TreeNode, content: &TabContent, tag_label: Option<&str>| {
            let path_key = normalize_path(&strip_query(&content.url));
            let method_key = content.method.key().to_string();
            let dedupe_key = (method_key.clone(), path_key.clone());
            if !seen.insert(dedupe_key) {
                return;
            }

            let mut operation = Map::new();
            operation.insert("summary".to_string(), Value::String(node.label.clone()));
            if let Some(tag_label) = tag_label {
                if !tag_label.trim().is_empty() {
                    tag_names.insert(tag_label.to_string());
                    operation.insert("tags".to_string(), json!([tag_label]));
                }
            }

        let parameters = build_parameters(content);
        if !parameters.is_empty() {
            operation.insert("parameters".to_string(), Value::Array(parameters));
        }

        if let Some(request_body) = build_request_body(content) {
            operation.insert("requestBody".to_string(), request_body);
        }

        operation.insert("responses".to_string(), json!({ "200": { "description": "OK" } }));

        let path_item = paths
            .entry(path_key)
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(map) = path_item {
            map.insert(method_key, Value::Object(operation));
        }
    };

    for server in servers.iter().filter(|value| !value.trim().is_empty()) {
        servers_set.insert(server.clone());
    }

    for node in &root.children {
        if let Some(content) = node.content.as_ref() {
            push_operation(node, content, None);
            continue;
        }

        let tag_label = node.label.clone();
        for child in &node.children {
            let Some(content) = child.content.as_ref() else {
                continue;
            };
            push_operation(child, content, Some(&tag_label));
        }
    }

    if paths.is_empty() {
        return Err("Nenhuma request válida para exportar.".to_string());
    }

    let server_list: Vec<Value> = servers_set
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
        "tags": tag_names.into_iter().map(|name| json!({ "name": name })).collect::<Vec<_>>(),
        "paths": Value::Object(paths),
    });

    serde_yaml::to_string(&doc).map_err(|err| err.to_string())
}

fn build_parameters(content: &TabContent) -> Vec<Value> {
    let mut parameters = Vec::new();
    let mut seen = HashSet::new();

    for param in content.path_params.iter().filter(|param| param.enable) {
        let key = param.key.trim();
        if key.is_empty() {
            continue;
        }
        push_parameter(
            &mut parameters,
            &mut seen,
            "path",
            key,
            true,
            Some(param.value.trim()),
        );
    }

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
                false,
                Some(param.value.trim()),
            );
        }
    } else {
        for (key, value) in parse_query_pairs(&content.url) {
            push_parameter(
                &mut parameters,
                &mut seen,
                "query",
                &key,
                false,
                Some(&value),
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
            false,
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
    required: bool,
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
    param.insert("required".to_string(), Value::Bool(required));
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

fn parse_query_pairs(value: &str) -> Vec<(String, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let parsed = if let Ok(url) = Url::parse(trimmed) {
        url
    } else {
        let base = "http://localhost";
        let candidate = if trimmed.starts_with('/') {
            format!("{base}{trimmed}")
        } else {
            format!("{base}/{trimmed}")
        };
        Url::parse(&candidate).unwrap_or_else(|_| Url::parse(base).expect("valid base"))
    };

    parsed
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn strip_query(value: &str) -> String {
    value.split('?').next().unwrap_or("").to_string()
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        "/".to_string()
    } else if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    }
}
