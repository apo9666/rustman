use yew::prelude::*;

use crate::state::TabAction;
use crate::state::TabState;

#[derive(Properties, Clone, PartialEq)]
pub struct ResponseContentProps {
    pub tab_index: usize,
    pub data: String,
}

#[function_component(ResponseContent)]
pub fn response_content(props: &ResponseContentProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let data = props.data.clone();
    let index = props.tab_index;

    let on_format = {
        let tab_state = tab_state.clone();
        Callback::from(move |_| {
            let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) else {
                return;
            };
            let Ok(pretty) = serde_json::to_string_pretty(&parsed) else {
                return;
            };
            let mut response = tab_state
                .tabs
                .get(index)
                .map(|tab| tab.content.response.clone())
                .unwrap_or_default();
            response.data = pretty;
            tab_state.dispatch(TabAction::SetResponse { index, response });
        })
    };

    html! {
        <div class="response">
            <div class="request-title">
                <h1>{ "Response" }</h1>
                <button class="button secondary" onclick={on_format}>{ "Format" }</button>
            </div>
            <div class="form-row">
                <textarea class="editor" readonly=true value={props.data.clone()} />
            </div>
        </div>
    }
}
