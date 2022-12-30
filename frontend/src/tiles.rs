use crate::button::GetNumberState;
use crate::Panel;
use crate::TheButton;
use sycamore::prelude::*;

#[derive(Prop)]
pub struct TilesProps<'mainbody> {
    pub subapp: String,
    pub selected_number: &'mainbody Signal<Option<i32>>,
    pub should_disable_button: &'mainbody ReadSignal<bool>,
    pub button_text: &'mainbody ReadSignal<String>,
    pub get_number_state: &'mainbody Signal<GetNumberState>,
    pub assigned_number: &'mainbody Signal<Option<i32>>,
    pub should_display_abandon_modal: &'mainbody Signal<bool>,
    pub abandoned_numbers: &'mainbody Signal<Vec<i32>>,
    pub done_numbers: &'mainbody Signal<Vec<i32>>,
}

#[component]
pub fn Tiles<'mainbody, G: Html>(cx: Scope<'mainbody>, props: TilesProps<'mainbody>) -> View<G> {
    view! {
        cx,
        div(class="tile is-ancestor"){

            div(class="tile is-parent"){
                article(class="tile is-child notification is-info"){
                    p(class="title"){
                        "Queueing App for: " (props.subapp)
                    }
                    p(class="subtitle"){
                        "Welcome to the queueing app!"
                    }
                }
            }

            div(class="tile is-parent"){
                article(class="tile is-child notification is-warning"){
                    p(class="title"){(
                        if let Some(number)= *props.selected_number.get() {
                            number.to_string()
                        } else {
                            "None".to_string()
                        }
                    )}
                    p(class="subtitle"){
                        "Current Queue"
                    }
                }
            }

            div(class="tile is-parent is-vertical"){
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
                    TheButton(
                        should_disable_button=props.should_disable_button,
                        button_text=props.button_text,
                        get_number_state=props.get_number_state,
                        assigned_number=props.assigned_number,
                        should_display_abandon_modal=props.should_display_abandon_modal
                    )
                }
                    Panel(abandoned_numbers=props.abandoned_numbers, done_numbers=props.done_numbers)
            }

            // div(class="tile is-parent"){
            //     Panel()
            // }

        }
    }
}
