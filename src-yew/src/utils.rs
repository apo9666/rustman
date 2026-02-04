use url::Url;

use crate::state::Param;

pub fn params_from_url(url: &str) -> Option<Vec<Param>> {
    let parsed = Url::parse(url).ok()?;
    let mut params = Vec::new();
    for (key, value) in parsed.query_pairs() {
        params.push(Param {
            enable: true,
            key: key.to_string(),
            value: value.to_string(),
        });
    }
    params.push(Param {
        enable: true,
        key: String::new(),
        value: String::new(),
    });
    Some(params)
}

pub fn url_from_params(url: &str, params: &[Param]) -> String {
    let base = url.split('?').next().unwrap_or(url);
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for param in params {
        if !param.enable {
            continue;
        }
        if param.key.trim().is_empty() {
            continue;
        }
        serializer.append_pair(&param.key, &param.value);
    }
    let query = serializer.finish();
    if query.is_empty() {
        base.to_string()
    } else {
        format!("{}?{}", base, query)
    }
}

pub fn ensure_trailing_param(params: &mut Vec<Param>) {
    let needs_trailing = params.last().map(|param| {
        !param.key.trim().is_empty() || !param.value.trim().is_empty() || !param.enable
    });
    if needs_trailing.unwrap_or(true) {
        params.push(Param {
            enable: true,
            key: String::new(),
            value: String::new(),
        });
    }
}
