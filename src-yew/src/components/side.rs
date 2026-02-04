use yew::prelude::*;

use wasm_bindgen::JsCast;

use crate::components::tree::directory::TreeDirectory;
use crate::state::{TreeAction, TreeNode, TreeState};

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
        let servers_len = tree_state.root.children.len();
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

    let servers = tree_state.root.children.clone();
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
    let on_menu_toggle = {
        let menu_open = menu_open.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(!*menu_open);
        })
    };

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
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            let Some(index) = selected_server else {
                return;
            };
            let label = tree_state
                .root
                .children
                .get(index)
                .map(|node| node.label.clone())
                .unwrap_or_default();
            if label.is_empty() {
                return;
            }
            if !confirm(&format!("Remove server?\n{label}")) {
                return;
            }
            tree_state.dispatch(TreeAction::RemoveServer { index });
        })
    };

    html! {
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
                                    html! { <option value={index.to_string()}>{ server.label.clone() }</option> }
                                }) }
                            }
                        }
                    </select>
                    <span class="select-chevron"></span>
                </div>
                <div class="tree-menu-wrap">
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
                    if servers.is_empty() {
                        html! { <div class="tree-empty">{ "Add a server to get started." }</div> }
                    } else {
                        let mut tag_entries: Vec<(Vec<usize>, TreeNode)> = Vec::new();
                        for (server_index, server) in servers.iter().enumerate() {
                            for (tag_index, tag) in server.children.iter().cloned().enumerate() {
                                tag_entries.push((vec![server_index, tag_index], tag));
                            }
                        }
                        tag_entries.sort_by(|(path_a, node_a), (path_b, node_b)| {
                            let is_file_a = node_a.content.is_some();
                            let is_file_b = node_b.content.is_some();
                            let type_cmp = is_file_a.cmp(&is_file_b);
                            if type_cmp != std::cmp::Ordering::Equal {
                                return type_cmp;
                            }
                            let label_cmp = node_a.label.to_lowercase().cmp(&node_b.label.to_lowercase());
                            if label_cmp == std::cmp::Ordering::Equal {
                                path_a.cmp(path_b)
                            } else {
                                label_cmp
                            }
                        });

                        if tag_entries.is_empty() {
                            html! { <div class="tree-empty">{ "Add a tag to get started." }</div> }
                        } else {
                            html! { for tag_entries.into_iter().map(|(path, node)| {
                                html! { <TreeDirectory node={node} path={path} /> }
                            }) }
                        }
                    }
                }
            </div>
        </nav>
    }
}

fn confirm(message: &str) -> bool {
    web_sys::window()
        .and_then(|window| window.confirm_with_message(message).ok())
        .unwrap_or(false)
}

fn select_value(event: &Event) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}
