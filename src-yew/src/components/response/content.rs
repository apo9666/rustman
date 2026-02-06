use std::collections::HashMap;

use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::json_highlight::{highlight_json, parse_json_value};
use crate::components::request::url::{authorization_header_value, build_request_debug};
use crate::state::TabAction;
use crate::state::{
    RequestDebugInfo, Response, ServerEntry, TabContent, TabState, TreeState,
};
use crate::tauri_api;

#[derive(Properties, Clone, PartialEq)]
pub struct ResponseContentProps {
    pub tab_index: usize,
    pub response: Response,
    pub request: TabContent,
}

#[function_component(ResponseContent)]
pub fn response_content(props: &ResponseContentProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let tree_state = use_context::<UseReducerHandle<TreeState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let Some(tree_state) = tree_state else {
        return html! {};
    };
    let response = props.response.clone();
    let request = props.request.clone();
    let data = response.data.clone();
    let index = props.tab_index;
    let formatted = response.formatted;
    let duration_ms = response.duration_ms;
    let debug_open = use_state(|| false);

    let on_format = {
        let tab_state = tab_state.clone();
        let data_for_format = data.clone();
        Callback::from(move |_| {
            let mut response = tab_state
                .tabs
                .get(index)
                .map(|tab| tab.content.response.clone())
                .unwrap_or_default();
            if let Some(parsed) = parse_json_value(&data_for_format) {
                if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                    response.data = pretty;
                }
            }
            response.formatted = true;
            tab_state.dispatch(TabAction::SetResponse { index, response });
        })
    };

    let on_debug = {
        let debug_open = debug_open.clone();
        Callback::from(move |_| {
            debug_open.set(true);
        })
    };

    let on_debug_close = {
        let debug_open = debug_open.clone();
        Callback::from(move |_| {
            debug_open.set(false);
        })
    };

    let selected_server = tree_state
        .selected_server
        .and_then(|index| tree_state.servers.get(index))
        .cloned();

    let (debug_request, debug_response) =
        format_debug_sections(&request, &response, selected_server.as_ref());

    let on_preview = {
        let data_for_preview = data.clone();
        Callback::from(move |_| {
            let html = data_for_preview.clone();
            spawn_local(async move {
                if html.trim().is_empty() {
                    return;
                }
                let payload = Object::new();
                if Reflect::set(&payload, &JsValue::from_str("html"), &JsValue::from_str(&html))
                    .is_err()
                {
                    return;
                }
                let _ = tauri_api::invoke("open_preview", payload.into()).await;
            });
        })
    };

    html! {
        <div class="response">
            <div class="request-title">
                <h1>{ "Response" }</h1>
                {
                    if let Some(duration) = duration_ms {
                        html! { <span class="response-meta">{ format!("{duration} ms") }</span> }
                    } else {
                        html! {}
                    }
                }
                <div class="request-actions">
                    <button class="button secondary" onclick={on_preview}>{ "Preview" }</button>
                    <button class="button secondary" onclick={on_debug}>{ "Debug" }</button>
                    <button class="button secondary" onclick={on_format}>{ "Format" }</button>
                </div>
            </div>
            <div class="response-body">
                {
                    if formatted {
                        if let Some(highlight) = highlight_json(&response.data) {
                            html! { <pre class="editor response-editor response-code"><code>{ highlight }</code></pre> }
                        } else {
                            html! { <pre class="editor response-editor response-code"><code>{ response.data.clone() }</code></pre> }
                        }
                    } else {
                        html! { <pre class="editor response-editor response-code"><code>{ response.data.clone() }</code></pre> }
                    }
                }
            </div>
            {
                if *debug_open {
                    let on_close_click = {
                        let on_debug_close = on_debug_close.clone();
                        Callback::from(move |_event: MouseEvent| on_debug_close.emit(()))
                    };
                    html! {
                        <div class="modal-backdrop">
                            <div class="modal debug-modal">
                            <h2 class="modal-title">{ "Request" }</h2>
                            <pre class="debug-text">{ debug_request.clone() }</pre>
                            <h2 class="modal-title">{ "Response" }</h2>
                                <pre class="debug-text">{ debug_response.clone() }</pre>
                                <div class="modal-actions">
                                    <button class="button secondary" onclick={on_close_click}>{ "Close" }</button>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

fn format_debug_sections(
    request: &TabContent,
    response: &Response,
    server: Option<&ServerEntry>,
) -> (String, String) {
    let mut request_text = String::new();
    let info = response
        .request
        .clone()
        .or_else(|| build_request_debug(request, server).ok());

    match info {
        Some(mut info) => {
            if let Some(server) = server {
                if let Some(value) = authorization_header_value(&server.auth) {
                    ensure_authorization(&mut info, value);
                }
            }

            request_text.push_str(&format!("Method: {}\n", info.method));
            request_text.push_str(&format!("URL: {}\n", info.url));
            request_text.push_str("Headers:\n");
            request_text.push_str(&format_headers_map(&info.headers));
            request_text.push_str("\nBody:\n");
            if let Some(body) = info.body {
                if body.trim().is_empty() {
                    request_text.push_str("(empty)\n");
                } else {
                    request_text.push_str(&body);
                    if !body.ends_with('\n') {
                        request_text.push('\n');
                    }
                }
            } else {
                request_text.push_str("(none)\n");
            }
        }
        None => {
            request_text.push_str("Request error: unable to build debug request.\n");
        }
    }

    let mut response_text = String::new();
    response_text.push_str(&format!("Status: {} (ok: {})\n", response.status, response.ok));
    if let Some(duration) = response.duration_ms {
        response_text.push_str(&format!("Duration: {} ms\n", duration));
    }
    response_text.push_str(&format!("URL: {}\n", response.url));
    response_text.push_str("Headers:\n");
    response_text.push_str(&format_headers_map(&response.headers));
    if !response.raw_headers.is_empty() {
        response_text.push_str("\nRaw Headers:\n");
        for (key, values) in response.raw_headers.iter() {
            response_text.push_str(&format!("  {}: {}\n", key, values.join(", ")));
        }
    }
    response_text.push_str("\nBody:\n");
    if response.data.trim().is_empty() {
        response_text.push_str("(empty)\n");
    } else {
        response_text.push_str(&response.data);
        if !response.data.ends_with('\n') {
            response_text.push('\n');
        }
    }

    (request_text, response_text)
}

fn find_header_key(headers: &HashMap<String, String>, key: &str) -> Option<String> {
    headers
        .keys()
        .find(|existing| existing.eq_ignore_ascii_case(key))
        .cloned()
}

fn ensure_authorization(info: &mut RequestDebugInfo, value: String) {
    if let Some(existing_key) = find_header_key(&info.headers, "authorization") {
        let should_replace = info
            .headers
            .get(&existing_key)
            .map(|current| current.trim().is_empty())
            .unwrap_or(true);
        if should_replace {
            info.headers.insert(existing_key, value);
        }
    } else {
        info.headers.insert("Authorization".to_string(), value);
    }
}

fn format_headers_map(headers: &HashMap<String, String>) -> String {
    if headers.is_empty() {
        return "  (none)\n".to_string();
    }
    let mut entries: Vec<_> = headers.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut text = String::new();
    for (key, value) in entries {
        text.push_str(&format!("  {}: {}\n", key, value));
    }
    text
}
