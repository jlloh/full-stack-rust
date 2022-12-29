use crate::button::GetNumberState;
use crate::TheButton;
use sycamore::prelude::*;

#[derive(Prop)]
pub struct TilesProps<'mainbody> {
    pub text: &'mainbody Signal<String>,
    pub should_disable_button: &'mainbody ReadSignal<bool>,
    pub button_text: &'mainbody ReadSignal<String>,
    pub get_number_state: &'mainbody Signal<GetNumberState>,
    pub assigned_number: &'mainbody Signal<Option<i32>>,
    pub should_display_abandon_modal: &'mainbody Signal<bool>,
}

#[component]
pub fn Tiles<'mainbody, G: Html>(cx: Scope<'mainbody>, props: TilesProps<'mainbody>) -> View<G> {
    view! {
        cx,
        div(class="tile is-ancestor"){
            div(class="tile is-parent"){
                article(class="tile is-child notification is-info"){
                    p(class="title"){
                        "Queueing App"
                    }
                    p(class="subtitle"){
                        "Welcome to the queueing app!"
                    }
                }
            }

            div(class="tile is-parent"){
                article(class="tile is-child notification is-warning"){
                    p(class="title"){(
                        *props.text.get()
                    )}
                    p(class="subtitle"){
                        "Current Queue"
                    }
                }
            }

            div(class="tile is-parent"){
                article(class="tile is-child notification is-primary"){
                    p(class="title"){(
                        if let Some(number) = *props.assigned_number.get() {
                            format!("{}", number)
                        } else {
                            "None Assigned".to_string()
                        }
                    )}
                    p(class="subtitle"){
                        "Your Number"
                    }
                    TheButton(props)
                }
            }

        }
    }
}
