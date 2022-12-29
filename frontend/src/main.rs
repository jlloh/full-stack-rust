mod abandon_confirmation_modal;
mod button;
mod navbar;
mod tiles;

use crate::abandon_confirmation_modal::AbandonConfirmationModal;
use crate::button::GetNumberState;
use crate::tiles::Tiles;
use button::TheButton;
use common::UserInfo;
use futures::stream::StreamExt;
use gloo_console::log;
use gloo_net::eventsource::futures::EventSource;
use gloo_net::http::Request;
use gloo_net::Error;
use navbar::NavBar;
use sycamore::futures::*;
use sycamore::prelude::*;

fn main() {
    sycamore::render(|cx| {
        let username = create_signal(cx, "anonymous".to_string());
        let is_logged_in = create_signal(cx, false);
        let get_number_state = create_signal(cx, GetNumberState::NotYetFired);
        let assigned_number = create_signal(cx, None::<i32>);
        let should_display_abandon_modal = create_signal(cx, false);
        // Derived signals
        let should_disable_button = create_memo(cx, || {
            if *is_logged_in.get() {
                match *get_number_state.get() {
                    GetNumberState::NotYetFired => false,
                    GetNumberState::Processing => true,
                    GetNumberState::Done => false,
                    GetNumberState::Failed => false,
                }
            } else {
                true
            }
        });
        let button_text = create_memo(cx, || {
            if *is_logged_in.get() {
                match *get_number_state.get() {
                    GetNumberState::NotYetFired => "Get Number".to_string(),
                    GetNumberState::Processing => "Processing".to_string(),
                    GetNumberState::Done => "Abandon Number".to_string(),
                    GetNumberState::Failed => "Failed".to_string(),
                }
            } else {
                "Please Login to Get Number".to_string()
            }
        });

        spawn_local_scoped(cx, async move {
            let user_info = get_user_info().await;
            match user_info {
                Ok(x) => {
                    username.set(x.email);
                    assigned_number.set(x.assigned_number);
                    if x.assigned_number.is_some() {
                        get_number_state.set(GetNumberState::Done);
                    }
                    is_logged_in.set(x.is_logged_in);
                }
                Err(e) => log!(format!("Failed to call get_user_info2 with error: {}", e)),
            }
        });
        // server sent events
        let live_data = create_signal(cx, "Initialising..".to_string());
        let mut es = EventSource::new("/public/subscribe").unwrap();
        let mut stream_1 = es.subscribe("data").unwrap();
        spawn_local_scoped(cx, async move {
            // weird bug, doesn't work if i don't call es.state() here
            log!(format!("{:#?}", es.state()));
            while let Some(Ok((_event_type, msg))) = stream_1.next().await {
                let x = msg.data().as_string().unwrap();
                live_data.set(x);
            }
        });
        // retrieve some data from /api/hello
        let text = create_signal(cx, "".to_string());
        spawn_local_scoped(cx, async move {
            let result = get_string_response("/api/hello").await;
            if let Ok(x) = result {
                text.set(x)
            } else {
                text.set(format!("{:?}", result))
            }
        });

        // Actual rendering code
        view! {
            cx,
            div(class="container is-widescreen"){
                NavBar(username=username, is_logged_in=is_logged_in)
                Tiles(
                    text=live_data,
                    get_number_state=get_number_state,
                    assigned_number=assigned_number,
                    should_disable_button=should_disable_button,
                    button_text=button_text,
                    should_display_abandon_modal=should_display_abandon_modal
                )
                AbandonConfirmationModal(
                    should_display_abandon_modal=should_display_abandon_modal,
                    get_number_state=get_number_state
                )
            }
        }
    })
}

async fn get_user_info() -> Result<UserInfo, Error> {
    Request::get("/public/get_user_info2")
        .send()
        .await?
        .json()
        .await
}

async fn get_string_response(url: &str) -> Result<String, Error> {
    Request::get(url).send().await.unwrap().text().await
}
