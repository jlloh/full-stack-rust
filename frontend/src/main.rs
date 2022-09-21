use dioxus::prelude::*;
use gloo_net::http::Request;

fn main() {
    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let x = use_future(&cx, (), |_| async move {
        Request::get("/hello").send().await.unwrap().text().await
    });

    cx.render(match x.value() {
        Some(Ok(val)) => rsx!(HeaderComponent {
            val: val.to_string()
        }),
        Some(Err(_)) => rsx!("Failed to call api"),
        None => rsx!("Loading api"),
    })
}

#[derive(Props, PartialEq)]
struct HeaderComponentProps {
    val: String,
}

#[allow(non_snake_case)]
fn HeaderComponent(cx: Scope<HeaderComponentProps>) -> Element {
    cx.render(rsx! {
        div{
            article {
                class: "mw7 center ph3 ph5-ns tc br2 pv5 bg-washed-yellow dark-blue mb5",
                h1{
                    class: "fw6 f3 f2-ns lh-title mt0 mb3",
                    "Retrieved value: {cx.props.val}"
                }
                h2 {
                    class: "fw2 f4 lh-copy mt0 mb3",
                    "This a sub header"
                }
            }
        }
    })
}
