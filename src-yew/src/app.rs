use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::section::Section;
use crate::components::side::Side;
use crate::openapi::build_tree_from_openapi;
use crate::state::{TabState, TreeAction, TreeState};
use crate::tauri_api;

#[function_component(App)]
pub fn app() -> Html {
    let tree_state = use_reducer(TreeState::default);
    let tab_state = use_reducer(TabState::default);

    let on_save = {
        let tab_state = tab_state.clone();
        Callback::from(move |_| {
            let tab_state = tab_state.clone();
            spawn_local(async move {
                let Ok(Some(path)) = tauri_api::dialog_save().await else {
                    return;
                };
                let body = tab_state
                    .tabs
                    .get(tab_state.active_tab_id)
                    .map(|tab| tab.content.body.clone())
                    .unwrap_or_default();
                let _ = tauri_api::fs_write_text(&path, &body).await;
            });
        })
    };

    {
        let tree_state = tree_state.clone();
        let tab_state = tab_state.clone();
        use_effect_with((), move |_| {
            let handler = Closure::wrap(Box::new(move |event: JsValue| {
                let Some(payload) = event_payload(&event) else {
                    return;
                };
                match payload.as_str() {
                    "open-event" => {
                        let tree_state = tree_state.clone();
                        spawn_local(async move {
                            open_openapi(tree_state).await;
                        });
                    }
                    "save-event" => {
                        let tab_state = tab_state.clone();
                        spawn_local(async move {
                            save_active_body(tab_state).await;
                        });
                    }
                    _ => {}
                }
            }) as Box<dyn FnMut(JsValue)>);

            let _ = tauri_api::event_listen("menu-event", handler.as_ref());
            handler.forget();
            || ()
        });
    }

    html! {
        <ContextProvider<UseReducerHandle<TreeState>> context={tree_state}>
            <ContextProvider<UseReducerHandle<TabState>> context={tab_state}>
                <div class="app">
                    <aside class="sidebar">
                        <Side />
                    </aside>
                    <main class="main">
                        <Section on_save={on_save} />
                    </main>
                </div>
            </ContextProvider<UseReducerHandle<TabState>>>
        </ContextProvider<UseReducerHandle<TreeState>>>
    }
}

fn event_payload(event: &JsValue) -> Option<String> {
    js_sys::Reflect::get(event, &JsValue::from_str("payload"))
        .ok()
        .and_then(|value| value.as_string())
}

async fn open_openapi(tree_state: UseReducerHandle<TreeState>) {
    let Ok(Some(path)) = tauri_api::dialog_open().await else {
        return;
    };
    let Ok(text) = tauri_api::fs_read_text(&path).await else {
        return;
    };
    let Ok(node) = build_tree_from_openapi(&text) else {
        return;
    };
    tree_state.dispatch(TreeAction::AddRootChild(node));
}

async fn save_active_body(tab_state: UseReducerHandle<TabState>) {
    let Ok(Some(path)) = tauri_api::dialog_save().await else {
        return;
    };
    let body = tab_state
        .tabs
        .get(tab_state.active_tab_id)
        .map(|tab| tab.content.body.clone())
        .unwrap_or_default();
    let _ = tauri_api::fs_write_text(&path, &body).await;
}
