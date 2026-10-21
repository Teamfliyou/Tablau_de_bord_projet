//! Icônes SVG (traces du jeu d'icônes historique).

use leptos::prelude::*;

fn chemins(nom: &str) -> &'static [&'static str] {
    match nom {
        "dashboard" => &["M4 20v-7", "M10 20V8", "M16 20v-10", "M2 20h20"],
        "calendar" => &["M3 4h18v16a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4z", "M8 2v4", "M16 2v4", "M3 10h18"],
        "list" => &["M8 6h13", "M8 12h13", "M8 18h13", "M3 6h.01", "M3 12h.01", "M3 18h.01"],
        "gear" => &[
            "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z",
            "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z",
        ],
        "devoir" => &["M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z", "M14 2v6h6", "M16 13H8", "M16 17H8", "M10 9H8"],
        "ie" => &["M12 20h9", "M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"],
        "ds" => &["M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z", "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"],
        "exam" => &["M12 15a7 7 0 1 0 0-14 7 7 0 0 0 0 14z", "M8.21 13.89 7 23l5-3 5 3-1.21-9.12"],
        "dot" => &["M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16z"],
        "filter" => &["M22 3H2l8 9.46V19l4 2v-8.54L22 3z"],
        "search" => &["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z", "M21 21l-4.35-4.35"],
        "check" => &["M20 6L9 17l-5-5"],
        "moon" => &["M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"],
        "trash" => &["M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"],
        "pencil" => &["M12 20h9M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4L16.5 3.5z"],
        "plus" => &["M12 4v16m8-8H4"],
        "chevron-left" => &["M15 19l-7-7 7-7"],
        "chevron-right" => &["M9 5l7 7-7 7"],
        "refresh" => &["M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"],
        "pin" => &["M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z", "M15 11a3 3 0 11-6 0 3 3 0 016 0z"],
        "user" => &["M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2", "M12 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z"],
        "clock" => &["M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"],
        "clipboard" => &["M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"],
        "bolt" => &["M13 10V3L4 14h7v7l9-11h-7z"],
        "sparkles" => &["M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"],
        "copy" => &["M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"],
        "download" => &["M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"],
        "upload" => &["M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4 4m0 0L9 8m4 4V4"],
        "logout" => &["M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"],
        "mortarboard" => &["M12 14l9-5-9-5-9 5 9 5z", "M12 14l6.16-3.42a12.08 12.08 0 01.84 6.42M12 14v6M5 10v6a4 4 0 0014 0"],
        _ => &[],
    }
}

/// Icône à contour (ou pleine pour « dot »), couleur héritée de `currentColor`.
#[component]
pub fn Icone(#[prop(into)] nom: String, #[prop(optional, into)] class: String) -> impl IntoView {
    let plein = nom == "dot";
    let traces = chemins(&nom);
    view! {
        <svg
            class=class
            fill=if plein { "currentColor" } else { "none" }
            stroke=if plein { "none" } else { "currentColor" }
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            {traces.iter().map(|d| view! { <path d=*d /> }).collect_view()}
        </svg>
    }
}

/// Spinner de chargement.
#[component]
pub fn Spinner(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=format!("animate-spin {class}") fill="none" viewBox="0 0 24 24" aria-hidden="true">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
    }
}
