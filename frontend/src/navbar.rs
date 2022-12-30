use sycamore::builder::prelude::*;
use sycamore::prelude::*;

#[derive(Prop, Clone)]
pub struct NavBarProps<'navbar> {
    username: &'navbar Signal<String>,
    is_logged_in: &'navbar ReadSignal<bool>,
    subapp: String,
}

#[component]
pub fn NavBar<'navbar, G: Html>(cx: Scope<'navbar>, props: NavBarProps<'navbar>) -> View<G> {
    view! {
        cx,
        nav(class="navbar",role="navigation"){
            div(class="navbar-brand"){
                a(class="navbar-item", href="#"){
                    img(src="./rust_logo.png")
                }
            }
            NavBarEndMenu(props)
        }
    }
}

#[component]
fn NavBarEndMenu<'navbar, G: Html>(cx: Scope<'navbar>, props: NavBarProps<'navbar>) -> View<G> {
    let subapp = props.subapp;
    let login_url = format!("/public/{}/trigger_login", &subapp);
    let logout_url = format!("/api/{}/trigger_logout", &subapp);
    let root = div().class("navbar-end");
    root.dyn_if(
        || *props.is_logged_in.get(),
        move || {
            let logout_url = logout_url.clone();
            div()
                .class("navbar-item has-dropdown is-hoverable")
                .c(a()
                    .class("navbar-link")
                    .attr("href", "#")
                    .dyn_t(|| props.username.get().to_string()))
                .c(div().class("navbar-dropdown").c(a()
                    .class("navbar-item")
                    .dyn_attr("href", move || Some(logout_url.clone()))
                    .attr("rel", "external")
                    .t("Logout")))
        },
        move || {
            let login_url = login_url.clone();
            a().class("button is-black")
                .dyn_attr("href", move || Some(login_url.clone()))
                .attr("rel", "external")
                .t("Login")
        },
    )
    .view(cx)
}

// #[component]
// fn NavBarEndMenuDsl<'navbar, G: Html>(cx: Scope<'navbar>, props: NavBarProps<'navbar>) -> View<G> {
//     let subapp = props.subapp;
//     let login_url = format!("/public/{}/trigger_login", &subapp);
//     let logout_url = format!("/api/{}/trigger_logout", &subapp);
//     view! {
//         cx,
//         div(class="navbar-end"){
//             (
//                 if *props.is_logged_in.get() {
//                 view! {cx,
//                     div(class="navbar-item has-dropdown is-hoverable"){
//                         a(class="navbar-link", href="#"){
//                             "User:" (*props.username.get())
//                         }
//                         div(class="navbar-dropdown"){
//                             a(class="navbar-item", href=logout_url.clone(), rel="external"){
//                                 "Logout"
//                             }
//                         }
//                     }
//                 }
//             } else {
//                 view! {cx,
//                     a(class="button is-black", href="/public/trigger_login", rel="external"){"Login"}
//                 }
//             })
//         }
//     }
// }
