use std::rc::Rc;

use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::state::{TabAction, TabContent, TabState, TreeAction, TreeNode, TreeState};

#[derive(Properties, Clone, PartialEq)]
pub struct TreeDirectoryProps {
    pub node: TreeNode,
    #[prop_or_default]
    pub path: Vec<usize>,
}

fn render_children(node: &TreeNode, path: &Vec<usize>) -> Html {
    let mut entries: Vec<(usize, TreeNode)> =
        node.children.iter().cloned().enumerate().collect();
    entries.sort_by_key(|(index, child)| {
        let is_file = child.content.is_some();
        (is_file, child.label.to_lowercase(), *index)
    });
    html! {
        { for entries.into_iter().map(|(index, child)| {
            let mut child_path = path.clone();
            child_path.push(index);
            html! {
                <TreeDirectory
                    key={index.to_string()}
                    node={child.clone()}
                    path={child_path}
                />
            }
        }) }
    }
}

#[function_component(TreeDirectory)]
pub fn tree_directory(props: &TreeDirectoryProps) -> Html {
    let tab_state = use_context::<UseReducerHandle<TabState>>();
    let tree_state = use_context::<UseReducerHandle<TreeState>>();
    let Some(tab_state) = tab_state else {
        return html! {};
    };
    let Some(tree_state) = tree_state else {
        return html! {};
    };

    let has_children = !props.node.children.is_empty();
    let is_folder = props.node.content.is_none();
    let is_selected = tree_state
        .selected_path
        .as_ref()
        .map(|path| path == &props.path)
        .unwrap_or(false);
    let is_editing = use_state(|| false);
    let draft = use_state(|| props.node.label.clone());
    let menu_open = use_state(|| false);

    {
        let label = props.node.label.clone();
        let draft = draft.clone();
        use_effect_with(label, move |label| {
            draft.set(label.clone());
            || ()
        });
    }

    if props.node.label == "Root" {
        return html! { <>{ render_children(&props.node, &props.path) }</> };
    }

    let commit_rename = {
        let tree_state = tree_state.clone();
        let path = props.path.clone();
        let draft = draft.clone();
        let is_editing = is_editing.clone();
        Rc::new(move || {
            let label = draft.trim();
            if label.is_empty() {
                is_editing.set(false);
                return;
            }
            tree_state.dispatch(TreeAction::Rename {
                path: path.clone(),
                label: label.to_string(),
            });
            is_editing.set(false);
        })
    };

    let on_rename = {
        let is_editing = is_editing.clone();
        let draft = draft.clone();
        let label = props.node.label.clone();
        let tree_state = tree_state.clone();
        let path = props.path.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            draft.set(label.clone());
            is_editing.set(true);
            tree_state.dispatch(TreeAction::SetSelected { path: path.clone() });
        })
    };

    let on_draft_change = {
        let draft = draft.clone();
        Callback::from(move |event: InputEvent| {
            let value = event_target_value(&event);
            draft.set(value);
        })
    };

    let on_draft_blur = {
        let commit_rename = commit_rename.clone();
        Callback::from(move |_| {
            commit_rename();
        })
    };

    let on_draft_keydown = {
        let commit_rename = commit_rename.clone();
        let is_editing = is_editing.clone();
        Callback::from(move |event: KeyboardEvent| match event.key().as_str() {
            "Enter" => {
                event.prevent_default();
                commit_rename();
            }
            "Escape" => {
                event.prevent_default();
                is_editing.set(false);
            }
            _ => {}
        })
    };

    let on_menu_toggle = {
        let menu_open = menu_open.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(!*menu_open);
        })
    };

    let on_menu_edit = {
        let menu_open = menu_open.clone();
        let on_rename = on_rename.clone();
        Callback::from(move |event: MouseEvent| {
            menu_open.set(false);
            on_rename.emit(event);
        })
    };

    let on_menu_move = {
        let menu_open = menu_open.clone();
        let tree_state = tree_state.clone();
        let path = props.path.clone();
        let label = props.node.label.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            tree_state.dispatch(TreeAction::SetSelected { path: path.clone() });
            tree_state.dispatch(TreeAction::RequestMove {
                path: path.clone(),
                label: label.clone(),
            });
        })
    };

    let on_menu_delete = {
        let menu_open = menu_open.clone();
        let tree_state = tree_state.clone();
        let path = props.path.clone();
        let label = props.node.label.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            menu_open.set(false);
            tree_state.dispatch(TreeAction::RequestDelete {
                path: path.clone(),
                label: label.clone(),
            });
        })
    };

    if is_folder {
        let expanded = props.node.expanded;
        let pending_move = tree_state.pending_move.clone();
        let on_toggle = {
            let tree_state = tree_state.clone();
            let path = props.path.clone();
            Callback::from(move |_| {
                if let Some(pending_move) = pending_move.as_ref() {
                    if !expanded {
                        tree_state.dispatch(TreeAction::SetExpanded {
                            path: path.clone(),
                            open: true,
                        });
                        return;
                    }
                    if pending_move.path == path {
                        return;
                    }
                    tree_state.dispatch(TreeAction::MoveNode {
                        from: pending_move.path.clone(),
                        to: path.clone(),
                    });
                    return;
                }
                tree_state.dispatch(TreeAction::SetSelected { path: path.clone() });
                tree_state.dispatch(TreeAction::SetExpanded {
                    path: path.clone(),
                    open: !expanded,
                });
            })
        };

        return html! {
            <div>
                <div class={classes!("tree-row-wrap", if is_selected { "selected" } else { "" })}>
                    {
                        if *is_editing {
                            html! {
                                <div class={classes!("tree-row", "tree-row-edit", if is_selected { "selected" } else { "" })}>
                                    <span class={classes!("tree-caret", if expanded { "expanded" } else { "" })}></span>
                                    <input
                                        class="tree-rename-input"
                                        value={(*draft).clone()}
                                        oninput={on_draft_change}
                                        onblur={on_draft_blur}
                                        onkeydown={on_draft_keydown}
                                        autofocus=true
                                    />
                                </div>
                            }
                        } else {
                            html! {
                                <button type="button" class={classes!("tree-row", if is_selected { "selected" } else { "" })} onclick={on_toggle}>
                                    <span class={classes!("tree-caret", if expanded { "expanded" } else { "" })}></span>
                                    <span class="tree-label">{ props.node.label.clone() }</span>
                                </button>
                            }
                        }
                    }
                    {
                        if !*is_editing {
                            html! {
                                <div class="tree-row-actions">
                                    <div class="tree-menu-wrap">
                                        <button
                                            type="button"
                                            class="tree-row-menu"
                                            title="More actions"
                                            onclick={on_menu_toggle}
                                        >
                                            { "⋯" }
                                        </button>
                                        {
                                            if *menu_open {
                                                html! {
                                                    <div class="tree-menu">
                                                        <button type="button" class="tree-menu-item" onclick={on_menu_edit.clone()}>
                                                            { "Edit" }
                                                        </button>
                                                        <button type="button" class="tree-menu-item" onclick={on_menu_move.clone()}>
                                                            { "Move" }
                                                        </button>
                                                        <button type="button" class="tree-menu-item danger" onclick={on_menu_delete.clone()}>
                                                            { "Remove" }
                                                        </button>
                                                    </div>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
                {
                    if expanded && has_children {
                        html! { <div class="tree-children">{ render_children(&props.node, &props.path) }</div> }
                    } else {
                        html! {}
                    }
                }
            </div>
        };
    }

    let label = props.node.label.clone();
    let content = props.node.content.clone();
    let method = content.as_ref().map(|content| content.method);
    let method_letter = method
        .map(|value| value.as_str().chars().next().unwrap_or(' '))
        .unwrap_or(' ');
    let method_class = method.map(|value| format!("method-{}", value.key()));
    let on_click = {
        let tab_state = tab_state.clone();
        let tree_state = tree_state.clone();
        let path = props.path.clone();
        let pending_move = tree_state.pending_move.clone();
        Callback::from(move |_| {
            if pending_move.is_some() {
                tree_state.dispatch(TreeAction::SetSelected { path: path.clone() });
                return;
            }
            let Some(content) = content.as_ref() else {
                return;
            };
            tree_state.dispatch(TreeAction::SetSelected { path: path.clone() });
            let new_content = TabContent::from_node(content);
            tab_state.dispatch(TabAction::OpenTab {
                label: label.clone(),
                content: new_content,
            });
        })
    };

    html! {
        <div class={classes!("tree-row-wrap", if is_selected { "selected" } else { "" })}>
            {
                if *is_editing {
                    html! {
                        <div class={classes!("tree-row", "tree-row-edit", if is_selected { "selected" } else { "" })}>
                            <span class="tree-caret-placeholder"></span>
                            <input
                                class="tree-rename-input"
                                value={(*draft).clone()}
                                oninput={on_draft_change}
                                onblur={on_draft_blur}
                                onkeydown={on_draft_keydown}
                                autofocus=true
                            />
                        </div>
                    }
                } else {
                    html! {
                        <button
                            type="button"
                            class={classes!("tree-row", if is_selected { "selected" } else { "" })}
                            onclick={on_click}
                            title={props.node.label.clone()}
                        >
                            <span class={classes!("tree-tab-icon", method_class)} aria-hidden="true">
                                { method_letter }
                            </span>
                            <span class="tree-label">{ props.node.label.clone() }</span>
                        </button>
                    }
                }
            }
            {
                if !*is_editing {
                    html! {
                        <div class="tree-row-actions">
                            <div class="tree-menu-wrap">
                                <button
                                    type="button"
                                    class="tree-row-menu"
                                    title="More actions"
                                    onclick={on_menu_toggle}
                                >
                                    { "⋯" }
                                </button>
                                {
                                    if *menu_open {
                                        html! {
                                            <div class="tree-menu">
                                                <button type="button" class="tree-menu-item" onclick={on_menu_edit}>
                                                    { "Edit" }
                                                </button>
                                                <button type="button" class="tree-menu-item" onclick={on_menu_move}>
                                                    { "Move" }
                                                </button>
                                                <button type="button" class="tree-menu-item danger" onclick={on_menu_delete}>
                                                    { "Remove" }
                                                </button>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

fn event_target_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}
