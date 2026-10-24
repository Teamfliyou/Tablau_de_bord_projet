mod api;
mod app;
mod icons;
mod maj;
mod pages;
mod state;
mod state_win;
mod ui;
mod util;
mod win;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
