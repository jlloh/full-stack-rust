mod abandon_confirmation_modal;
mod panel;
// use gloo_console::info;
mod button;
mod navbar;
mod tiles;

use crate::abandon_confirmation_modal::AbandonConfirmationModal;
use crate::button::GetNumberState;
use crate::tiles::Tiles;
use button::TheButton;
use common::{ServerSentData, UserInfo};
use futures::stream::StreamExt;
use gloo_console::log;
use gloo_net::eventsource::futures::EventSource;
use gloo_net::http::Request;
use gloo_net::Error;
use navbar::NavBar;
use panel::Panel;
use serde::de;
use sycamore::futures::*;
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};

#[derive(Route)]
enum AppRoutes {
    #[to("/<subapp>")]
    SubApp(String),
    #[not_found]
    NotFound,
}

fn main() {
    sycamore::render(|cx| {
        let username = create_signal(cx, "anonymous".to_string());
        let is_logged_in = create_signal(cx, false);
        let get_number_state = create_signal(cx, GetNumberState::New);
        let assigned_number = create_signal(cx, None::<i32>);
        let should_display_abandon_modal = create_signal(cx, false);
        let selected_number = create_signal(cx, None::<i32>);
        // let items = create_signal(cx, Vec::<i32>::new());
        let abandoned_numbers = create_signal(cx, Vec::<i32>::new());
        let done_numbers = create_signal(cx, Vec::<i32>::new());
        // Derived signals
        let should_disable_button = create_memo(cx, || {
            if *is_logged_in.get() {
                match *get_number_state.get() {
                    GetNumberState::New => false,
                    GetNumberState::Processing => true,
                    GetNumberState::Done => false,
                    GetNumberState::Failed => false,
                    GetNumberState::Locked => true,
                }
            } else {
                true
            }
        });
        let button_text = create_memo(cx, || {
            if *is_logged_in.get() {
                match *get_number_state.get() {
                    GetNumberState::New => "Get Number".to_string(),
                    GetNumberState::Processing => "Processing".to_string(),
                    GetNumberState::Done => "Abandon Number".to_string(),
                    GetNumberState::Failed => "Failed".to_string(),
                    GetNumberState::Locked => "Locked".to_string(),
                }
            } else {
                "Please Login to Get Number".to_string()
            }
        });

        spawn_local_scoped(cx, async move {
            // let user_info = get_user_info().await;
            let user_info = get_json_response::<UserInfo>("/public/get_user_info2").await;
            match user_info {
                Ok(x) => {
                    username.set(x.email);
                    is_logged_in.set(x.is_logged_in);
                }
                Err(e) => log!(format!("Failed to call get_user_info2 with error: {}", e)),
            }
        });

        // TODO: Remove this if we can push from serverside
        create_effect(cx, move || {
            if *get_number_state.get() == GetNumberState::New {
                spawn_local_scoped(cx, async move {
                    let result =
                        get_json_response::<ServerSentData>("/public/get_selected_number").await;
                    if let Ok(data) = result {
                        selected_number.set(data.selected_number);
                        assigned_number.set(data.assigned_number);
                        abandoned_numbers.set(data.abandoned_numbers);
                        done_numbers.set(data.done_numbers);
                    }
                });
            }
        });
        // server sent events
        let mut es = EventSource::new("/public/subscribe").unwrap();
        let mut server_sent_stream = es.subscribe("data").unwrap();
        spawn_local_scoped(cx, async move {
            // weird bug, doesn't work if i don't call es.state() here
            log!(format!("{:#?}", es.state()));
            while let Some(Ok((_event_type, msg))) = server_sent_stream.next().await {
                // let k = msg.data();
                let string_data = msg.data().as_string().unwrap();
                let data: ServerSentData = serde_json::from_str(&string_data)
                    .expect("Expected to be able to deserialise server sent event");
                selected_number.set(data.selected_number);
                assigned_number.set(data.assigned_number);
                abandoned_numbers.set(data.abandoned_numbers);
                done_numbers.set(data.done_numbers);
            }
        });

        // Can an effect update a signal?
        create_effect(cx, || {
            if *selected_number.get() == *assigned_number.get() {
                if selected_number.get().is_some() {
                    get_number_state.set(GetNumberState::Locked)
                }
            } else if *get_number_state.get() == GetNumberState::Locked {
                // if numbers are not equal, and state is locked, reset it
                get_number_state.set(GetNumberState::New)
            } else if (*assigned_number.get()).is_some() {
                get_number_state.set(GetNumberState::Done)
            };
        });

        // Actual rendering code
        view! {
            cx,
            Router(
                integration=HistoryIntegration::new(),
                view=move |cx, route: &ReadSignal<AppRoutes>| {
                    match route.get().as_ref() {
                        AppRoutes::NotFound => {
                            view! {cx, "Not Found"}
                        }
                        AppRoutes::SubApp(subapp) => {
                            view! {
                                cx,
                                div(class="container is-widescreen"){
                                    NavBar(username=username, is_logged_in=is_logged_in, subapp=subapp.to_string())
                                    Tiles(
                                        subapp=subapp.clone(),
                                        selected_number=selected_number,
                                        get_number_state=get_number_state,
                                        assigned_number=assigned_number,
                                        should_disable_button=should_disable_button,
                                        button_text=button_text,
                                        should_display_abandon_modal=should_display_abandon_modal,
                                        abandoned_numbers=abandoned_numbers,
                                        done_numbers=done_numbers
                                    )
                                    AbandonConfirmationModal(
                                        should_display_abandon_modal=should_display_abandon_modal,
                                        get_number_state=get_number_state,
                                        assigned_number=assigned_number
                                    )
                                }
                            }
                        }
                    }
                }
            )
        }
    })
}

async fn get_json_response<T: de::DeserializeOwned>(url: &str) -> Result<T, Error> {
    Request::get(url).send().await?.json::<T>().await
}

// async fn get_string_response(url: &str) -> Result<String, Error> {
//     Request::get(url).send().await.unwrap().text().await
// }
