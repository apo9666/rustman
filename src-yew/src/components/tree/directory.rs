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

    if has_children && props.node.label == "Root" {
        return html! { <div class="tree">{ render_children(&props.node, &props.path) }</div> };
    }

    if has_children {
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

        return html! {
            <div>
                <button type="button" class="tree-row" onclick={on_toggle}>
                    <span>{ if expanded { "v" } else { ">" } }</span>
                    <span class="tree-label">{ props.node.label.clone() }</span>
                </button>
                {
                    if expanded {
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
            <span class="tree-label">{ props.node.label.clone() }</span>
            <span class="muted">{ "->" }</span>
        </button>
    }
}
