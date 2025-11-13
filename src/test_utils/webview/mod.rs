// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

//! This module provides a ground truth web page for comparing v1 and v2
//! logicals

use crate::test_utils::backend;
use leptos::prelude::*;

mod app;
mod display;
mod login;

use app::App;
use display::Display;
use login::Login;

#[derive(Clone)]
enum AppState {
    Login,
    Loading,
    Display(backend::Endpoints),
}

#[cfg(feature = "test_utils_webview")]
pub fn index() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    use leptos::prelude::*;
    let (read_state, write_state) = signal(AppState::Login);

    leptos::mount::mount_to_body(move || {
        view! {
            <App read_state=read_state write_state=write_state />
        }
    })
}
