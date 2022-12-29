use common::UserInfo;
use gloo_console::info;
use gloo_net::http::Request;
use sycamore::futures::*;
use sycamore::prelude::*;

use crate::tiles::TilesProps;

#[component]
pub fn TheButton<'mainbody, G: Html>(
    cx: Scope<'mainbody>,
    props: TilesProps<'mainbody>,
) -> View<G> {
    view! {
        cx,
        button(
            class="button is-large is-light",
            disabled=*props.should_disable_button.get(),
            on:click=move|_| {
                spawn_local_scoped(cx, handle_get_number(props.get_number_state, props.assigned_number, props.should_display_abandon_modal));
            }
        )
        {
            (*props.button_text.get())
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum GetNumberState {
    NotYetFired,
    Processing,
    Done,
    Failed,
}

async fn handle_get_number(
    get_number_state: &Signal<GetNumberState>,
    assigned_number: &Signal<Option<i32>>,
    should_display_abandon_modal: &Signal<bool>,
) {
    // Abandon number flow. Toggle a modal?
    if (*get_number_state.get()) == GetNumberState::Done {
        (*should_display_abandon_modal).set(true)
    }

    // Get number flow
    (*get_number_state).set(GetNumberState::Processing);
    let req = match Request::post("/api/get_new_number").send().await {
        Ok(response) => {
            info!("Done firing getting new number");
            response
        }
        Err(_) => {
            info!("Error when firing request to get_number_number endpoint");
            (*get_number_state).set(GetNumberState::Failed);
            return;
        }
    };
    if !req.ok() {
        (*get_number_state).set(GetNumberState::Failed);
        info!("Error when trying to obtain queue number. Check serverside logs")
    } else {
        info!("Done firing getting new number");
        if let Ok(x) = req.json::<UserInfo>().await {
            info!("Successfully assigned number for {}", x.email);
            (*assigned_number).set(x.assigned_number);
            (*get_number_state).set(GetNumberState::Done);
        } else {
            info!("Error unmarshaling UserInfo struct");
            (*get_number_state).set(GetNumberState::Failed);
        };
    }
}
