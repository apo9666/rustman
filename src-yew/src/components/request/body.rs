use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::components::json_highlight::{highlight_json, parse_json_value};
use crate::state::{TabAction, TabState};

#[derive(Properties, Clone, PartialEq)]
pub struct RequestBodyProps {
    pub tab_index: usize,
    pub body: String,
    pub formatted: bool,
}

#[function_component(RequestBody)]
pub fn request_body(props: &RequestBodyProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let index = props.tab_index;
    let body = props.body.clone();
    let formatted = props.formatted;

    let on_change = {
        let tab_state = tab_state.clone();
        Callback::from(move |event: InputEvent| {
            let value = event_target_value(&event);
            tab_state.dispatch(TabAction::UpdateBody { index, body: value });
        })
    };

    let on_format = {
        let tab_state = tab_state.clone();
        let body_for_format = body.clone();
        Callback::from(move |_| {
            if formatted {
                tab_state.dispatch(TabAction::SetBodyState {
                    index,
                    body: body_for_format.clone(),
                    formatted: false,
                });
                return;
            }

            let mut next_body = body_for_format.clone();
            if let Some(parsed) = parse_json_value(&body_for_format) {
                if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                    next_body = pretty;
                }
            }

            tab_state.dispatch(TabAction::SetBodyState {
                index,
                body: next_body,
                formatted: true,
            });
        })
    };

    html! {
            <div class="table-wrap body-wrap">
                <div class="request-title">
                    <h1>{ "Body" }</h1>
                    <button class="button secondary" onclick={on_format}>
                        { if formatted { "Edit" } else { "Format" } }
                    </button>
                </div>
                <hr class="section-divider" />
                <div class="body-editor-wrap">
                    {
                        if formatted {
                            if let Some(highlight) = highlight_json(&body) {
                                html! { <pre class="editor body-editor response-code"><code>{ highlight }</code></pre> }
                            } else {
                                html! { <pre class="editor body-editor response-code"><code>{ body.clone() }</code></pre> }
                            }
                        } else {
                            html! { <textarea class="editor body-editor" value={body.clone()} oninput={on_change} /> }
                        }
                    }
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
