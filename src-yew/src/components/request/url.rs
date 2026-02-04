use std::collections::HashMap;

use gloo_net::http::{Request, RequestBuilder};
use http::Method;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::SubmitEvent;
use yew::prelude::*;

use crate::state::{Header, MethodEnum, Response, TabAction, TabContent, TabState};
use crate::utils::params_from_url;

#[derive(Properties, Clone, PartialEq)]
pub struct RequestUrlProps {
    pub tab_index: usize,
    pub content: TabContent,
}

#[function_component(RequestUrl)]
pub fn request_url(props: &RequestUrlProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let index = props.tab_index;
    let content = props.content.clone();

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
            if let Some(params) = params_from_url(&value) {
                tab_state.dispatch(TabAction::UpdateUrlAndParams {
                    index,
                    url: value,
                    params,
                });
            } else {
                tab_state.dispatch(TabAction::UpdateUrl { index, url: value });
            }
        })
    };

    let on_submit = {
        let tab_state = tab_state.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let tab_state = tab_state.clone();
            spawn_local(async move {
                let Some(tab) = tab_state.tabs.get(index).cloned() else {
                    return;
                };
                let response = match perform_request(&tab.content).await {
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
                    placeholder="Enter request URL"
                    value={content.url}
                    oninput={on_url_change}
                />
            </div>
            <button type="submit" class="button">{ "Send" }</button>
        </form>
    }
}

async fn perform_request(content: &TabContent) -> Result<Response, String> {
    let builder = request_builder(content.method, &content.url);
    let builder = apply_headers(builder, &content.headers);

    let response = if should_send_body(content.method) {
        let request = builder
            .body(content.body.clone())
            .map_err(|err| err.to_string())?;
        request.send().await.map_err(|err| err.to_string())?
    } else {
        builder.send().await.map_err(|err| err.to_string())?
    };

    let mut headers = HashMap::new();
    let mut raw_headers = HashMap::new();
    for (key, value) in response.headers().entries() {
        headers.insert(key.clone(), value.clone());
        raw_headers.insert(key, vec![value]);
    }

    let data = response.text().await.map_err(|err| err.to_string())?;

    Ok(Response {
        url: response.url(),
        status: response.status(),
        ok: response.ok(),
        headers,
        raw_headers,
        data,
    })
}

fn request_builder(method: MethodEnum, url: &str) -> RequestBuilder {
    match method {
        MethodEnum::Get => Request::get(url),
        MethodEnum::Post => Request::post(url),
        MethodEnum::Put => Request::put(url),
        MethodEnum::Patch => Request::patch(url),
        MethodEnum::Delete => Request::delete(url),
        MethodEnum::Options => RequestBuilder::new(url).method(Method::OPTIONS),
        MethodEnum::Head => RequestBuilder::new(url).method(Method::HEAD),
        MethodEnum::Trace => RequestBuilder::new(url).method(Method::TRACE),
    }
}

fn apply_headers(builder: RequestBuilder, headers: &[Header]) -> RequestBuilder {
    let mut builder = builder;
    for header in headers {
        if !header.enable {
            continue;
        }
        if header.key.trim().is_empty() {
            continue;
        }
        builder = builder.header(&header.key, &header.value);
    }
    builder
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
