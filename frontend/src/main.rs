use gloo_net::http::Request;
use sycamore::futures::*;
use sycamore::prelude::*;

fn main() {
    sycamore::render(|cx| {
        let username = create_signal(cx, "unknown".to_string());
        let is_logged_in = create_selector(cx, || is_logged_in(username.get().to_string()));
        //TODO: Move this into it's own function and don't inline this
        spawn_local_scoped(cx, async move {
            let user_info = Request::get("/api/get_user_info")
                .send()
                .await
                .unwrap()
                .text()
                .await;
            if let Ok(x) = user_info {
                username.set(x)
            } else {
                username.set("unknown".to_string())
            };
        });
        view! {
            cx,
            div(){
                NavBar(username=username, is_logged_in=is_logged_in)
                div(class="container is-widescreen"){
                    MainBody()
                }
            }
        }
    })
}

#[derive(Prop)]
struct NavBarProps<'navbar> {
    username: &'navbar Signal<String>,
    is_logged_in: &'navbar ReadSignal<bool>,
}

#[component]
fn NavBar<'navbar, G: Html>(cx: Scope<'navbar>, props: NavBarProps<'navbar>) -> View<G> {
    view! {cx,
        nav(class="navbar",role="navigation"){
            div(class="navbar-brand"){
                a(class="navbar-item", href="#"){
                    // img(src="https://bulma.io/images/bulma-logo.png")
                    img(src="./rust_logo.png")
                }
            }
            NavBarEndMenu(username=props.username, is_logged_in=props.is_logged_in)
        }
    }
}

#[component]
fn NavBarEndMenu<'navbar, G: Html>(cx: Scope<'navbar>, props: NavBarProps<'navbar>) -> View<G> {
    view! {cx,
        div(class="navbar-end"){
            (if *props.is_logged_in.get() {
                view! {cx,
                div(class="navbar-item has-dropdown is-hoverable"){
                    a(class="navbar-link", href="#"){
                        "User:" (*props.username.get())
                    }
                    div(class="navbar-dropdown"){
                        a(class="navbar-item", href="/api/trigger_logout"){
                            "Logout"
                        }
                    }
                }}
            } else {
                view! {cx,
                    a(class="button is-primary", href="/api/trigger_login"){"Login"}
                }
            })
        }
    }
}

/// Checked whether email is a valid email
fn is_logged_in(user_info: String) -> bool {
    vec!["anonymous", "unknown"]
        .into_iter()
        .filter(|x| *x == user_info)
        .count()
        == 0
}

#[component]
fn MainBody<G: Html>(cx: Scope) -> View<G> {
    view! {cx,
        section(class="hero is-primary"){
            div(class="hero-body"){
                p(class="title"){
                    "Wasm Website"
                }
                p(class="subtitle"){
                    "Written using Sycamore for frontend, Actix for the backend and web, Bulma for CSS."
                }
            }
        }
    }
}
