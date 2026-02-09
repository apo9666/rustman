use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use js_sys::{Date, Object, Reflect};
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::SubmitEvent;
use yew::prelude::*;

use url::Url;

use crate::state::{
    ApiKeyLocation, Header, MethodEnum, Param, RequestDebugInfo, Response, ServerAuth,
    ServerEntry, TabAction, TabContent, TabState, TreeAction, TreeState,
};
use crate::tauri_api;
use crate::utils::{params_from_url, path_params_from_url};

#[derive(Properties, Clone, PartialEq)]
pub struct RequestUrlProps {
    pub tab_index: usize,
    pub content: TabContent,
}

#[function_component(RequestUrl)]
pub fn request_url(props: &RequestUrlProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let tree_state = use_context::<UseReducerHandle<TreeState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let Some(tree_state) = tree_state else {
        return html! {};
    };
    let is_sending = use_state(|| false);
    let index = props.tab_index;
    let content = props.content.clone();
    let selected_server = tree_state
        .selected_server
        .and_then(|index| tree_state.servers.get(index))
        .cloned();
    let selected_server_index = tree_state.selected_server;

    let on_method_change = {
        let tab_state = tab_state.clone();
        Callback::from(move |event: Event| {
            let value = select_value(&event);
            if let Some(method) = MethodEnum::from_str(&value) {
                tab_state.dispatch(TabAction::UpdateMethod { index, method });
            }
        })
    };

    let on_url_change = {
        let tab_state = tab_state.clone();
        Callback::from(move |event: InputEvent| {
            let value = input_value(&event);
            let normalized = normalize_request_path(&value);
            let base_path = strip_query(&normalized);
            let base_path_clone = base_path.clone();
            let existing_path_params = tab_state
                .tabs
                .get(index)
                .map(|tab| tab.content.path_params.clone())
                .unwrap_or_else(|| vec![Param {
                    enable: true,
                    key: String::new(),
                    value: String::new(),
                }]);
            if let Some(params) = params_from_url(&normalized) {
                tab_state.dispatch(TabAction::UpdateUrlAndParams {
                    index,
                    url: base_path_clone,
                    params,
                });
                let next_path_params = path_params_from_url(&base_path, &existing_path_params);
                tab_state.dispatch(TabAction::UpdatePathParams {
                    index,
                    path_params: next_path_params,
                });
            } else {
                tab_state.dispatch(TabAction::UpdateUrl {
                    index,
                    url: base_path_clone,
                });
                let next_path_params = path_params_from_url(&base_path, &existing_path_params);
                tab_state.dispatch(TabAction::UpdatePathParams {
                    index,
                    path_params: next_path_params,
                });
            }
        })
    };

    let on_submit = {
        let tab_state = tab_state.clone();
        let tree_state = tree_state.clone();
        let selected_server = selected_server.clone();
        let selected_server_index = selected_server_index;
        let is_sending = is_sending.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if *is_sending {
                return;
            }
            is_sending.set(true);
            let tab_state = tab_state.clone();
            let tree_state = tree_state.clone();
            let selected_server = selected_server.clone();
            let selected_server_index = selected_server_index;
            let is_sending = is_sending.clone();
            spawn_local(async move {
                let Some(tab) = tab_state.tabs.get(index).cloned() else {
                    is_sending.set(false);
                    return;
                };
                let started_at = Date::now();
                let response =
                    match perform_request(&tab.content, selected_server.as_ref()).await {
                        Ok(mut response) => {
                            let request_info =
                                build_request_debug(&tab.content, selected_server.as_ref()).ok();
                            response.request = request_info;
                            response.duration_ms = Some(duration_ms(started_at));
                            response
                        }
                        Err(error) => {
                            let request_info =
                                build_request_debug(&tab.content, selected_server.as_ref()).ok();
                            Response {
                                data: error,
                                ok: false,
                                status: 0,
                                duration_ms: Some(duration_ms(started_at)),
                                request: request_info,
                                ..Response::default()
                            }
                        }
                    };
                if let Some(next_auth) = extract_bearer_auth_update(
                    selected_server.as_ref(),
                    &response.data,
                ) {
                    if let Some(server_index) = selected_server_index {
                        tree_state.dispatch(TreeAction::UpdateServerAuth {
                            index: server_index,
                            auth: next_auth,
                        });
                    }
                }

                tab_state.dispatch(TabAction::SetResponse { index, response });
                is_sending.set(false);
            });
        })
    };

    html! {
        <form class="form-row" onsubmit={on_submit}>
            <div class="request-url">
                <div class="select-wrap">
                    <select class="method-select" onchange={on_method_change}>
                        { for MethodEnum::all().iter().map(|method| {
                            let is_selected = *method == content.method;
                            html! {
                                <option value={method.as_str().to_string()} selected={is_selected}>
                                    { method.as_str() }
                                </option>
                            }
                        }) }
                    </select>
                    <span class="select-chevron"></span>
                </div>
                <input
                    class="url-input"
                    placeholder="/path"
                    value={content.url}
                    oninput={on_url_change}
                />
            </div>
            <button
                type="submit"
                class="button"
                disabled={*is_sending}
                aria-busy={if *is_sending { Some(String::from("true")) } else { None }}
            >
                { if *is_sending { "Sending..." } else { "Send" } }
            </button>
        </form>
    }
}

async fn perform_request(
    content: &TabContent,
    server: Option<&ServerEntry>,
) -> Result<Response, String> {
    let url = build_request_url(content, server)?;
    let mut headers = build_headers(&content.headers, content.method, &content.body);
    let url = if let Some(server) = server {
        apply_auth(&url, &mut headers, &server.auth)?
    } else {
        url
    };
    let request = TauriRequest {
        method: content.method.as_str().to_string(),
        url,
        headers,
        body: if should_send_body(content.method) {
            Some(content.body.clone())
        } else {
            None
        },
    };

    let payload = build_request_payload(&request)
        .map_err(|err| format_request_error(tauri_api::js_error_to_string(&err)))?;
    let value = tauri_api::invoke("send_request", payload)
        .await
        .map_err(|err| format_request_error(tauri_api::js_error_to_string(&err)))?;
    let response: Response =
        serde_wasm_bindgen::from_value(value).map_err(|err| format!("Resposta inválida: {err}"))?;
    Ok(response)
}

pub(crate) fn build_request_debug(
    content: &TabContent,
    server: Option<&ServerEntry>,
) -> Result<RequestDebugInfo, String> {
    let url = build_request_url(content, server)?;
    let mut headers = build_headers(&content.headers, content.method, &content.body);
    let url = if let Some(server) = server {
        apply_auth(&url, &mut headers, &server.auth)?
    } else {
        url
    };

    Ok(RequestDebugInfo {
        method: content.method.as_str().to_string(),
        url,
        headers,
        body: if content.body.trim().is_empty() {
            None
        } else {
            Some(content.body.clone())
        },
    })
}

pub(crate) fn authorization_header_value(auth: &ServerAuth) -> Option<String> {
    match auth {
        ServerAuth::HttpBasic { username, password } => {
            let token = STANDARD.encode(format!("{username}:{password}"));
            Some(format!("Basic {token}"))
        }
        ServerAuth::HttpBearer {
            token,
            bearer_format,
            ..
        } => {
            if token.trim().is_empty() {
                None
            } else {
                let scheme = if bearer_format.trim().is_empty() {
                    "Bearer"
                } else {
                    bearer_format.trim()
                };
                Some(format!("{scheme} {token}"))
            }
        }
        ServerAuth::OAuth2 { access_token, .. } => {
            if access_token.trim().is_empty() {
                None
            } else {
                Some(format!("Bearer {access_token}"))
            }
        }
        ServerAuth::OpenIdConnect { access_token, .. } => {
            if access_token.trim().is_empty() {
                None
            } else {
                Some(format!("Bearer {access_token}"))
            }
        }
        _ => None,
    }
}

fn build_headers(headers: &[Header], method: MethodEnum, body: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut has_accept = false;
    let mut has_content_type = false;

    for header in headers {
        if !header.enable {
            continue;
        }
        if header.key.trim().is_empty() {
            continue;
        }
        let key = header.key.trim();
        if key.eq_ignore_ascii_case("accept") {
            has_accept = true;
        }
        if key.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        map.insert(key.to_string(), header.value.clone());
    }

    if !has_accept {
        map.insert("Accept".to_string(), "*/*".to_string());
    }
    if !has_content_type && should_send_body(method) && !body.trim().is_empty() {
        map.insert("Content-Type".to_string(), "application/json".to_string());
    }

    map
}

fn should_send_body(method: MethodEnum) -> bool {
    matches!(method, MethodEnum::Post | MethodEnum::Put | MethodEnum::Patch)
}

fn input_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn select_value(event: &Event) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

#[derive(Debug, Serialize)]
struct TauriRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

fn build_request_payload(request: &TauriRequest) -> Result<JsValue, JsValue> {
    let payload = Object::new();
    let request_obj = Object::new();
    let headers_obj = Object::new();

    Reflect::set(
        &request_obj,
        &JsValue::from_str("method"),
        &JsValue::from_str(&request.method),
    )?;
    Reflect::set(
        &request_obj,
        &JsValue::from_str("url"),
        &JsValue::from_str(&request.url),
    )?;

    for (key, value) in &request.headers {
        Reflect::set(
            &headers_obj,
            &JsValue::from_str(key),
            &JsValue::from_str(value),
        )?;
    }
    Reflect::set(
        &request_obj,
        &JsValue::from_str("headers"),
        &headers_obj,
    )?;

    let body_value = request
        .body
        .as_ref()
        .map(|value| JsValue::from_str(value))
        .unwrap_or(JsValue::NULL);
    Reflect::set(
        &request_obj,
        &JsValue::from_str("body"),
        &body_value,
    )?;

    Reflect::set(
        &payload,
        &JsValue::from_str("request"),
        &request_obj,
    )?;

    Ok(payload.into())
}

fn format_request_error(message: String) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return "Erro desconhecido.".to_string();
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("typeerror: load failed")
        || lower.contains("failed to fetch")
        || lower.contains("networkerror")
    {
        return "Falha ao conectar ao servidor.".to_string();
    }
    if lower.contains("invalid args `request`")
        || lower.contains("missing required key request")
        || lower.contains("command send_request")
    {
        return "Falha ao enviar a requisição.".to_string();
    }
    trimmed.to_string()
}

fn duration_ms(started_at: f64) -> u64 {
    let elapsed = Date::now() - started_at;
    if elapsed.is_finite() && elapsed > 0.0 {
        elapsed.round() as u64
    } else {
        0
    }
}

fn build_request_url(content: &TabContent, server: Option<&ServerEntry>) -> Result<String, String> {
    let path = normalize_request_path(&content.url);
    let path_with_values = apply_path_params(&path, &content.path_params);
    if path.is_empty() {
        return Err("Path vazio.".to_string());
    }

    let path_with_query = apply_params(&path_with_values, &content.params);

    if let Ok(url) = Url::parse(&path_with_query) {
        if matches!(url.scheme(), "http" | "https") {
            return Ok(path_with_query);
        }
    }

    let Some(server) = server else {
        return Err("Selecione um server.".to_string());
    };
    let base = server.url.trim_end_matches('/');
    let path = if path.starts_with('/') {
        path_with_query
    } else {
        format!("/{}", path_with_query)
    };
    Ok(format!("{base}{path}"))
}

fn normalize_request_path(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Ok(url) = Url::parse(trimmed) {
        if matches!(url.scheme(), "http" | "https") {
            let mut path = url.path().to_string();
            if let Some(query) = url.query() {
                path.push('?');
                path.push_str(query);
            }
            return normalize_slash_path(&path);
        }
    }

    normalize_slash_path(trimmed)
}

fn normalize_slash_path(value: &str) -> String {
    if value.starts_with('/') || value.starts_with('?') {
        value.to_string()
    } else {
        format!("/{}", value)
    }
}

fn strip_query(value: &str) -> String {
    value.split('?').next().unwrap_or("").to_string()
}

fn apply_params(base: &str, params: &[Param]) -> String {
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
    let base = strip_query(base);
    if query.is_empty() {
        base
    } else {
        format!("{}?{}", base, query)
    }
}

fn apply_path_params(base: &str, params: &[Param]) -> String {
    let mut values = std::collections::HashMap::new();
    for param in params {
        if !param.enable {
            continue;
        }
        let key = param.key.trim();
        if key.is_empty() {
            continue;
        }
        let value = param.value.trim();
        if value.is_empty() {
            continue;
        }
        let encoded: String = url::form_urlencoded::byte_serialize(value.as_bytes()).collect();
        values.insert(key.to_string(), encoded);
    }

    let mut result = String::new();
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
            if let Some(value) = values.get(trimmed) {
                result.push_str(value);
            } else {
                result.push('{');
                result.push_str(&key);
                result.push('}');
            }
            continue;
        }
        result.push(ch);
    }
    result
}

fn apply_auth(
    url: &str,
    headers: &mut HashMap<String, String>,
    auth: &ServerAuth,
) -> Result<String, String> {
    match auth {
        ServerAuth::None => Ok(url.to_string()),
        ServerAuth::ApiKey {
            name,
            location,
            value,
        } => {
            if name.trim().is_empty() || value.trim().is_empty() {
                return Ok(url.to_string());
            }
            match location {
                ApiKeyLocation::Header => {
                    set_header(headers, name, value.clone());
                    Ok(url.to_string())
                }
                ApiKeyLocation::Query => append_query_param(url, name, value),
                ApiKeyLocation::Cookie => {
                    append_cookie(headers, name, value);
                    Ok(url.to_string())
                }
            }
        }
        ServerAuth::HttpBasic { username, password } => {
            let token = STANDARD.encode(format!("{username}:{password}"));
            set_header(headers, "Authorization", format!("Basic {token}"));
            Ok(url.to_string())
        }
        ServerAuth::HttpBearer {
            token,
            bearer_format,
            ..
        } => {
            if !token.trim().is_empty() {
                let scheme = if bearer_format.trim().is_empty() {
                    "Bearer"
                } else {
                    bearer_format.trim()
                };
                set_header(headers, "Authorization", format!("{scheme} {token}"));
            }
            Ok(url.to_string())
        }
        ServerAuth::OAuth2 { access_token, .. } => {
            if !access_token.trim().is_empty() {
                set_header(headers, "Authorization", format!("Bearer {access_token}"));
            }
            Ok(url.to_string())
        }
        ServerAuth::OpenIdConnect { access_token, .. } => {
            if !access_token.trim().is_empty() {
                set_header(headers, "Authorization", format!("Bearer {access_token}"));
            }
            Ok(url.to_string())
        }
    }
}

fn extract_bearer_auth_update(
    server: Option<&ServerEntry>,
    response_body: &str,
) -> Option<ServerAuth> {
    let server = server?;
    let ServerAuth::HttpBearer {
        token,
        bearer_format,
        auto_update,
        token_path,
    } = &server.auth
    else {
        return None;
    };

    if !*auto_update {
        return None;
    }

    let next_token = extract_json_token(response_body, token_path)?;
    if next_token.trim().is_empty() || next_token == *token {
        return None;
    }

    Some(ServerAuth::HttpBearer {
        token: next_token,
        bearer_format: bearer_format.clone(),
        auto_update: *auto_update,
        token_path: token_path.clone(),
    })
}

fn extract_json_token(body: &str, path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let value: Value = serde_json::from_str(body).ok()?;
    let parts = parse_token_path(trimmed);
    if parts.is_empty() {
        return None;
    }
    let mut current = &value;
    for part in parts {
        if let Some(array) = current.as_array() {
            if let Ok(index) = part.parse::<usize>() {
                current = array.get(index)?;
                continue;
            }
        }
        let Some(obj) = current.as_object() else {
            return None;
        };
        current = obj.get(part)?;
    }

    match current {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn parse_token_path(path: &str) -> Vec<&str> {
    let trimmed = path.trim();
    let trimmed = trimmed.strip_prefix("$.").unwrap_or(trimmed);
    let trimmed = trimmed.strip_prefix('$').unwrap_or(trimmed);
    trimmed
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

fn append_query_param(url: &str, name: &str, value: &str) -> Result<String, String> {
    let mut parsed = Url::parse(url).map_err(|_| "URL inválida.".to_string())?;
    parsed.query_pairs_mut().append_pair(name, value);
    Ok(parsed.to_string())
}

fn set_header(headers: &mut HashMap<String, String>, key: &str, value: String) {
    if let Some(existing) = headers
        .keys()
        .find(|existing| existing.eq_ignore_ascii_case(key))
        .cloned()
    {
        headers.insert(existing, value);
    } else {
        headers.insert(key.to_string(), value);
    }
}

fn append_cookie(headers: &mut HashMap<String, String>, name: &str, value: &str) {
    let pair = format!("{name}={value}");
    if let Some(existing_key) = headers
        .keys()
        .find(|existing| existing.eq_ignore_ascii_case("cookie"))
        .cloned()
    {
        let next = match headers.get(&existing_key) {
            Some(current) if !current.trim().is_empty() => format!("{current}; {pair}"),
            _ => pair,
        };
        headers.insert(existing_key, next);
    } else {
        headers.insert("Cookie".to_string(), pair);
    }
}
