use super::AppState;
use super::{Display, Login};
use leptos::prelude::*;

#[leptos::component]
pub fn App(
    read_state: ReadSignal<AppState>,
    write_state: WriteSignal<AppState>,
) -> impl IntoView {
    let display = move || match read_state.get() {
        AppState::Login => view! { <Login state=write_state /> }.into_any(),
        AppState::Display(_endpoints) => {
            view! { <Display state=read_state /> }.into_any()
        }
        AppState::Loading => {
            view! { <div class="a">"Logging in ..."</div> }.into_any()
        }
    };

    view! {
        {display}
    }
}
