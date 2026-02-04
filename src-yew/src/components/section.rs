use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::components::request::content::RequestContent;
use crate::components::request::title::RequestTitle;
use crate::components::request::url::RequestUrl;
use crate::components::response::content::ResponseContent;
use crate::state::{TabAction, TabState};

#[derive(Properties, Clone, PartialEq)]
pub struct SectionProps {
    pub on_save: Callback<()>,
    pub on_add_server: Callback<()>,
}

#[function_component(Section)]
pub fn section(props: &SectionProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };

    let panel_ref = use_node_ref();
    let request_height = use_state(|| 320.0);
    let dragging = use_state(|| false);
    let drag_state = use_mut_ref(|| DragState::default());

    let tabs = tab_state.tabs.clone();
    let active = tab_state.active_tab_id;

    let on_add = {
        let tab_state = tab_state.clone();
        Callback::from(move |_| {
            tab_state.dispatch(TabAction::AddTab);
        })
    };

    let on_add_server = props.on_add_server.clone();
    let on_add_server_click = Callback::from(move |_event: MouseEvent| {
        on_add_server.emit(());
    });

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
                {
                    if tab.dirty {
                        html! { <span class="tab-icon" aria-hidden="true"></span> }
                    } else {
                        html! {}
                    }
                }
                <span>{ tab.label.clone() }</span>
                <span class="tab-close" onclick={on_close.clone()}>{ "x" }</span>
            </button>
        }
    });

    let active_tab = tabs.get(active).cloned();

    let on_resize_start = {
        let dragging = dragging.clone();
        let request_height = request_height.clone();
        let drag_state = drag_state.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            dragging.set(true);
            let mut state = drag_state.borrow_mut();
            state.start_y = event.client_y() as f64;
            state.start_height = *request_height;
        })
    };

    {
        let dragging = dragging.clone();
        let request_height = request_height.clone();
        let panel_ref = panel_ref.clone();
        let drag_state = drag_state.clone();
        use_effect_with(dragging.clone(), move |is_dragging| {
            if !**is_dragging {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }

            let window = web_sys::window().expect("window not available");
            let move_listener = EventListener::new(&window, "mousemove", move |event| {
                let event = event
                    .dyn_ref::<web_sys::MouseEvent>()
                    .expect("event should be a mouse event");
                let state = drag_state.borrow();
                let delta = event.client_y() as f64 - state.start_y;
                let mut next_height = state.start_height + delta;

                let container_height = panel_ref
                    .cast::<web_sys::Element>()
                    .map(|element| element.get_bounding_client_rect().height())
                    .unwrap_or(600.0);

                let min_request = 180.0;
                let min_response = 180.0;
                let max_request = (container_height - min_response).max(min_request);

                if next_height < min_request {
                    next_height = min_request;
                }
                if next_height > max_request {
                    next_height = max_request;
                }

                request_height.set(next_height);
            });

            let dragging = dragging.clone();
            let up_listener = EventListener::new(&window, "mouseup", move |_| {
                dragging.set(false);
            });

            Box::new(move || {
                drop(move_listener);
                drop(up_listener);
            }) as Box<dyn FnOnce()>
        });
    }

    html! {
        <div class="tabs">
            <div class="tab-list">
                { for triggers }
                <button class="tab-add" onclick={on_add} disabled={tabs.is_empty()}>{ "+" }</button>
            </div>
            {
                if let Some(tab) = active_tab {
                    let request_style = format!("height: {}px;", *request_height);
                    html! {
                        <div class="tab-panel" ref={panel_ref}>
                            <div class="request-pane" style={request_style}>
                                <RequestTitle title={tab.label.clone()} on_save={props.on_save.clone()} />
                                <RequestUrl tab_index={active} content={tab.content.clone()} />
                                <RequestContent tab_index={active} content={tab.content.clone()} />
                            </div>
                            <div class="resize-handle" onmousedown={on_resize_start}></div>
                            <ResponseContent
                                tab_index={active}
                                data={tab.content.response.data.clone()}
                                formatted={tab.content.response.formatted}
                            />
                        </div>
                    }
                } else {
                    html! {
                        <div class="tab-empty">
                            <div class="tab-empty-card">
                                <p class="tab-empty-text">{ "Add a server to start." }</p>
                                <button class="tab-server-add" onclick={on_add_server_click}>{ "Add server" }</button>
                            </div>
                        </div>
                    }
                }
            }
        </div>
    }
}

#[derive(Default)]
struct DragState {
    start_y: f64,
    start_height: f64,
}
