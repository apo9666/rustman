use yew::prelude::*;

use crate::components::request::content::RequestContent;
use crate::components::request::title::RequestTitle;
use crate::components::request::url::RequestUrl;
use crate::components::response::content::ResponseContent;
use crate::state::{TabAction, TabState};

#[derive(Properties, Clone, PartialEq)]
pub struct SectionProps {
    pub on_save: Callback<()>,
}

#[function_component(Section)]
pub fn section(props: &SectionProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };

    let tabs = tab_state.tabs.clone();
    let active = tab_state.active_tab_id;

    let on_add = {
        let tab_state = tab_state.clone();
        Callback::from(move |_| {
            tab_state.dispatch(TabAction::AddTab);
        })
    };

    let triggers = tabs.iter().enumerate().map(|(index, tab)| {
        let is_active = index == active;
        let tab_state_for_select = tab_state.clone();
        let on_select = Callback::from(move |_| {
            tab_state_for_select.dispatch(TabAction::SetActive(index));
        });
        let tab_state_for_close = tab_state.clone();
        let on_close = Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            tab_state_for_close.dispatch(TabAction::CloseTab(index));
        });
        html! {
            <button
                class={classes!("tab-trigger", if is_active { "active" } else { "" })}
                onclick={on_select}
                title={tab.label.clone()}
            >
                <span>{ tab.label.clone() }</span>
                <span class="tab-close" onclick={on_close.clone()}>{ "x" }</span>
            </button>
        }
    });

    let active_tab = tabs.get(active).cloned();

    html! {
        <div class="tabs">
            <div class="tab-list">
                { for triggers }
                <button class="tab-add" onclick={on_add}>{ "+" }</button>
            </div>
            {
                if let Some(tab) = active_tab {
                    html! {
                        <div class="tab-panel">
                            <RequestTitle title={tab.label.clone()} on_save={props.on_save.clone()} />
                            <RequestUrl tab_index={active} content={tab.content.clone()} />
                            <RequestContent tab_index={active} content={tab.content.clone()} />
                            <ResponseContent tab_index={active} data={tab.content.response.data.clone()} />
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
