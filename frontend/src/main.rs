use dioxus::prelude::*;
use gloo_net::http::Request;

fn main() {
    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let user_info = use_future(&cx, (), |_| async move {
        Request::get("/api/get_user_info")
            .send()
            .await
            .unwrap()
            .text()
            .await
    });

    cx.render(match user_info.value() {
        Some(Ok(user_info)) =>
        // let logged_in = is_logged_in(user_info.to_string());
        {
            let logged_in = is_logged_in(user_info.to_string());
            rsx!(
                div {
                    class: "container",
                    NavBar {user_info: user_info.to_string(), logged_in: logged_in}
                    div {
                        class: "columns",
                        PanelComponent {
                            val: user_info.to_string()
                        }
                        PanelComponent {
                            val: "static value".to_string()
                        }
                    }
                }
            )
        }
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

/// Checked whether email is a valid email
fn is_logged_in(user_info: String) -> bool {
    vec!["anonymous", "unknown"]
        .into_iter()
        .filter(|x| *x == user_info)
        .count()
        == 0
}

#[allow(non_snake_case)]
#[inline_props]
fn NavBar(cx: Scope, user_info: String, logged_in: bool) -> Element {
    // let logged_in = is_logged_in(user_info.to_string());
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
    let nav_login = if !logged_in {
        rsx!(
            a {
                class: "btn btn-primary",
                href: "/api/trigger_login",
                "Login",
            }
        )
    } else {
        rsx!(div {
            class: "dropdown",
            a {
                class: "btn btn-link dropdown-toggle",
                href: "#",
                "User: {user_info}"
                i {
                    class: "icon icon-caret"
                }
            }
            ul {
                class: "menu",
                li {
                    class: "menu-item",
                    a {
                        href: "/api/trigger_logout",
                        "Logout"
                    }
                }
            }
        })
    };
    cx.render(rsx! {
        header {
            class: "navbar",
            section {
                class: "navbar-section",
                nav_logo
                nav_items
            }
            section {
                class: "navbar-section",
                nav_login
            }
        }
    })
}
