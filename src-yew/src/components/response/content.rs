use std::collections::HashMap;

use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use gloo::events::EventListener;
use http::StatusCode;
use js_sys::{Object, Reflect};
use wasm_bindgen::JsCast;
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
    let status_label = status_label(response.status);
    let debug_open = use_state(|| false);
    let debug_ref = use_node_ref();
    let drag_vert = use_state(|| false);
    let drag_horiz = use_state(|| false);
    let split_width = use_state(|| 360.0);
    let split_height = use_state(|| 320.0);
    let drag_state = use_mut_ref(|| DebugDragState::default());

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

    let on_drag_vert_start = {
        let drag_vert = drag_vert.clone();
        let drag_state = drag_state.clone();
        let split_width = split_width.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            drag_vert.set(true);
            let mut state = drag_state.borrow_mut();
            state.start_x = event.client_x() as f64;
            state.start_width = *split_width;
        })
    };

    let on_drag_horiz_start = {
        let drag_horiz = drag_horiz.clone();
        let drag_state = drag_state.clone();
        let split_height = split_height.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            drag_horiz.set(true);
            let mut state = drag_state.borrow_mut();
            state.start_y = event.client_y() as f64;
            state.start_height = *split_height;
        })
    };

    {
        let drag_vert = drag_vert.clone();
        let drag_state = drag_state.clone();
        let split_width = split_width.clone();
        let debug_ref = debug_ref.clone();
        use_effect_with(drag_vert.clone(), move |is_dragging| {
            if !**is_dragging {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }

            let window = web_sys::window().expect("window not available");
            let move_listener = EventListener::new(&window, "mousemove", move |event| {
                let event = event
                    .dyn_ref::<web_sys::MouseEvent>()
                    .expect("event should be a mouse event");
                let state = drag_state.borrow();
                let delta = event.client_x() as f64 - state.start_x;
                let mut next_width = state.start_width + delta;

                if let Some(container) = debug_ref.cast::<web_sys::HtmlElement>() {
                    let rect = container.get_bounding_client_rect();
                    let max_width = (rect.width() - 200.0).max(200.0);
                    next_width = next_width.clamp(200.0, max_width);
                } else {
                    next_width = next_width.max(200.0);
                }

                split_width.set(next_width);
            });

            let drag_vert = drag_vert.clone();
            let up_listener = EventListener::new(&window, "mouseup", move |_| {
                drag_vert.set(false);
            });

            Box::new(move || {
                drop(move_listener);
                drop(up_listener);
            }) as Box<dyn FnOnce()>
        });
    }

    {
        let drag_horiz = drag_horiz.clone();
        let drag_state = drag_state.clone();
        let split_height = split_height.clone();
        let debug_ref = debug_ref.clone();
        use_effect_with(drag_horiz.clone(), move |is_dragging| {
            if !**is_dragging {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }

            let window = web_sys::window().expect("window not available");
            let move_listener = EventListener::new(&window, "mousemove", move |event| {
                let event = event
                    .dyn_ref::<web_sys::MouseEvent>()
                    .expect("event should be a mouse event");
                let state = drag_state.borrow();
                let delta = event.client_y() as f64 - state.start_y;
                let mut next_height = state.start_height + delta;

                if let Some(container) = debug_ref.cast::<web_sys::HtmlElement>() {
                    let rect = container.get_bounding_client_rect();
                    let max_height = (rect.height() - 160.0).max(160.0);
                    next_height = next_height.clamp(160.0, max_height);
                } else {
                    next_height = next_height.max(160.0);
                }

                split_height.set(next_height);
            });

            let drag_horiz = drag_horiz.clone();
            let up_listener = EventListener::new(&window, "mouseup", move |_| {
                drag_horiz.set(false);
            });

            Box::new(move || {
                drop(move_listener);
                drop(up_listener);
            }) as Box<dyn FnOnce()>
        });
    }

    let selected_server = tree_state
        .selected_server
        .and_then(|index| tree_state.servers.get(index))
        .cloned();

    let (debug_request, debug_response, debug_jwt) =
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

    let on_tools = Callback::from(move |_| {
        spawn_local(async move {
            let payload = Object::new();
            let _ = tauri_api::invoke("open_tools", payload.into()).await;
        });
    });

    html! {
        <div class="response">
            <div class="request-title">
                <h1>{ "Response" }</h1>
                {
                    if let Some(meta) = build_response_meta(status_label.as_ref(), duration_ms) {
                        html! { <span class="response-meta">{ meta }</span> }
                    } else {
                        html! {}
                    }
                }
                <div class="request-actions">
                    <button class="button secondary" onclick={on_preview}>{ "Preview" }</button>
                    <button class="button secondary" onclick={on_debug}>{ "Debug" }</button>
                    <button class="button secondary" onclick={on_tools}>{ "Tools" }</button>
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
                                <div class="debug-layout" ref={debug_ref}>
                                    <div class="debug-top" style={format!("height: {}px;", *split_height)}>
                                        <div class="debug-grid" style={format!("--debug-left-width: {}px;", *split_width)}>
                                            <div class="debug-title debug-title-left">
                                                <h2 class="modal-title">{ "Request" }</h2>
                                            </div>
                                            <div class="debug-title debug-title-right">
                                                <h2 class="modal-title">{ "JWT" }</h2>
                                            </div>
                                            <div class="debug-panel debug-panel-left">
                                                <pre class="debug-text">{ debug_request.clone() }</pre>
                                            </div>
                                            <div class="debug-resize-vert" onmousedown={on_drag_vert_start}></div>
                                            <div class="debug-panel debug-panel-right">
                                                <pre class="debug-text">{ debug_jwt.clone() }</pre>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="debug-resize-horiz" onmousedown={on_drag_horiz_start}></div>
                                    <div class="debug-bottom">
                                        <h2 class="modal-title">{ "Response" }</h2>
                                        <pre class="debug-text">{ debug_response.clone() }</pre>
                                    </div>
                                </div>
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
) -> (String, String, String) {
    let mut request_text = String::new();
    let mut info = response
        .request
        .clone()
        .or_else(|| build_request_debug(request, server).ok());

    if let Some(info) = info.as_mut() {
        if let Some(server) = server {
            if let Some(value) = authorization_header_value(&server.auth) {
                ensure_authorization(info, value);
            }
        }
    }

    match info.as_ref() {
        Some(info) => {
            request_text.push_str(&format!("Method: {}\n", info.method));
            request_text.push_str(&format!("URL: {}\n", info.url));
            request_text.push_str("Headers:\n");
            request_text.push_str(&format_headers_map(&info.headers));
            request_text.push_str("\nBody:\n");
            if let Some(body) = info.body.as_ref() {
                if body.trim().is_empty() {
                    request_text.push_str("(empty)\n");
                } else {
                    request_text.push_str(body);
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

    let jwt_text = match info.as_ref() {
        Some(info) => build_jwt_debug(&info.headers),
        None => "No request available.".to_string(),
    };

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

    (request_text, response_text, jwt_text)
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

#[derive(Default)]
struct DebugDragState {
    start_x: f64,
    start_y: f64,
    start_width: f64,
    start_height: f64,
}

fn build_jwt_debug(headers: &HashMap<String, String>) -> String {
    let Some(token) = extract_bearer_token(headers) else {
        return "No bearer token found in Authorization header.".to_string();
    };

    match decode_jwt(&token) {
        Ok((header, payload)) => {
            let mut text = String::new();
            text.push_str("Header:\n");
            text.push_str(&header);
            if !header.ends_with('\n') {
                text.push('\n');
            }
            text.push_str("\nPayload:\n");
            text.push_str(&payload);
            if !payload.ends_with('\n') {
                text.push('\n');
            }
            text
        }
        Err(err) => format!("JWT decode failed: {err}\n"),
    }
}

fn extract_bearer_token(headers: &HashMap<String, String>) -> Option<String> {
    let key = find_header_key(headers, "authorization")?;
    let value = headers.get(&key)?.trim();
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = parts.next().unwrap_or("").trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn decode_jwt(token: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return Err("Token does not look like a JWT.".to_string());
    }

    let header = decode_base64url(parts[0])?;
    let payload = decode_base64url(parts[1])?;
    Ok((format_json_or_text(&header), format_json_or_text(&payload)))
}

fn decode_base64url(value: &str) -> Result<String, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .map_err(|_| "Invalid base64url content.".to_string())?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn format_json_or_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "(empty)".to_string();
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Ok(pretty) = serde_json::to_string_pretty(&value) {
            return pretty;
        }
    }
    trimmed.to_string()
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

fn status_label(status: u16) -> Option<String> {
    if status == 0 {
        return None;
    }
    let label = StatusCode::from_u16(status)
        .ok()
        .and_then(|code| code.canonical_reason().map(|reason| (code, reason)))
        .map(|(code, reason)| format!("{} {}", code.as_u16(), reason))
        .unwrap_or_else(|| status.to_string());
    Some(label)
}

fn build_response_meta(status: Option<&String>, duration_ms: Option<u64>) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(status) = status {
        parts.push(status.clone());
    }
    if let Some(duration) = duration_ms {
        parts.push(format!("{duration} ms"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}
