mod app;
mod ask;
mod button_link;
mod home;
mod invoke;
mod kanji_info;
mod learningkanji;
mod radical_info;
mod svgs;
mod utils;
mod vocab_info;
use app::*;
use leptos::*;
use leptos_router::Router;

fn main() {
    mount_to_body(|| {
        view! {
            <Router>
                <App/>
            </Router>
        }
    })
}
