use super::AppState;
use crate::test_utils::backend;
use leptos::prelude::*;

#[leptos::component]
pub fn Login(state: WriteSignal<AppState>) -> impl IntoView {
    log::info!("Drawing login page");

    let (read_username, write_username) = signal(String::new());
    let (read_password, write_password) = signal(String::new());
    let (read_twofa, write_twofa) = signal(String::new());

    let draw = move || {
        view! {
            <form method="post">
                <label for="username">"Username:"</label>
                <input type="text" id="username" name="username"  on:change=move |ev| {
                    let value = event_target_value(&ev);
                    log::info!("Username input changed: {}", &value);
                    write_username.set(value);

                } />
                <br />
                <label for="password">"Password:"</label>
                <input type="password" id="password" name="password"  on:change=move |ev| {
                    let value = event_target_value(&ev);
                    log::info!("Password input changed: {}", &value);
                    write_password.set(value);
                } />
                <br />
                <label for="twofa">"Twofa:"</label>
                <input type="text" id="twofa" name="twofa"  on:change=move |ev| {
                    let value = event_target_value(&ev);
                    log::info!("Twofa input changed: {}", &value);
                    write_twofa.set(value);
                } />
                <br />
                <button type="submit" on:click=move |_| {
                    log::info!("Login button clicked");
                    log::info!("    Username: {}, Password: {}", read_username.get(), read_password.get());

                    //muon::Client
                    leptos::task::spawn_local(async move {
                        log::info!("Spawning login task");

                        match backend::Endpoints::new_from_params(
                            &read_username.get(),
                            &read_password.get(),
                            &read_twofa.get(),
                        ).await {
                            Ok(endpoints) => {
                                log::info!("Login successful, client: {endpoints:?}");

                                state.set(AppState::Display(endpoints));
                            }
                            Err(e) => {
                                log::error!("Login failed: {e}");
                            }
                        }
                    });


                    log::info!("Task spawned, waiting for login to complete ");

                    state.set(AppState::Loading);
                }>"Login"</button>
                <br/>
                <br/>
                <br/>
                <br/>
                <p>This only works with CORS is disabled, for example:</p>
                <pre>"google-chrome --user-data-dir=/tmp/usr_data_chrome --disable-web-security"</pre>
            </form>
        }
    };

    view! {
        <div class="a">
        {draw}
        </div>
    }
}
