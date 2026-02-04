mod app;
mod components;
mod openapi;
mod state;
mod tauri_api;
mod utils;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
