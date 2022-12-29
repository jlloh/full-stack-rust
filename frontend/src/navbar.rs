use sycamore::prelude::*;

#[derive(Prop)]
pub struct NavBarProps<'navbar> {
    username: &'navbar Signal<String>,
    is_logged_in: &'navbar ReadSignal<bool>,
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
            NavBarEndMenu(username=props.username, is_logged_in=props.is_logged_in)
        }
    }
}

#[component]
fn NavBarEndMenu<'navbar, G: Html>(cx: Scope<'navbar>, props: NavBarProps<'navbar>) -> View<G> {
    view! {
        cx,
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
                    }
                }
            } else {
                view! {cx,
                    a(class="button is-black", href="/public/trigger_login"){"Login"}
                }
            })
        }
    }
}
