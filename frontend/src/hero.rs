use sycamore::prelude::*;

#[derive(Prop, Clone)]
pub struct HeroProps {
    title: String,
    subtitle: String,
}

#[component]
pub fn Hero<G: Html>(cx: Scope, props: HeroProps) -> View<G> {
    view! {
        cx,
        section(class="hero is-primary"){
            div(class="hero-body"){
                p(class="title"){
                    (props.title)
                }
                p(class="subtitle"){
                    (props.subtitle)
                }
            }
        }
    }
}
