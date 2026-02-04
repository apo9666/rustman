use gloo::events::EventListener;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::section::Section;
use crate::components::side::Side;
use crate::openapi::{build_openapi_from_tree, build_tree_from_openapi};
use crate::state::{Tab, TabAction, TabState, TreeAction, TreeNode, TreeState};
use crate::tauri_api;

#[function_component(App)]
pub fn app() -> Html {
    let tree_state = use_reducer(TreeState::default);
    let tab_state = use_reducer(TabState::default);
    let tree_state_ref = use_mut_ref(|| tree_state.clone());
    let app_ref = use_node_ref();
    let sidebar_width = use_state(|| 280.0);
    let dragging = use_state(|| false);
    let drag_state = use_mut_ref(|| DragState::default());
    let save_dialog_open = use_state(|| false);
    let save_tag = use_state(String::new);
    let server_dialog = use_state(|| None::<ServerDialogMode>);
    let server_input = use_state(String::new);
    let pending_tab = use_state(|| None::<Tab>);
    let pending_tab_index = use_state(|| None::<usize>);

    {
        let mut state = tree_state_ref.borrow_mut();
        *state = tree_state.clone();
    }

    let open_save_dialog = {
        let save_dialog_open = save_dialog_open.clone();
        let save_tag = save_tag.clone();
        let pending_tab = pending_tab.clone();
        let pending_tab_index = pending_tab_index.clone();
        Callback::from(move |(index, tab, tag): (usize, Tab, String)| {
            save_tag.set(tag);
            pending_tab.set(Some(tab));
            pending_tab_index.set(Some(index));
            save_dialog_open.set(true);
        })
    };

    let on_save = {
        let tab_state = tab_state.clone();
        let tree_state = tree_state.clone();
        let open_save_dialog = open_save_dialog.clone();
        Callback::from(move |_| {
            let index = tab_state.active_tab_id;
            let Some(tab) = tab_state.tabs.get(index).cloned() else {
                return;
            };
            let tag = infer_tag_from_selection(&tree_state.root, tree_state.selected_path.as_ref())
                .unwrap_or_default();
            open_save_dialog.emit((index, tab, tag));
        })
    };

    {
        let tree_state_ref = tree_state_ref.clone();
        use_effect_with((), move |_| {
            let handler = Closure::wrap(Box::new(move |event: JsValue| {
                let Some(payload) = event_payload(&event) else {
                    return;
                };
                match payload.as_str() {
                    "open-event" => {
                        let tree_state = tree_state_ref.borrow().clone();
                        spawn_local(async move {
                            open_openapi(tree_state).await;
                        });
                    }
                    "save-event" => {
                        let tree_state = tree_state_ref.borrow().clone();
                        spawn_local(async move {
                            export_openapi(tree_state).await;
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

    let pending_delete = tree_state.pending_delete.clone();

    let on_open_add_server = {
        let server_dialog = server_dialog.clone();
        let server_input = server_input.clone();
        Callback::from(move |_| {
            server_input.set("http://localhost".to_string());
            server_dialog.set(Some(ServerDialogMode::AddServer));
        })
    };

    let on_open_add_tag = {
        let server_dialog = server_dialog.clone();
        let server_input = server_input.clone();
        Callback::from(move |_| {
            server_input.set("New tag".to_string());
            server_dialog.set(Some(ServerDialogMode::AddTag));
        })
    };

    html! {
        <ContextProvider<UseReducerHandle<TreeState>> context={tree_state.clone()}>
            <ContextProvider<UseReducerHandle<TabState>> context={tab_state.clone()}>
                <div class="app" ref={app_ref}>
                    <aside class="sidebar" style={format!("width: {}px;", *sidebar_width)}>
                        <Side
                            on_add_server={on_open_add_server.clone()}
                            on_add_tag={on_open_add_tag}
                        />
                    </aside>
                    <div class="sidebar-resize" onmousedown={on_resize_start}></div>
                    <main class="main">
                        <Section on_save={on_save} on_add_server={on_open_add_server.clone()} />
                    </main>
                </div>
                {
                    if let Some(mode) = (*server_dialog).clone() {
                        let on_input_change = {
                            let server_input = server_input.clone();
                            Callback::from(move |event: InputEvent| {
                                server_input.set(event_target_value(&event));
                            })
                        };

                        let on_confirm = {
                            let tree_state = tree_state.clone();
                            let tab_state = tab_state.clone();
                            let server_dialog = server_dialog.clone();
                            let server_input = server_input.clone();
                            Callback::from(move |_| {
                                let value = (*server_input).trim().to_string();
                                match mode {
                                    ServerDialogMode::AddServer => {
                                        let Some(url) = normalize_server_url(&value) else {
                                            show_alert("Invalid server URL. Use http:// or https://.");
                                            return;
                                        };
                                        let new_index = tree_state.servers.len();
                                        tree_state.dispatch(TreeAction::AddServer { label: url });
                                        tree_state.dispatch(TreeAction::SetSelectedServer { index: new_index });
                                        if tab_state.tabs.is_empty() {
                                            tab_state.dispatch(TabAction::AddTab);
                                        }
                                    }
                                    ServerDialogMode::AddTag => {
                                        let label = if value.is_empty() {
                                            "New tag".to_string()
                                        } else {
                                            value.clone()
                                        };
                                        if tree_state
                                            .root
                                            .children
                                            .iter()
                                            .any(|child| child.content.is_none() && child.label == label)
                                        {
                                            show_alert("Tag already exists.");
                                            return;
                                        }
                                        tree_state.dispatch(TreeAction::AddChild {
                                            path: vec![],
                                            node: TreeNode {
                                                label,
                                                content: None,
                                                expanded: true,
                                                children: Vec::new(),
                                            },
                                        });
                                    }
                                }
                                server_dialog.set(None);
                            })
                        };

                        let on_cancel = {
                            let server_dialog = server_dialog.clone();
                            Callback::from(move |_| {
                                server_dialog.set(None);
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

                        let (title, label, placeholder) = match mode {
                            ServerDialogMode::AddServer => ("Add server", "Server URL", "http://localhost"),
                            ServerDialogMode::AddTag => ("Add tag", "Tag name", "New tag"),
                        };

                        html! {
                            <div class="modal-backdrop">
                                <div class="modal">
                                    <h2 class="modal-title">{ title }</h2>
                                    <label class="modal-label">{ label }</label>
                                    <input
                                        class="modal-input"
                                        value={(*server_input).clone()}
                                        placeholder={placeholder}
                                        oninput={on_input_change}
                                        onkeydown={on_keydown}
                                        autofocus=true
                                    />
                                    <div class="modal-actions">
                                        <button class="button secondary" onclick={on_cancel_click}>{ "Cancel" }</button>
                                        <button class="button" onclick={on_confirm_click}>{ "Save" }</button>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if *save_dialog_open {
                        let on_save_tag_change = {
                            let save_tag = save_tag.clone();
                            Callback::from(move |event: InputEvent| {
                                save_tag.set(event_target_value(&event));
                            })
                        };

                        let on_confirm = {
                            let tree_state = tree_state.clone();
                            let tab_state = tab_state.clone();
                            let pending_tab = pending_tab.clone();
                            let pending_tab_index = pending_tab_index.clone();
                            let save_tag = save_tag.clone();
                            let save_dialog_open = save_dialog_open.clone();
                            Callback::from(move |_| {
                                let Some(tab) = (*pending_tab).clone() else {
                                    save_dialog_open.set(false);
                                    return;
                                };

                                let tag_label = (*save_tag).trim().to_string();
                                let use_tag = !tag_label.is_empty();

                                let path_value = normalize_request_path(&tab.content.url);
                                let path_label = strip_query(&path_value);
                                if path_label.is_empty() {
                                    show_alert("Path inválido. Use /caminho.");
                                    return;
                                }
                                let label = path_label.clone();

                                let root = tree_state.root.clone();
                                let existing_path = find_request_path_by_label(&root, &label);
                                let selected_path = tree_state.selected_path.clone();
                                let is_selected_same = selected_path
                                    .as_ref()
                                    .and_then(|path| node_at_path(&root, path))
                                    .map(|node| node.content.is_some() && node.label == label)
                                    .unwrap_or(false);

                                if let Some(path) = existing_path.as_ref() {
                                    if !is_selected_same || selected_path.as_ref() != Some(path) {
                                        show_alert("Já existe uma request com esse path.");
                                        return;
                                    }
                                }

                                let new_node = TreeNode {
                                    label: label.clone(),
                                    content: Some(tab.content.clone()),
                                    expanded: false,
                                    children: Vec::new(),
                                };

                                if let Some(path) = existing_path {
                                    if is_selected_same && selected_path.as_ref() == Some(&path) {
                                        tree_state.dispatch(TreeAction::ReplaceNode {
                                            path,
                                            node: new_node,
                                        });

                                        if tab.content.url != label {
                                            tab_state.dispatch(TabAction::UpdateUrl {
                                                index: tab_state.active_tab_id,
                                                url: label.clone(),
                                            });
                                        }
                                        tab_state.dispatch(TabAction::RenameTab {
                                            index: tab_state.active_tab_id,
                                            label,
                                        });
                                        tab_state.dispatch(TabAction::SetDirty {
                                            index: tab_state.active_tab_id,
                                            dirty: false,
                                        });

                                        pending_tab.set(None);
                                        pending_tab_index.set(None);
                                        save_dialog_open.set(false);
                                        return;
                                    }
                                }

                                if use_tag {
                                    let (tag_index, tag_exists) =
                                        find_tag_index_by_label(&root, &tag_label);

                                    if !tag_exists {
                                        tree_state.dispatch(TreeAction::AddChild {
                                            path: vec![],
                                            node: TreeNode {
                                                label: tag_label.clone(),
                                                content: None,
                                                expanded: true,
                                                children: Vec::new(),
                                            },
                                        });
                                    }

                                    let tag_path = vec![tag_index];
                                    tree_state.dispatch(TreeAction::AddChild {
                                        path: tag_path,
                                        node: new_node,
                                    });
                                } else {
                                    tree_state.dispatch(TreeAction::AddChild {
                                        path: vec![],
                                        node: new_node,
                                    });
                                }

                                if tab.content.url != label {
                                    tab_state.dispatch(TabAction::UpdateUrl {
                                        index: tab_state.active_tab_id,
                                        url: label.clone(),
                                    });
                                }
                                tab_state.dispatch(TabAction::RenameTab {
                                    index: tab_state.active_tab_id,
                                    label,
                                });
                                tab_state.dispatch(TabAction::SetDirty {
                                    index: tab_state.active_tab_id,
                                    dirty: false,
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
                                    <label class="modal-label" for="save-tag">{ "Tag" }</label>
                                    <input
                                        id="save-tag"
                                        class="modal-input"
                                        value={(*save_tag).clone()}
                                        oninput={on_save_tag_change}
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ServerDialogMode {
    AddServer,
    AddTag,
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

fn normalize_server_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let url = Url::parse(trimmed).ok()?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return None,
    }

    let mut normalized = url.to_string();
    while normalized.ends_with('/') {
        normalized.pop();
    }
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

async fn open_openapi(tree_state: UseReducerHandle<TreeState>) {
    let path = match tauri_api::dialog_open().await {
        Ok(Some(path)) => path,
        Ok(None) => return,
        Err(err) => {
            show_alert(&format!("Falha ao abrir diálogo: {:?}", err));
            return;
        }
    };
    let text = match tauri_api::fs_read_text(&path).await {
        Ok(text) => text,
        Err(err) => {
            show_alert(&format!(
                "Falha ao ler o arquivo: {}",
                tauri_api::js_error_to_string(&err)
            ));
            return;
        }
    };
    let (root, servers) = match build_tree_from_openapi(&text) {
        Ok(result) => result,
        Err(err) => {
            show_alert(&format!("Falha ao importar OpenAPI: {err}"));
            return;
        }
    };
    tree_state.dispatch(TreeAction::SetTree { root, servers });
    let title = filename_from_path(&path);
    let _ = tauri_api::set_window_title(&title).await;
}

async fn export_openapi(tree_state: UseReducerHandle<TreeState>) {
    let path = match tauri_api::dialog_save().await {
        Ok(Some(path)) => path,
        Ok(None) => return,
        Err(err) => {
            show_alert(&format!("Falha ao abrir diálogo de salvar: {err:?}"));
            return;
        }
    };

    let text = match build_openapi_from_tree(&tree_state.root, &tree_state.servers) {
        Ok(text) => text,
        Err(err) => {
            show_alert(&err);
            return;
        }
    };

    let target = ensure_openapi_extension(&path);
    let create_new_options = r#"{"createNew":true,"create":true}"#;
    match tauri_api::fs_write_text_with_options(&target, &text, Some(create_new_options)).await {
        Ok(()) => {
            let title = filename_from_path(&target);
            let _ = tauri_api::set_window_title(&title).await;
            return;
        }
        Err(err) => {
            let message = tauri_api::js_error_to_string(&err);
            if is_exists_error(&message) {
                if !show_confirm(&format!("Arquivo já existe. Substituir?\n{target}")) {
                    return;
                }
                if let Err(err) = tauri_api::fs_write_text(&target, &text).await {
                    show_alert(&format!(
                        "Falha ao salvar o arquivo: {}",
                        tauri_api::js_error_to_string(&err)
                    ));
                    return;
                }
                let title = filename_from_path(&target);
                let _ = tauri_api::set_window_title(&title).await;
                return;
            }
            show_alert(&format!("Falha ao salvar o arquivo: {message}"));
        }
    }
}

fn ensure_openapi_extension(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".yaml") || lower.ends_with(".yml") || lower.ends_with(".json") {
        return path.to_string();
    }
    format!("{path}.yaml")
}

fn filename_from_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized
        .split('/')
        .last()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("rustman")
        .to_string()
}

fn infer_tag_from_selection(root: &TreeNode, selected: Option<&Vec<usize>>) -> Option<String> {
    let path = selected?;
    let node = node_at_path(root, path)?;
    if node.content.is_none() {
        return Some(node.label.clone());
    }
    if let Some(parent_path) = path.split_last().map(|(_, parent)| parent) {
        if parent_path.is_empty() {
            return None;
        }
        let parent = node_at_path(root, parent_path)?;
        if parent.content.is_none() {
            return Some(parent.label.clone());
        }
    }
    None
}

fn find_tag_index_by_label(root: &TreeNode, label: &str) -> (usize, bool) {
    for (index, child) in root.children.iter().enumerate() {
        if child.content.is_none() && child.label == label {
            return (index, true);
        }
    }
    (root.children.len(), false)
}

fn normalize_request_path(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.starts_with('/') || trimmed.starts_with('?') {
        return trimmed.to_string();
    }

    format!("/{}", trimmed)
}

fn strip_query(value: &str) -> String {
    value.split('?').next().unwrap_or("").to_string()
}

fn find_request_path_by_label(root: &TreeNode, label: &str) -> Option<Vec<usize>> {
    for (index, child) in root.children.iter().enumerate() {
        if child.content.is_some() {
            if child.label == label {
                return Some(vec![index]);
            }
            continue;
        }
        for (child_index, grand) in child.children.iter().enumerate() {
            if grand.content.is_some() && grand.label == label {
                return Some(vec![index, child_index]);
            }
        }
    }
    None
}

fn event_target_value(event: &InputEvent) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn show_alert(message: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.alert_with_message(message);
    }
}

fn show_confirm(message: &str) -> bool {
    web_sys::window()
        .and_then(|window| window.confirm_with_message(message).ok())
        .unwrap_or(false)
}

fn is_exists_error(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("exists") || lower.contains("exist") || lower.contains("eexist")
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
