use gloo::events::EventListener;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::section::Section;
use crate::components::side::Side;
use crate::openapi::build_tree_from_openapi;
use crate::state::{Tab, TabAction, TabState, TreeAction, TreeNode, TreeState};
use crate::tauri_api;

#[function_component(App)]
pub fn app() -> Html {
    let tree_state = use_reducer(TreeState::default);
    let tab_state = use_reducer(TabState::default);
    let app_ref = use_node_ref();
    let sidebar_width = use_state(|| 280.0);
    let dragging = use_state(|| false);
    let drag_state = use_mut_ref(|| DragState::default());
    let save_dialog_open = use_state(|| false);
    let save_name = use_state(String::new);
    let pending_tab = use_state(|| None::<Tab>);
    let pending_tab_index = use_state(|| None::<usize>);

    let open_save_dialog = {
        let save_dialog_open = save_dialog_open.clone();
        let save_name = save_name.clone();
        let pending_tab = pending_tab.clone();
        let pending_tab_index = pending_tab_index.clone();
        Callback::from(move |(index, tab): (usize, Tab)| {
            save_name.set(tab.label.clone());
            pending_tab.set(Some(tab));
            pending_tab_index.set(Some(index));
            save_dialog_open.set(true);
        })
    };

    let on_save = {
        let tab_state = tab_state.clone();
        let open_save_dialog = open_save_dialog.clone();
        Callback::from(move |_| {
            let index = tab_state.active_tab_id;
            let Some(tab) = tab_state.tabs.get(index).cloned() else {
                return;
            };
            open_save_dialog.emit((index, tab));
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
                        let index = tab_state.active_tab_id;
                        let Some(tab) = tab_state.tabs.get(index).cloned() else {
                            return;
                        };
                        open_save_dialog.emit((index, tab));
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

    let pending_delete = tree_state.pending_delete.clone();

    html! {
        <ContextProvider<UseReducerHandle<TreeState>> context={tree_state.clone()}>
            <ContextProvider<UseReducerHandle<TabState>> context={tab_state.clone()}>
                <div class="app" ref={app_ref}>
                    <aside class="sidebar" style={format!("width: {}px;", *sidebar_width)}>
                        <Side />
                    </aside>
                    <div class="sidebar-resize" onmousedown={on_resize_start}></div>
                    <main class="main">
                        <Section on_save={on_save} />
                    </main>
                </div>
                {
                    if *save_dialog_open {
                        let on_save_name_change = {
                            let save_name = save_name.clone();
                            Callback::from(move |event: InputEvent| {
                                save_name.set(event_target_value(&event));
                            })
                        };

                        let on_confirm = {
                            let tree_state = tree_state.clone();
                            let tab_state = tab_state.clone();
                            let pending_tab = pending_tab.clone();
                            let pending_tab_index = pending_tab_index.clone();
                            let save_name = save_name.clone();
                            let save_dialog_open = save_dialog_open.clone();
                            Callback::from(move |_| {
                                let Some(tab) = (*pending_tab).clone() else {
                                    save_dialog_open.set(false);
                                    return;
                                };

                                let name = (*save_name).trim().to_string();
                                let label = if name.is_empty() {
                                    tab.label.clone()
                                } else {
                                    name
                                };

                                let root = tree_state.root.clone();
                                let selected = tree_state.selected_path.clone();
                                let target_path = resolve_save_path(&root, selected.as_ref());

                                tree_state.dispatch(TreeAction::AddChild {
                                    path: target_path,
                                    node: TreeNode {
                                        label: label.clone(),
                                        content: Some(tab.content.clone()),
                                        expanded: false,
                                        children: Vec::new(),
                                    },
                                });

                                tab_state.dispatch(TabAction::RenameTab {
                                    index: tab_state.active_tab_id,
                                    label,
                                });

                                pending_tab.set(None);
                                pending_tab_index.set(None);
                                save_dialog_open.set(false);
                            })
                        };

                        let on_cancel = {
                            let pending_tab = pending_tab.clone();
                            let pending_tab_index = pending_tab_index.clone();
                            let save_dialog_open = save_dialog_open.clone();
                            Callback::from(move |_| {
                                pending_tab.set(None);
                                pending_tab_index.set(None);
                                save_dialog_open.set(false);
                            })
                        };

                        let on_confirm_click = {
                            let on_confirm = on_confirm.clone();
                            Callback::from(move |_event: MouseEvent| on_confirm.emit(()))
                        };

                        let on_cancel_click = {
                            let on_cancel = on_cancel.clone();
                            Callback::from(move |_event: MouseEvent| on_cancel.emit(()))
                        };

                        let on_keydown = {
                            let on_confirm = on_confirm.clone();
                            let on_cancel = on_cancel.clone();
                            Callback::from(move |event: KeyboardEvent| match event.key().as_str() {
                                "Enter" => {
                                    event.prevent_default();
                                    on_confirm.emit(());
                                }
                                "Escape" => {
                                    event.prevent_default();
                                    on_cancel.emit(());
                                }
                                _ => {}
                            })
                        };

                        html! {
                            <div class="modal-backdrop">
                                <div class="modal">
                                    <h2 class="modal-title">{ "Salvar no Tree" }</h2>
                                    <label class="modal-label" for="save-name">{ "Nome da aba" }</label>
                                    <input
                                        id="save-name"
                                        class="modal-input"
                                        value={(*save_name).clone()}
                                        oninput={on_save_name_change}
                                        onkeydown={on_keydown}
                                        autofocus=true
                                    />
                                    <div class="modal-actions">
                                        <button class="button secondary" onclick={on_cancel_click}>{ "Cancelar" }</button>
                                        <button class="button" onclick={on_confirm_click}>{ "Salvar" }</button>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if let Some(pending) = pending_delete {
                        let label = pending.label.clone();
                        let path = pending.path.clone();
                        let on_confirm = {
                            let tree_state = tree_state.clone();
                            Callback::from(move |_| {
                                tree_state.dispatch(TreeAction::RemoveNode { path: path.clone() });
                            })
                        };

                        let on_cancel = {
                            let tree_state = tree_state.clone();
                            Callback::from(move |_| {
                                tree_state.dispatch(TreeAction::ClearPendingDelete);
                            })
                        };

                        let on_confirm_click = {
                            let on_confirm = on_confirm.clone();
                            Callback::from(move |_event: MouseEvent| on_confirm.emit(()))
                        };

                        let on_cancel_click = {
                            let on_cancel = on_cancel.clone();
                            Callback::from(move |_event: MouseEvent| on_cancel.emit(()))
                        };

                        html! {
                            <div class="modal-backdrop">
                                <div class="modal">
                                    <h2 class="modal-title">{ "Confirmar remoção" }</h2>
                                    <p class="modal-text">{ format!("Remover \"{}\" do tree?", label) }</p>
                                    <div class="modal-actions">
                                        <button class="button secondary" onclick={on_cancel_click}>{ "Cancelar" }</button>
                                        <button class="button danger" onclick={on_confirm_click}>{ "Remover" }</button>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
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

fn resolve_save_path(root: &TreeNode, selected: Option<&Vec<usize>>) -> Vec<usize> {
    if let Some(path) = selected {
        if is_folder_path(root, path) {
            return path.clone();
        }

        if let Some(parent) = path.split_last().map(|(_, parent)| parent.to_vec()) {
            if is_folder_path(root, &parent) {
                return parent;
            }
        }
    }

    Vec::new()
}

fn event_target_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn is_folder_path(root: &TreeNode, path: &[usize]) -> bool {
    node_at_path(root, path)
        .map(|node| node.content.is_none())
        .unwrap_or(false)
}

fn node_at_path<'a>(root: &'a TreeNode, path: &[usize]) -> Option<&'a TreeNode> {
    if path.is_empty() {
        return Some(root);
    }

    let mut current = root;
    for index in path {
        current = current.children.get(*index)?;
    }
    Some(current)
}
