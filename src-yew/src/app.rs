use gloo::events::EventListener;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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
    let app_ref = use_node_ref();
    let sidebar_width = use_state(|| 280.0);
    let dragging = use_state(|| false);
    let drag_state = use_mut_ref(|| DragState::default());

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

    let on_resize_start = {
        let dragging = dragging.clone();
        let sidebar_width = sidebar_width.clone();
        let drag_state = drag_state.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            dragging.set(true);
            let mut state = drag_state.borrow_mut();
            state.start_x = event.client_x() as f64;
            state.start_width = *sidebar_width;
        })
    };

    {
        let dragging = dragging.clone();
        let sidebar_width = sidebar_width.clone();
        let app_ref = app_ref.clone();
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
                let delta = event.client_x() as f64 - state.start_x;
                let mut next_width = state.start_width + delta;

                let container_width = app_ref
                    .cast::<web_sys::Element>()
                    .map(|element| element.get_bounding_client_rect().width())
                    .unwrap_or(1200.0);

                let min_sidebar = 200.0;
                let min_main = 360.0;
                let max_sidebar = (container_width - min_main).max(min_sidebar);

                if next_width < min_sidebar {
                    next_width = min_sidebar;
                }
                if next_width > max_sidebar {
                    next_width = max_sidebar;
                }

                sidebar_width.set(next_width);
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
        <ContextProvider<UseReducerHandle<TreeState>> context={tree_state}>
            <ContextProvider<UseReducerHandle<TabState>> context={tab_state}>
                <div class="app" ref={app_ref}>
                    <aside class="sidebar" style={format!("width: {}px;", *sidebar_width)}>
                        <Side />
                    </aside>
                    <div class="sidebar-resize" onmousedown={on_resize_start}></div>
                    <main class="main">
                        <Section on_save={on_save} />
                    </main>
                </div>
            </ContextProvider<UseReducerHandle<TabState>>>
        </ContextProvider<UseReducerHandle<TreeState>>>
    }
}

#[derive(Default)]
struct DragState {
    start_x: f64,
    start_width: f64,
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
