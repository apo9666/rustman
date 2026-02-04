use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::state::{TabAction, TabState};

#[derive(Properties, Clone, PartialEq)]
pub struct RequestBodyProps {
    pub tab_index: usize,
    pub body: String,
}

#[function_component(RequestBody)]
pub fn request_body(props: &RequestBodyProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let index = props.tab_index;
    let body = props.body.clone();

    let on_change = {
        let tab_state = tab_state.clone();
        Callback::from(move |event: InputEvent| {
            let value = event_target_value(&event);
            tab_state.dispatch(TabAction::UpdateBody { index, body: value });
        })
    };

    let on_format = {
        let tab_state = tab_state.clone();
        Callback::from(move |_| {
            let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) else {
                return;
            };
            let Ok(pretty) = serde_json::to_string_pretty(&parsed) else {
                return;
            };
            tab_state.dispatch(TabAction::UpdateBody { index, body: pretty });
        })
    };

    html! {
        <div class="table-wrap">
            <h2 class="table-title">{ "Body" }</h2>
            <div class="form-row">
                <button class="button secondary" onclick={on_format}>{ "Format" }</button>
            </div>
            <div class="form-row">
                <textarea class="editor" value={props.body.clone()} oninput={on_change} />
            </div>
        </div>
    }
}

fn event_target_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}
