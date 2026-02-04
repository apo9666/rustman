use js_sys::{Array, Function, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

fn tauri_module(name: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not available"))?;
    let tauri = Reflect::get(&window, &JsValue::from_str("__TAURI__"))?;
    Reflect::get(&tauri, &JsValue::from_str(name))
}

fn js_value_to_string(value: JsValue) -> Option<String> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    if Array::is_array(&value) {
        let array = Array::from(&value);
        return array.get(0).as_string();
    }
    value.as_string()
}

pub fn event_listen(event: &str, handler: &JsValue) -> Result<(), JsValue> {
    let event_module = tauri_module("event")?;
    let listen_fn = Reflect::get(&event_module, &JsValue::from_str("listen"))?
        .dyn_into::<Function>()?;
    let _ = listen_fn.call2(
        &event_module,
        &JsValue::from_str(event),
        handler,
    )?;
    Ok(())
}

pub async fn dialog_open() -> Result<Option<String>, JsValue> {
    let dialog = tauri_module("dialog")?;
    let open_fn = Reflect::get(&dialog, &JsValue::from_str("open"))?
        .dyn_into::<Function>()?;
    let promise = open_fn.call0(&dialog)?.dyn_into::<Promise>()?;
    let value = JsFuture::from(promise).await?;
    Ok(js_value_to_string(value))
}

pub async fn dialog_save() -> Result<Option<String>, JsValue> {
    let dialog = tauri_module("dialog")?;
    let save_fn = Reflect::get(&dialog, &JsValue::from_str("save"))?
        .dyn_into::<Function>()?;
    let promise = save_fn.call0(&dialog)?.dyn_into::<Promise>()?;
    let value = JsFuture::from(promise).await?;
    Ok(js_value_to_string(value))
}

pub async fn fs_read_text(path: &str) -> Result<String, JsValue> {
    let fs = tauri_module("fs")?;
    let read_fn = Reflect::get(&fs, &JsValue::from_str("readTextFile"))?
        .dyn_into::<Function>()?;
    let promise = read_fn.call1(&fs, &JsValue::from_str(path))?
        .dyn_into::<Promise>()?;
    let value = JsFuture::from(promise).await?;
    Ok(value.as_string().unwrap_or_default())
}

pub async fn fs_write_text(path: &str, contents: &str) -> Result<(), JsValue> {
    let fs = tauri_module("fs")?;
    let write_fn = Reflect::get(&fs, &JsValue::from_str("writeTextFile"))?
        .dyn_into::<Function>()?;
    let promise = write_fn
        .call2(&fs, &JsValue::from_str(path), &JsValue::from_str(contents))?
        .dyn_into::<Promise>()?;
    let _ = JsFuture::from(promise).await?;
    Ok(())
}
