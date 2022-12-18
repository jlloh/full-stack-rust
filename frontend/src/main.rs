// use futures::{stream, StreamExt};
use futures::stream::StreamExt;
use gloo_console::log;
use gloo_net::eventsource::futures::EventSource;
use gloo_net::http::Request;
use gloo_net::Error;
use sycamore::futures::*;
use sycamore::prelude::*;
// use sycamore::suspense::Suspense;

fn main() {
    sycamore::render(|cx| {
        let anonymous_user = "anonymous".to_string();
        let username = create_signal(cx, anonymous_user.clone());
        let is_logged_in = create_selector(cx, || is_logged_in(username.get().to_string()));
        spawn_local_scoped(cx, async move {
            let user_info = get_string_response("/api/get_user_info").await;
            if let Ok(x) = user_info {
                username.set(x)
            } else {
                username.set(anonymous_user)
            };
        });
        // server sent events
        let live_data = create_signal(cx, "None".to_string());
        let mut es = EventSource::new("/api/subscribe").unwrap();
        let mut stream_1 = es.subscribe("data").unwrap();
        spawn_local_scoped(cx, async move {
            // weird bug, doesn't work if i don't call es.state() here
            log!(format!("{:#?}", es.state()));
            while let Some(Ok((_event_type, msg))) = stream_1.next().await {
                let x = msg.data().as_string().unwrap();
                live_data.set(x);
            }
        });
        // retrieve some data from /api/hello
        let text = create_signal(cx, "".to_string());
        spawn_local_scoped(cx, async move {
            let result = get_string_response("/api/hello").await;
            if let Ok(x) = result {
                text.set(x)
            } else {
                text.set(format!("{:?}", result))
            }
        });
        view! {
            cx,
            div(){
                NavBar(username=username, is_logged_in=is_logged_in)
                div(class="container is-widescreen"){
                    MainBody(text=live_data, is_logged_in=is_logged_in)
                }
            }
        }
    })
}

async fn get_string_response(url: &str) -> Result<String, Error> {
    Request::get(url).send().await.unwrap().text().await
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

#[derive(Prop)]
struct MainBodyProps<'mainbody> {
    text: &'mainbody Signal<String>,
    is_logged_in: &'mainbody ReadSignal<bool>,
}

#[component]
fn MainBody<'mainbody, G: Html>(cx: Scope<'mainbody>, props: MainBodyProps<'mainbody>) -> View<G> {
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
        div(class="box"){
            (if *props.is_logged_in.get() {
                view! {cx, (*props.text.get())}
            } else {
                view! {cx, "Welcome anonymous user"}
            }
            )}
    }
}

// #[component]
// async fn AsyncComponent<'asynccomponent, G: Html>(cx: Scope<'asynccomponent>) -> View<G> {
//     view! {cx,
//     }
// }
