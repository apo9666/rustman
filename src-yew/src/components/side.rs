use gloo::events::EventListener;
use yew::prelude::*;

use wasm_bindgen::JsCast;

use crate::state::{TreeAction, TreeState};

#[derive(Properties, Clone, PartialEq)]
pub struct SideProps {
    pub on_add_server: Callback<()>,
    pub on_add_tag: Callback<()>,
}

#[function_component(Side)]
pub fn side(props: &SideProps) -> Html {
    let tree_state = use_context::<UseReducerHandle<TreeState>>();

    let Some(tree_state) = tree_state else {
        return html! {};
    };

    {
        let tree_state = tree_state.clone();
        let servers_len = tree_state.servers.len();
        let selected = tree_state.selected_server;
        use_effect_with((servers_len, selected), move |(len, selected)| {
            if *len > 0 && selected.is_none() {
                tree_state.dispatch(TreeAction::SetSelectedServer { index: 0 });
            }
            || ()
        });
    }

    let on_cancel_move = {
        let tree_state = tree_state.clone();
        Callback::from(move |_| {
            tree_state.dispatch(TreeAction::ClearPendingMove);
        })
    };

    let servers = tree_state.servers.clone();
    let selected_server = tree_state
        .selected_server
        .filter(|index| *index < servers.len())
        .or_else(|| if servers.is_empty() { None } else { Some(0) });

    let on_server_change = {
        let tree_state = tree_state.clone();
        Callback::from(move |event: Event| {
            let value = select_value(&event);
            if let Ok(index) = value.parse::<usize>() {
                tree_state.dispatch(TreeAction::SetSelectedServer { index });
            }
        })
    };

    let menu_open = use_state(|| false);
    let menu_ref = use_node_ref();
    let pending_remove_server = use_state(|| None::<usize>);
    let on_menu_toggle = {
        let menu_open = menu_open.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(!*menu_open);
        })
    };

    {
        let menu_open = menu_open.clone();
        let menu_ref = menu_ref.clone();
        use_effect_with(menu_open.clone(), move |is_open| {
            if !**is_open {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };
            let listener = EventListener::new(&document, "click", move |event| {
                let target = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
                let menu_node = menu_ref.cast::<web_sys::Node>();
                if let (Some(menu_node), Some(target)) = (menu_node, target) {
                    if menu_node.contains(Some(&target)) {
                        return;
                    }
                }
                menu_open.set(false);
            });
            Box::new(move || drop(listener)) as Box<dyn FnOnce()>
        });
    }

    let on_add_server = {
        let menu_open = menu_open.clone();
        let on_add_server = props.on_add_server.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            on_add_server.emit(());
        })
    };

    let on_add_tag = {
        let menu_open = menu_open.clone();
        let on_add_tag = props.on_add_tag.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            on_add_tag.emit(());
        })
    };

    let on_remove_server = {
        let tree_state = tree_state.clone();
        let menu_open = menu_open.clone();
        let selected_server = selected_server.clone();
        let pending_remove_server = pending_remove_server.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            let Some(index) = selected_server else {
                return;
            };
            if tree_state.servers.get(index).is_some() {
                pending_remove_server.set(Some(index));
            }
        })
    };

    html! {
        <>
        <nav class="tree-panel">
            <div class="tree-header">
                <div class="server-select-wrap">
                    <select
                        class="server-select"
                        onchange={on_server_change}
                        value={selected_server.map(|index| index.to_string()).unwrap_or_default()}
                        disabled={servers.is_empty()}
                    >
                        {
                            if servers.is_empty() {
                                html! { <option value="">{ "No servers" }</option> }
                            } else {
                                html! { for servers.iter().enumerate().map(|(index, server)| {
                                    html! { <option value={index.to_string()}>{ server.clone() }</option> }
                                }) }
                            }
                        }
                    </select>
                    <span class="select-chevron"></span>
                </div>
                <div class="tree-menu-wrap" ref={menu_ref}>
                    <button type="button" class="tree-row-menu" title="Servers" onclick={on_menu_toggle}>{ "⋯" }</button>
                    {
                        if *menu_open {
                            html! {
                                <div class="tree-menu">
                                    <button type="button" class="tree-menu-item" onclick={on_add_server.clone()}>
                                        { "Add server" }
                                    </button>
                                    <button type="button" class="tree-menu-item" onclick={on_add_tag.clone()}>
                                        { "Add tag" }
                                    </button>
                                    <button type="button" class="tree-menu-item danger" onclick={on_remove_server.clone()}>
                                        { "Remove server" }
                                    </button>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </div>
            {
                if let Some(pending_move) = tree_state.pending_move.as_ref() {
                    html! {
                        <div class="tree-banner">
                            <div class="tree-banner-text">
                                <span class="tree-banner-title">{ "Move:" }</span>
                                <span class="tree-banner-label">{ pending_move.label.clone() }</span>
                                <span class="muted">{ "Select destination tag." }</span>
                            </div>
                            <button class="tree-banner-cancel" type="button" onclick={on_cancel_move.clone()}>
                                { "Cancel" }
                            </button>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            <div class="tree">
                {
                    if tree_state.root.children.is_empty() {
                        html! { <div class="tree-empty">{ "Add a tag to get started." }</div> }
                    } else {
                        html! { <crate::components::tree::directory::TreeDirectory node={tree_state.root.clone()} path={vec![]} /> }
                    }
                }
            </div>
        </nav>
        {
            if let Some(index) = (*pending_remove_server).clone() {
                let label = tree_state.servers.get(index).cloned().unwrap_or_default();
                let on_confirm = {
                    let tree_state = tree_state.clone();
                    let pending_remove_server = pending_remove_server.clone();
                    Callback::from(move |_| {
                        tree_state.dispatch(TreeAction::RemoveServer { index });
                        pending_remove_server.set(None);
                    })
                };
                let on_cancel = {
                    let pending_remove_server = pending_remove_server.clone();
                    Callback::from(move |_| {
                        pending_remove_server.set(None);
                    })
                };
                let on_confirm_click =
                    Callback::from(move |_event: MouseEvent| on_confirm.emit(()));
                let on_cancel_click =
                    Callback::from(move |_event: MouseEvent| on_cancel.emit(()));

                html! {
                    <div class="modal-backdrop">
                        <div class="modal">
                            <h2 class="modal-title">{ "Remove server" }</h2>
                            <p class="modal-text">{ format!("Remove \"{}\"?", label) }</p>
                            <div class="modal-actions">
                                <button class="button secondary" onclick={on_cancel_click}>{ "Cancel" }</button>
                                <button class="button danger" onclick={on_confirm_click}>{ "Remove" }</button>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }
        }
        </>
    }
}

fn select_value(event: &Event) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}
