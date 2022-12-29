use gloo_console::info;
use gloo_net::http::Request;
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::button::GetNumberState;

#[derive(Prop)]
pub struct AbandonConfirmationModalProps<'mainbody> {
    should_display_abandon_modal: &'mainbody Signal<bool>,
    get_number_state: &'mainbody Signal<GetNumberState>,
}

#[component]
pub fn AbandonConfirmationModal<'mainbody, G: Html>(
    cx: Scope<'mainbody>,
    props: AbandonConfirmationModalProps<'mainbody>,
) -> View<G> {
    view! {
        cx,
        div(class=(
                if *props.should_display_abandon_modal.get() {
                    "modal is-active"
                } else {
                    "modal"
                }
            )
        ){
            div(class="modal-background", on:click=|_|{(*props.should_display_abandon_modal).set(false)}){}
            div(class="modal-card"){
                section(class="modal-card-body"){
                    div(class="notification is-danger"){
                        "Are you sure you want to abandon your queue?"
                    }
                }
                footer(class="modal-card-foot"){
                    button(class="button is-danger", on:click=move|_|{spawn_local_scoped(cx, handle_abandon_number(props.should_display_abandon_modal, props.get_number_state))}){
                        "Yes"
                    }
                }
            }
        }
    }
}

async fn handle_abandon_number(
    should_display_abandon_modal: &Signal<bool>,
    get_number_state: &Signal<GetNumberState>,
) {
    let _req = match Request::post("/api/abandon_assigned_number").send().await {
        Ok(response) => response,
        Err(_) => {
            info!("Error when firing request to abandon endpoint");
            return;
        }
    };
    (*get_number_state).set(GetNumberState::NotYetFired);
    (*should_display_abandon_modal).set(false);
}
