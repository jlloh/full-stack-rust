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
        Some(Ok(val)) => rsx!(
            div {
                class: "container",
                NavBar {},
                div {
                    class: "columns",
                    PanelComponent {
                        val: val.to_string()
                    }
                    PanelComponent {
                        val: "static value".to_string()
                    }
                }
            }
        ),
        Some(Err(_)) => rsx!("Failed to call api"),
        None => rsx!("Loading api"),
    })
}

#[allow(non_snake_case)]
#[inline_props]
fn PanelComponent(cx: Scope, val: String) -> Element {
    cx.render(rsx! {
        div {
            class: "col-6 col-xs-12",
            div {
                class: "panel",
                div {
                    class: "panel-header text-center",
                    div {
                        class: "panel-title h3",
                        "This is a panel title"
                    }
                }
                div {
                    class: "panel-body",
                    h4 {
                        "This is a header"
                    }
                    p {
                        "Retrieved value: {val}"
                    }
                }
            }
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
fn NavBar(cx: Scope) -> Element {
    let nav_items = vec!["home", "about"].into_iter().map(|x| {
        rsx!(
            a {
                    class: "btn btn-link",
                    href: "#",
                    "{x}"
            }
        )
    });
    let nav_logo = rsx!(
        a {
            class: "navbar-brand mr-2",
            href: "#",
            "Logo",
        }
    );
    cx.render(rsx! {
        header {
            class: "navbar",
            section{
                class: "navbar-section",
                nav_logo
                nav_items
            }
        }
    })
}
