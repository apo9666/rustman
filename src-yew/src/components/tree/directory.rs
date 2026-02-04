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
    html! {
        { for node.children.iter().enumerate().map(|(index, child)| {
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
    let is_editing = use_state(|| false);
    let draft = use_state(|| props.node.label.clone());

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

    if is_folder {
        let expanded = props.node.expanded;
        let on_toggle = {
            let tree_state = tree_state.clone();
            let path = props.path.clone();
            Callback::from(move |_| {
                tree_state.dispatch(TreeAction::SetExpanded {
                    path: path.clone(),
                    open: !expanded,
                });
            })
        };
        let on_add_folder = {
            let tree_state = tree_state.clone();
            let path = props.path.clone();
            let siblings = props.node.children.clone();
            Callback::from(move |event: MouseEvent| {
                event.stop_propagation();
                let label = prompt_folder_name()
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or_else(|| next_folder_name(&siblings));
                tree_state.dispatch(TreeAction::AddChild {
                    path: path.clone(),
                    node: TreeNode {
                        label,
                        content: None,
                        expanded: true,
                        children: Vec::new(),
                    },
                });
            })
        };

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
            Callback::from(move |event: MouseEvent| {
                event.stop_propagation();
                draft.set(label.clone());
                is_editing.set(true);
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

        return html! {
            <div>
                <div class="tree-row-wrap">
                    {
                        if *is_editing {
                            html! {
                                <div class="tree-row tree-row-edit">
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
                                <button type="button" class="tree-row" onclick={on_toggle}>
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
                                    <button
                                        type="button"
                                        class="tree-row-add"
                                        title="Add folder"
                                        onclick={on_add_folder}
                                    >
                                        { "+" }
                                    </button>
                                    <button
                                        type="button"
                                        class="tree-row-rename"
                                        title="Rename folder"
                                        onclick={on_rename}
                                    >
                                        { "✎" }
                                    </button>
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
    let on_click = {
        let tab_state = tab_state.clone();
        Callback::from(move |_| {
            let Some(content) = content.as_ref() else {
                return;
            };
            let new_content = TabContent::from_node(content);
            tab_state.dispatch(TabAction::OpenTab {
                label: label.clone(),
                content: new_content,
            });
        })
    };

    html! {
        <button
            type="button"
            class="tree-row"
            onclick={on_click}
            title={props.node.label.clone()}
        >
            <span class="tree-leaf-icon"></span>
            <span class="tree-label">{ props.node.label.clone() }</span>
        </button>
    }
}

fn prompt_folder_name() -> Option<String> {
    let window = web_sys::window()?;
    let name = window
        .prompt_with_message_and_default("Nome da pasta", "Nova pasta")
        .ok()
        .flatten()?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn next_folder_name(children: &[TreeNode]) -> String {
    let base = "Nova pasta";
    if !children.iter().any(|child| child.label == base) {
        return base.to_string();
    }
    for index in 2..1000 {
        let candidate = format!("{base} ({index})");
        if !children.iter().any(|child| child.label == candidate) {
            return candidate;
        }
    }
    format!("{base} ({})", children.len() + 1)
}

fn event_target_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}
