use url::Url;

use crate::state::Param;

pub fn params_from_url(url: &str) -> Option<Vec<Param>> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(parsed) = parse_url_with_fallback(url) {
        let mut params = Vec::new();
        for (key, value) in parsed.query_pairs() {
            params.push(Param {
                enable: true,
                key: key.to_string(),
                value: value.to_string(),
            });
        }
        if params.is_empty() {
            params.push(Param {
                enable: true,
                key: String::new(),
                value: String::new(),
            });
        }
        return Some(params);
    }

    let query = trimmed.splitn(2, '?').nth(1).unwrap_or("");
    let mut params: Vec<Param> = url::form_urlencoded::parse(query.as_bytes())
        .map(|(key, value)| Param {
            enable: true,
            key: key.to_string(),
            value: value.to_string(),
        })
        .collect();

    if params.is_empty() {
        params.push(Param {
            enable: true,
            key: String::new(),
            value: String::new(),
        });
    }

    Some(params)
}

fn parse_url_with_fallback(value: &str) -> Option<Url> {
    if let Ok(url) = Url::parse(value) {
        return Some(url);
    }

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let base = "http://localhost";
    if trimmed.starts_with('/') {
        Url::parse(&format!("{base}{trimmed}")).ok()
    } else if trimmed.starts_with('?') {
        Url::parse(&format!("{base}/{}", trimmed)).ok()
    } else {
        Url::parse(&format!("{base}/{trimmed}")).ok()
    }
}

pub fn url_from_params(url: &str, _params: &[Param]) -> String {
    let base = url.split('?').next().unwrap_or(url);
    if base.is_empty() {
        "/".to_string()
    } else {
        base.to_string()
    }
}

pub fn path_params_from_url(url: &str, existing: &[Param]) -> Vec<Param> {
    let keys = extract_path_keys(url);
    if keys.is_empty() {
        return vec![Param {
            enable: true,
            key: String::new(),
            value: String::new(),
        }];
    }

    keys.into_iter()
        .map(|key| {
            if let Some(existing) = existing.iter().find(|param| param.key == key) {
                Param {
                    enable: existing.enable,
                    key,
                    value: existing.value.clone(),
                }
            } else {
                Param {
                    enable: true,
                    key,
                    value: String::new(),
                }
            }
        })
        .collect()
}

pub fn url_from_path_params(url: &str, params: &[Param]) -> String {
    let (base, query) = split_url(url);
    let mut result = String::new();
    let mut placeholder_index = 0;
    let mut chars = base.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut key = String::new();
            while let Some(next) = chars.next() {
                if next == '}' {
                    break;
                }
                key.push(next);
            }
            let param = params.get(placeholder_index);
            let next_key = param.map(|value| value.key.trim()).unwrap_or("");
            if next_key.is_empty() {
                result.push('{');
                if !key.is_empty() {
                    result.push_str(&key);
                }
                result.push('}');
            } else {
                result.push('{');
                result.push_str(next_key);
                result.push('}');
            }
            placeholder_index += 1;
            continue;
        }
        result.push(ch);
    }

    for param in params.iter().skip(placeholder_index) {
        let key = param.key.trim();
        if key.is_empty() {
            result.push_str("/{}");
        } else {
            if !result.ends_with('/') {
                result.push('/');
            }
            result.push('{');
            result.push_str(key);
            result.push('}');
        }
    }

    if result.is_empty() {
        result.push('/');
    }

    if let Some(query) = query {
        if !query.is_empty() {
            result.push('?');
            result.push_str(&query);
        }
    }

    result
}

fn extract_path_keys(url: &str) -> Vec<String> {
    let (base, _) = split_url(url);
    let mut keys = Vec::new();
    let mut chars = base.chars().peekable();
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
                keys.push(trimmed.to_string());
            }
        }
    }
    keys
}

fn split_url(url: &str) -> (String, Option<String>) {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }

    if let Ok(parsed) = Url::parse(trimmed) {
        let path = parsed.path().to_string();
        let query = parsed.query().map(|value| value.to_string());
        return (path, query);
    }

    let parts: Vec<&str> = trimmed.splitn(2, '?').collect();
    let path = parts.first().unwrap_or(&"").to_string();
    let query = if parts.len() > 1 {
        Some(parts[1].to_string())
    } else {
        None
    };

    let normalized = if path.starts_with('/') || path.is_empty() {
        path
    } else {
        format!("/{}", path)
    };

    (normalized, query)
}
