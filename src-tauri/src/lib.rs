use std::collections::HashMap;
use std::sync::Arc;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::webview::PageLoadEvent;
use tauri::Emitter;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .menu(|handle| {
            let open = MenuItem::with_id(handle, "open", "Open", true, Some("cmdOrControl+O"))?;
            let save = MenuItem::with_id(
                handle,
                "save",
                "Save",
                true,
                Some("cmdOrControl+S"),
            )?;
            let close = MenuItem::with_id(handle, "close", "Close", true, Some("cmdOrControl+Q"))?;
            let file_menu = Submenu::with_items(handle, "File", true, &[&open, &save, &close])?;
            let edit_menu = Submenu::with_items(
                handle,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(handle, None)?,
                    &PredefinedMenuItem::redo(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::cut(handle, None)?,
                    &PredefinedMenuItem::copy(handle, None)?,
                    &PredefinedMenuItem::paste(handle, None)?,
                    &PredefinedMenuItem::select_all(handle, None)?,
                ],
            )?;
            Menu::with_items(handle, &[&file_menu, &edit_menu])
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "save" => {
                let _ = app.emit("menu-event", "save-event");
            }
            "open" => {
                let _ = app.emit("menu-event", "open-event");
            }
            "close" => {
                app.exit(0);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![send_request, open_preview])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

#[derive(Debug, Serialize)]
struct HttpResponse {
    url: String,
    status: u16,
    ok: bool,
    headers: HashMap<String, String>,
    raw_headers: HashMap<String, Vec<String>>,
    data: String,
}

#[tauri::command]
async fn send_request(
    request: Option<HttpRequest>,
    method: Option<String>,
    url: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<HttpResponse, String> {
    let request = match request {
        Some(request) => request,
        None => {
            let method = method.ok_or_else(|| "missing request.method".to_string())?;
            let url = url.ok_or_else(|| "missing request.url".to_string())?;
            HttpRequest {
                method,
                url,
                headers: headers.unwrap_or_default(),
                body,
            }
        }
    };

    let method = Method::from_bytes(request.method.as_bytes())
        .map_err(|err| format!("invalid method: {err}"))?;

    let client = reqwest::Client::new();
    let mut builder = client.request(method, &request.url);

    for (key, value) in request.headers {
        builder = builder.header(key, value);
    }

    if let Some(body) = request.body {
        builder = builder.body(body);
    }

    let response = builder
        .send()
        .await
        .map_err(|err| format!("request failed: {err}"))?;

    let status = response.status();
    let url = response.url().to_string();

    let mut headers = HashMap::new();
    let mut raw_headers: HashMap<String, Vec<String>> = HashMap::new();
    for (name, value) in response.headers().iter() {
        let name = name.to_string();
        let value = value.to_str().unwrap_or("").to_string();
        headers.insert(name.clone(), value.clone());
        raw_headers.entry(name).or_default().push(value);
    }

    let data = response
        .text()
        .await
        .map_err(|err| format!("read response failed: {err}"))?;

    Ok(HttpResponse {
        url,
        status: status.as_u16(),
        ok: status.is_success(),
        headers,
        raw_headers,
        data,
    })
}

#[tauri::command]
fn open_preview(app: AppHandle, html: String) -> Result<(), String> {
    let encoded = STANDARD.encode(html);
    let script = format!(
        "(function(){{const html=atob('{encoded}');setTimeout(()=>{{document.open();document.write(html);document.close();}},0);}})();"
    );
    if let Some(window) = app.get_webview_window("preview") {
        let _ = window.eval(&script);
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let script = Arc::new(script);
    let script_for_load = script.clone();

    let window = WebviewWindowBuilder::new(&app, "preview", WebviewUrl::App("index.html".into()))
        .title("Preview")
        .on_page_load(move |window, payload| {
            if matches!(payload.event(), PageLoadEvent::Finished) {
                let _ = window.eval(script_for_load.as_str());
            }
        })
        .build()
        .map_err(|err| err.to_string())?;

    let _ = window.eval(script.as_str());

    Ok(())
}
