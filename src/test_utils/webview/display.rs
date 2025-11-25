use super::AppState;
use crate::test_utils::backend;
use leptos::prelude::*;

#[leptos::component]
pub fn LogicalComponent(
    logical: backend::v1::Server,
    variance: f64,
) -> impl IntoView {
    view! {
    <tr>
        <td>{logical.name.clone()}</td>
        <td>{logical.status}</td>
        <td>{logical.load}</td>
        <td style="text-align:left;">{logical.score}</td>
        {
            if variance > 0.1 {
                view! { <td style="text-align:left; color: red;">{format!("{:.6} %", variance)}</td> }
            } else {
                view! { <td style="text-align:left; color: black;">{format!("{:.6} %", variance)}</td> }
            }
        }
    </tr>
    }.into_any()
}

#[leptos::component]
pub fn Display(state: ReadSignal<AppState>) -> impl IntoView {
    type Logicals = (backend::v1::Logicals, backend::v1::Logicals);
    let (read_logicals, write_logicals) = signal::<Option<Logicals>>(None);

    let logicals = move || {
        if let Some((v1, v2)) = read_logicals.get() {
            let v2_lookup: std::collections::HashMap<
                &str,
                backend::v1::Server,
            > = v2
                .logical_servers
                .iter()
                .map(|s| (s.name.as_str(), s.clone()))
                .collect();

            view! {
                <div class="b">
                <table>
                <tr>
                    <th>Name</th>
                    <th>Status</th>
                    <th>Load</th>
                    <th>Score</th>
                    <th>Variance</th>
                </tr>
                {
                    v1.logical_servers.iter().map(|l1| {
                        let p_l1 = l1.clone();

                        if let Some(p_l2) = v2_lookup.get(l1.name.as_str()) {
                            let variance = backend::compute_variance_mbps(&p_l1, p_l2);

                            if p_l1.load == p_l2.load && variance > 0.01 {
                                view! {
                                    <LogicalComponent logical=p_l1 variance=variance/>
                                    <LogicalComponent logical=p_l2.clone() variance=variance/>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }
                        } else {
                            view! {
                                <LogicalComponent logical=p_l1 variance=0.0/>
                                <tr>
                                    <td colspan="8" style="text-align:center; color: gray;">"No matching logical"</td>
                                </tr>
                            }.into_any()
                        }
                    }).collect::<Vec<_>>()
                }
                </table>
                </div>
            }.into_any()
        } else {
            view! {
                <div class="a">"No logicals loaded"</div>
            }
            .into_any()
        }
    };

    let on_click = move |_| {
        log::info!("Getting logicals...");
        match &state.get() {
            AppState::Display(endpoints) => {
                let mut local_endpoints_a = endpoints.clone();
                let mut local_endpoints_b = endpoints.clone();

                write_logicals.set(None);

                leptos::task::spawn_local(async move {
                    let (v1_logicals, v2_logicals) = futures::join!(
                        backend::v1::get_logicals(
                            &mut local_endpoints_a,
                            |_| true
                        ),
                        backend::v2::get_logicals(
                            &mut local_endpoints_b,
                            |_| true
                        )
                    );

                    if v1_logicals.is_err() || v2_logicals.is_err() {
                        log::error!("Failed to fetch logicals: v1: {v1_logicals:?}, v2: {v2_logicals:?}");
                        return;
                    }

                    write_logicals.set(Some((
                        v1_logicals.unwrap(),
                        v2_logicals.unwrap(),
                    )));
                });
            }
            _ => {
                log::warn!(
                    "State is not in Display mode, cannot fetch logicals"
                );
            }
        }
    };

    let country = std::str::from_utf8(backend::USER_COUNTRY)
        .expect("Invalid country code");

    view! {
        <div>
            <div class="toolbar">
                <div>
                    <pre>
                        ip: {backend::USER_IP_ADDRESS},
                        country: {country},
                        latitude: {backend::USER_LATITUDE},
                        longitude: {backend::USER_LONGITUDE}
                    </pre>
                </div>
                <button on:click=on_click class="load_logicals">Load Logicals</button>
            </div>

            <div>
            {logicals}
            </div>
        </div>
    }
}
