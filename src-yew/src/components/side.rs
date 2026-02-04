use yew::prelude::*;

use crate::components::tree::directory::TreeDirectory;
use crate::state::TreeState;

#[function_component(Side)]
pub fn side() -> Html {
    let tree_state = use_context::<UseReducerHandle<TreeState>>();

    let Some(tree_state) = tree_state else {
        return html! {};
    };

    html! {
        <nav class="tree">
            <TreeDirectory node={tree_state.root.clone()} />
        </nav>
    }
}
