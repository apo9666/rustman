use std::collections::HashMap;

use js_sys::{Object, Reflect};
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::SubmitEvent;
use yew::prelude::*;

use url::Url;

use crate::state::{
    Header, MethodEnum, Param, Response, TabAction, TabContent, TabState, TreeState,
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
    let index = props.tab_index;
    let content = props.content.clone();
    let selected_server = tree_state
        .selected_server
        .and_then(|index| tree_state.servers.get(index))
        .cloned();

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
        let selected_server = selected_server.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let tab_state = tab_state.clone();
            let selected_server = selected_server.clone();
            spawn_local(async move {
                let Some(tab) = tab_state.tabs.get(index).cloned() else {
                    return;
                };
                let response =
                    match perform_request(&tab.content, selected_server.as_deref()).await {
                        Ok(response) => response,
                        Err(error) => Response {
                            data: error,
                        ok: false,
                        status: 0,
                        ..Response::default()
                    },
                };
                tab_state.dispatch(TabAction::SetResponse { index, response });
            });
        })
    };

    html! {
        <form class="form-row" onsubmit={on_submit}>
            <div class="request-url">
                <div class="select-wrap">
                    <select class="method-select" value={content.method.as_str().to_string()} onchange={on_method_change}>
                        { for MethodEnum::all().iter().map(|method| html! {
                            <option value={method.as_str().to_string()}>{ method.as_str() }</option>
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
            <button type="submit" class="button">{ "Send" }</button>
        </form>
    }
}

async fn perform_request(content: &TabContent, server: Option<&str>) -> Result<Response, String> {
    let url = build_request_url(content, server)?;
    let headers = build_headers(&content.headers, content.method, &content.body);
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

    let payload = build_request_payload(&request).map_err(|err| format!("{:?}", err))?;
    let value = tauri_api::invoke("send_request", payload)
        .await
        .map_err(|err| format!("{:?}", err))?;
    let response: Response =
        serde_wasm_bindgen::from_value(value).map_err(|err| err.to_string())?;
    Ok(response)
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

fn build_request_url(content: &TabContent, server: Option<&str>) -> Result<String, String> {
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
    let base = server.trim_end_matches('/');
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
