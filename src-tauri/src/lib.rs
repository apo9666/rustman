use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .menu(|handle| {
            let open = MenuItem::with_id(handle, "open", "Open", true, Some("cmdOrControl+O"))?;
            let save = MenuItem::with_id(handle, "save", "Save", true, Some("cmdOrControl+S"))?;
            let close = MenuItem::with_id(handle, "close", "Close", true, Some("cmdOrControl+Q"))?;
            let file_menu = Submenu::with_items(handle, "File", true, &[&open, &save, &close])?;
            Menu::with_items(handle, &[&file_menu])
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "save" => {
                let _ = app.emit("menu-event", "save-event");
            }
            "open" => {
                let _ = app.emit("menu-event", "open-event");
            }
            "close" => {
                app.exit(0);
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
