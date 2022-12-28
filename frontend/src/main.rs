// use futures::{stream, StreamExt};
use common::UserInfo;
use futures::stream::StreamExt;
use gloo_console::info;
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
        let get_number_state = create_signal(cx, GetNumberState::NotYetFired);
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
                    Tiles(text=live_data, is_logged_in=is_logged_in, get_number_state=get_number_state)
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
    get_number_state: &'mainbody Signal<GetNumberState>,
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
                        *props.text.get()
                    )}
                    p(class="subtitle"){
                        "Current Queue"
                    }
                }
            }

            div(class="tile is-parent"){
                article(class="tile is-child notification is-primary"){
                    TheButton(props)
                }
            }

        }
    }
}

#[component]
fn TheButton<'mainbody, G: Html>(cx: Scope<'mainbody>, props: TilesProps<'mainbody>) -> View<G> {
    view! {cx,
    (
        if *props.is_logged_in.get() {
            // TODO: This should be a derived state since it's a composite between is_logged_in
            match *props.get_number_state.get() {
                    GetNumberState::NotYetFired =>
                        view! {cx,
                            button(class="button is-large", on:click= move |_| {
                                // spawn_local_scoped(cx, handle_get_number(props.get_number_state));
                                spawn_local_scoped(cx, async move {
                                    (*props.get_number_state).set(GetNumberState::Processing);
                                    let req = match Request::post("/api/get_new_number").send().await {
                                        Ok(response) => {
                                            info!("Done firing getting new number");
                                            response
                                        }
                                        Err(_) => {
                                            info!("Error when firing request to get_number_number endpoint");
                                            (*props.get_number_state).set(GetNumberState::Failed);
                                            return;
                                        }
                                    };
                                    if !req.ok() {
                                        (*props.get_number_state).set(GetNumberState::Failed);
                                        info!("Error when trying to obtain queue number. Check serverside logs")
                                    } else {
                                        info!("Done firing getting new number");
                                        (*props.get_number_state).set(GetNumberState::Done);
                                    }

                                })
                                // (*props.get_number_state).set(GetNumberState::Done);
                            })
                            {
                                "Get Number"
                            }
                        }
                    ,
                    GetNumberState::Processing =>
                        view! {cx,
                            button(class="button is-large", disabled=true){
                                "Processing..."
                            }
                        }
                    ,
                    GetNumberState::Done =>
                        view! {cx,
                            button(class="button is-large", disabled=true){
                                "Number Obtained."
                            }
                        }
                    ,
                    GetNumberState::Failed =>
                        view! {cx,
                            button(class="button is-large", disabled=true){
                                "Failed to obtain number"
                            }
                        }
                    ,
                }
        } else {
            view! {cx,
                button(class="button is-large", disabled=true){
                    "Please Log in to get number"
                }
            }
        }

    )
    }
}

enum GetNumberState {
    NotYetFired,
    Processing,
    Done,
    Failed,
}

// async fn handle_test() {
//     info!("Test test");
//     let req_result = Request::post("/api/get_new_number").send().await;
//     info!("Test2 test2");
// }

async fn handle_get_number(get_number_state: &Signal<GetNumberState>) {
    (*get_number_state).set(GetNumberState::Processing);
    info!("Test test");
    // let req_result = Request::post("/api/get_new_number").send().await;
    // if req_result.is_err() {
    //     info!("Error when firing request to get_number_number endpoint");
    //     (*get_number_state).set(GetNumberState::Failed);
    // }

    let req = match Request::post("/api/get_new_number").send().await {
        Ok(response) => {
            info!("Done firing getting new number");
            response
        }
        Err(_) => {
            // TODO: Error handling does not work because future is aborted before this can be called
            info!("Error when firing request to get_number_number endpoint");
            (*get_number_state).set(GetNumberState::Failed);
            return;
        }
    };
    if !req.ok() {
        (*get_number_state).set(GetNumberState::Failed);
        info!("Error when trying to obtain queue number. Check serverside logs")
    } else {
        info!("Done firing getting new number");
        (*get_number_state).set(GetNumberState::Done);
    }
}

async fn async_request() {
    let request = Request::post("/api/get_new_number").send().await;
}

// #[component]
// async fn AsyncComponent<'asynccomponent, G: Html>(cx: Scope<'asynccomponent>) -> View<G> {
//     view! {cx,
//     }
// }
