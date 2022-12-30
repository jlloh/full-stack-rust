use sycamore::builder::prelude::*;
use sycamore::prelude::*;

#[derive(Prop)]
pub struct PanelProps<'panel> {
    abandoned_numbers: &'panel ReadSignal<Vec<i32>>,
    done_numbers: &'panel ReadSignal<Vec<i32>>,
}

#[component]
pub fn Panel<'panel, G: Html>(cx: Scope<'panel>, props: PanelProps<'panel>) -> View<G> {
    view! {
        cx,
        article(class="panel is-primary tile is-child"){
            p(class="panel-heading"){
                "Past Numbers"
            }
            p(class="panel-tabs"){
                a(class="is-active"){"All"}
                a(){"Done"}
                a(){"Abandoned"}
            }
            Keyed(
                iterable=props.abandoned_numbers,
                view=|cx, x| view! {cx, a(class="panel-block"){(x)}},
                key=|x| *x
            )
            Keyed(
                iterable=props.done_numbers,
                view=|cx, x| view! {cx, a(class="panel-block"){(x)}},
                key=|x| *x
            )
        }
    }
}

#[component]
pub fn PanelBuilder<'panel, G: Html>(cx: Scope<'panel>, props: PanelProps<'panel>) -> View<G> {
    let root = article()
        .class("panel is-primary tile is-child")
        .c(p().class("panel-heading").t("Past Numbers"));
    let with_panels = root.c(p()
        .class("panel-tabs")
        .c(a().class("is-active").t("All"))
        .c(a().t("Done"))
        .c(a().t("Abandoned")));
    let abandoned: Vec<View<G>> = (*props.abandoned_numbers.get())
        .iter()
        .map(move |x| {
            let x_1 = *x;
            a().class("panel-block")
                .dyn_t(move || x_1.to_string())
                .view(cx)
        })
        .collect();
    let done: Vec<View<G>> = (*props.done_numbers.get())
        .iter()
        .map(move |x| {
            let x_1 = *x;
            a().class("panel-block")
                .dyn_t(move || x_1.to_string())
                .view(cx)
        })
        .collect();
    with_panels
        .c(View::new_fragment(abandoned))
        .c(View::new_fragment(done))
        .view(cx)
}
