mod api;
mod app;
mod icons;
mod maj;
mod pages;
mod state;
mod state_win;
mod win;
mod ui;
mod util;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
