use yew::prelude::*;

use crate::state::{ServerAuth, TreeAction, TreeState};

#[derive(Properties, Clone, PartialEq)]
pub struct RequestTitleProps {
    pub title: String,
    pub on_save: Callback<()>,
}

#[function_component(RequestTitle)]
pub fn request_title(props: &RequestTitleProps) -> Html {
    let tree_state = use_context::<UseReducerHandle<TreeState>>();
    let (has_auth, selected_index) = tree_state
        .as_ref()
        .and_then(|state| {
            let index = state.selected_server?;
            let server = state.servers.get(index)?;
            let has_auth = !matches!(server.auth, ServerAuth::None);
            Some((has_auth, Some(index)))
        })
        .unwrap_or((false, None));
    let on_save = {
        let on_save = props.on_save.clone();
        Callback::from(move |_| on_save.emit(()))
    };
    let on_open_auth = {
        let tree_state = tree_state.clone();
        let selected_index = selected_index;
        Callback::from(move |_| {
            if let (Some(tree_state), Some(index)) = (tree_state.as_ref(), selected_index) {
                tree_state.dispatch(TreeAction::RequestAuth { index });
            }
        })
    };

    html! {
        <div class="request-title">
            <div class="request-title-left">
                {
                    if has_auth {
                        html! {
                            <button
                                type="button"
                                class="auth-indicator"
                                title="Server auth enabled"
                                onclick={on_open_auth}
                            >
                                <svg viewBox="0 0 24 24" aria-hidden="true">
                                    <path d="M12 3.5c2.21 0 4 1.79 4 4v2h1.5c.83 0 1.5.67 1.5 1.5v8c0 .83-.67 1.5-1.5 1.5h-11c-.83 0-1.5-.67-1.5-1.5v-8c0-.83.67-1.5 1.5-1.5H8v-2c0-2.21 1.79-4 4-4zm-2 6h4v-2c0-1.1-.9-2-2-2s-2 .9-2 2v2z"/>
                                </svg>
                            </button>
                        }
                    } else {
                        html! {}
                    }
                }
                <h1>{ props.title.clone() }</h1>
            </div>
            <div class="request-actions">
                <button class="button secondary" onclick={on_save}>{ "Save" }</button>
            </div>
        </div>
    }
}
