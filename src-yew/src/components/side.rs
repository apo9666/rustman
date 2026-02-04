use yew::prelude::*;

use crate::components::tree::directory::TreeDirectory;
use crate::state::{TreeAction, TreeNode, TreeState};

#[function_component(Side)]
pub fn side() -> Html {
    let tree_state = use_context::<UseReducerHandle<TreeState>>();

    let Some(tree_state) = tree_state else {
        return html! {};
    };

    let on_add_root = {
        let tree_state = tree_state.clone();
        Callback::from(move |_| {
            let label = prompt_folder_name()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| next_folder_name(&tree_state.root.children));
            tree_state.dispatch(TreeAction::AddRootChild(TreeNode {
                label,
                content: None,
                expanded: true,
                children: Vec::new(),
            }));
        })
    };

    html! {
        <nav class="tree-panel">
            <div class="tree-header">
                <span>{ "Folders" }</span>
                <button class="tree-add-root" type="button" onclick={on_add_root}>{ "+" }</button>
            </div>
            <div class="tree">
                <TreeDirectory node={tree_state.root.clone()} />
            </div>
        </nav>
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
