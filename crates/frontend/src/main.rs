mod api;
mod app;
mod icons;
mod pages;
mod state;
mod ui;
mod util;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
