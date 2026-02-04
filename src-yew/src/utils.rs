use url::Url;

use crate::state::Param;

pub fn params_from_url(url: &str) -> Option<Vec<Param>> {
    let parsed = parse_url_with_fallback(url)?;
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
