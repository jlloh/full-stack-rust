// use futures::{stream, StreamExt};
use common::UserInfo;
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
        let username = create_signal(cx, "anonymous".to_string());
        let is_logged_in = create_signal(cx, false);
        spawn_local_scoped(cx, async move {
            let user_info = get_user_info().await;
            // can't do error propagation in spawn_local_scoped?
            match user_info {
                Ok(x) => {
                    username.set(x.email);
                    is_logged_in.set(x.is_logged_in);
                }
                Err(e) => log!(format!("Failed to call get_user_info2 with error: {}", e)),
            }
        });
        // server sent events
        let live_data = create_signal(cx, "Initialising..".to_string());
        let mut es = EventSource::new("/public/subscribe").unwrap();
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
                div(class="container is-widescreen"){
                    NavBar(username=username, is_logged_in=is_logged_in)
                    Tiles(text=live_data, is_logged_in=is_logged_in)
                }
            }
        }
    })
}

async fn get_user_info() -> Result<UserInfo, Error> {
    Request::get("/public/get_user_info2")
        .send()
        .await?
        .json()
        .await
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
                    a(class="button is-black", href="/public/trigger_login"){"Login"}
                }
            })
        }
    }
}

// /// Checked whether email is a valid email
// fn is_logged_in(user_info: String) -> bool {
//     vec!["anonymous", "unknown"]
//         .into_iter()
//         .filter(|x| *x == user_info)
//         .count()
//         == 0
// }

#[derive(Prop)]
struct TilesProps<'mainbody> {
    text: &'mainbody Signal<String>,
    is_logged_in: &'mainbody ReadSignal<bool>,
}

#[component]
fn Tiles<'mainbody, G: Html>(cx: Scope<'mainbody>, props: TilesProps<'mainbody>) -> View<G> {
    view! {cx,
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
                        view! {cx, (*props.text.get())}
                    )}
                    p(class="subtitle"){
                        "Current Queue"
                    }
                }
            }

            div(class="tile is-parent"){
                article(class="tile is-child notification is-primary"){
                    (
                        if *props.is_logged_in.get() {
                            view! {cx, button(class="button is-large"){
                                "Get Number"
                            }}
                        } else {
                            view! {cx, button(class="button is-large", disabled=true){
                                "Please Log in to get number"
                            }}
                        }
                    )
                }
            }

        }
    }
}

// #[component]
// async fn AsyncComponent<'asynccomponent, G: Html>(cx: Scope<'asynccomponent>) -> View<G> {
//     view! {cx,
//     }
// }
