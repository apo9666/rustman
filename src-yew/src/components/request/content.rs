use yew::prelude::*;

use crate::components::request::body::RequestBody;
use crate::components::request::header_table::HeaderTable;
use crate::components::request::param_table::ParamTable;
use crate::state::TabContent;

#[derive(Properties, Clone, PartialEq)]
pub struct RequestContentProps {
    pub tab_index: usize,
    pub content: TabContent,
}

#[function_component(RequestContent)]
pub fn request_content(props: &RequestContentProps) -> Html {
    let active = use_state(|| "params".to_string());
    let on_select = |value: &'static str, active: UseStateHandle<String>| {
        Callback::from(move |_| active.set(value.to_string()))
    };

    let tab_index = props.tab_index;
    let content = props.content.clone();

    html! {
        <div class="request">
            <div class="subtabs">
                <button
                    class={classes!("subtab", if *active == "params" { "active" } else { "" })}
                    onclick={on_select("params", active.clone())}
                >
                    { "Params" }
                </button>
                <button
                    class={classes!("subtab", if *active == "headers" { "active" } else { "" })}
                    onclick={on_select("headers", active.clone())}
                >
                    { "Headers" }
                </button>
                <button
                    class={classes!("subtab", if *active == "body" { "active" } else { "" })}
                    onclick={on_select("body", active.clone())}
                >
                    { "Body" }
                </button>
            </div>

            {
                match active.as_str() {
                    "headers" => html! { <HeaderTable tab_index={tab_index} headers={content.headers.clone()} /> },
                    "body" => html! {
                        <RequestBody
                            tab_index={tab_index}
                            body={content.body.clone()}
                            formatted={content.body_formatted}
                        />
                    },
                    _ => html! { <ParamTable tab_index={tab_index} url={content.url.clone()} params={content.params.clone()} /> },
                }
            }
        </div>
    }
}
