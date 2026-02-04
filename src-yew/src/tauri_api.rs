use js_sys::{
    encode_uri_component, Array, ArrayBuffer, Function, Object, Promise, Reflect, Uint8Array,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

fn tauri_root() -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not available"))?;
    Reflect::get(&window, &JsValue::from_str("__TAURI__"))
}

fn tauri_module(name: &str) -> Result<JsValue, JsValue> {
    let tauri = tauri_root()?;
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

fn js_value_to_text(value: JsValue) -> Result<String, JsValue> {
    if let Some(text) = value.as_string() {
        return Ok(text);
    }

    if value.is_instance_of::<Uint8Array>() {
        let bytes: Uint8Array = value.dyn_into()?;
        return decode_bytes(&bytes);
    }

    if value.is_instance_of::<ArrayBuffer>() {
        let buffer: ArrayBuffer = value.dyn_into()?;
        let bytes = Uint8Array::new(&buffer);
        return decode_bytes(&bytes);
    }

    if Array::is_array(&value) {
        let array = Array::from(&value);
        let bytes = Uint8Array::new_with_length(array.length());
        for (index, item) in array.iter().enumerate() {
            let byte = item.as_f64().unwrap_or_default() as u8;
            bytes.set_index(index as u32, byte);
        }
        return decode_bytes(&bytes);
    }

    Err(JsValue::from_str("Unsupported text payload"))
}

fn decode_bytes(bytes: &Uint8Array) -> Result<String, JsValue> {
    let decoder = web_sys::TextDecoder::new()?;
    let data = bytes.to_vec();
    decoder.decode_with_u8_array(&data)
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

pub async fn invoke(command: &str, payload: JsValue) -> Result<JsValue, JsValue> {
    invoke_with_options(command, payload, None).await
}

pub async fn invoke_with_options(
    command: &str,
    payload: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let tauri = tauri_root()?;
    let invoke_fn = resolve_invoke(&tauri)?;
    let promise = match options {
        Some(options) => invoke_fn
            .call3(&tauri, &JsValue::from_str(command), &payload, &options)?
            .dyn_into::<Promise>()?,
        None => invoke_fn
            .call2(&tauri, &JsValue::from_str(command), &payload)?
            .dyn_into::<Promise>()?,
    };
    JsFuture::from(promise).await
}

fn resolve_invoke(tauri: &JsValue) -> Result<Function, JsValue> {
    if let Ok(core) = Reflect::get(tauri, &JsValue::from_str("core")) {
        if !core.is_undefined() && !core.is_null() {
            if let Ok(invoke_fn) = Reflect::get(&core, &JsValue::from_str("invoke")) {
                if invoke_fn.is_function() {
                    return invoke_fn.dyn_into::<Function>();
                }
            }
        }
    }
    if let Ok(invoke_fn) = Reflect::get(tauri, &JsValue::from_str("invoke")) {
        if invoke_fn.is_function() {
            return invoke_fn.dyn_into::<Function>();
        }
    }
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not available"))?;
    let internals = Reflect::get(&window, &JsValue::from_str("__TAURI_INTERNALS__"))?;
    if let Ok(invoke_fn) = Reflect::get(&internals, &JsValue::from_str("invoke")) {
        if invoke_fn.is_function() {
            return invoke_fn.dyn_into::<Function>();
        }
    }
    Err(JsValue::from_str("invoke not available on __TAURI__"))
}

pub async fn dialog_open() -> Result<Option<String>, JsValue> {
    let payload = Object::new();
    let options = Object::new();
    Reflect::set(&payload, &JsValue::from_str("options"), &options)?;
    let value = invoke("plugin:dialog|open", payload.into()).await?;
    Ok(js_value_to_string(value))
}

pub async fn dialog_save() -> Result<Option<String>, JsValue> {
    let payload = Object::new();
    let options = Object::new();
    Reflect::set(&payload, &JsValue::from_str("options"), &options)?;
    let value = invoke("plugin:dialog|save", payload.into()).await?;
    Ok(js_value_to_string(value))
}

pub async fn fs_read_text(path: &str) -> Result<String, JsValue> {
    let payload = Object::new();
    Reflect::set(&payload, &JsValue::from_str("path"), &JsValue::from_str(path))?;
    let value = invoke("plugin:fs|read_text_file", payload.into()).await?;
    js_value_to_text(value)
}

pub async fn fs_write_text(path: &str, contents: &str) -> Result<(), JsValue> {
    let encoder = web_sys::TextEncoder::new()?;
    let bytes = encoder.encode_with_input(contents);
    let payload = Uint8Array::from(bytes.as_slice());
    let headers = Object::new();
    let encoded_path = encode_uri_component(path);
    Reflect::set(
        &headers,
        &JsValue::from_str("path"),
        &JsValue::from(encoded_path),
    )?;
    let invoke_options = Object::new();
    Reflect::set(&invoke_options, &JsValue::from_str("headers"), &headers)?;
    let _ = invoke_with_options(
        "plugin:fs|write_text_file",
        payload.into(),
        Some(invoke_options.into()),
    )
    .await?;
    Ok(())
}
