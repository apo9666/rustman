use yew::prelude::*;

use crate::components::json_highlight::{highlight_json, parse_json_value};
use crate::state::TabAction;
use crate::state::TabState;

#[derive(Properties, Clone, PartialEq)]
pub struct ResponseContentProps {
    pub tab_index: usize,
    pub data: String,
    pub formatted: bool,
}

#[function_component(ResponseContent)]
pub fn response_content(props: &ResponseContentProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let data = props.data.clone();
    let index = props.tab_index;
    let formatted = props.formatted;

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

    html! {
        <div class="response">
            <div class="request-title">
                <h1>{ "Response" }</h1>
                <button class="button secondary" onclick={on_format}>{ "Format" }</button>
            </div>
            <div class="response-body">
                {
                    if formatted {
                        if let Some(highlight) = highlight_json(&data) {
                            html! { <pre class="editor response-editor response-code"><code>{ highlight }</code></pre> }
                        } else {
                            html! { <pre class="editor response-editor response-code"><code>{ data.clone() }</code></pre> }
                        }
                    } else {
                        html! { <pre class="editor response-editor response-code"><code>{ data.clone() }</code></pre> }
                    }
                }
            </div>
        </div>
    }
}
