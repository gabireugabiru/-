use leptos::*;
use serde::Serialize;
use wasm_bindgen::UnwrapThrowExt;

use crate::invoke::invokers;

#[component]
pub fn ButtonLink(url: String, text: String) -> impl IntoView {
    view! {
        <button on:click=move |_| {
            let url = url.clone();
            spawn_local(async move {
                #[derive(Serialize)]
                struct T {
                    url: String
                }
                invokers::<T, bool>("open_url", T {
                    url: url.clone()
                }).await.expect_throw("Cannot open link");
            });
        }>
            {text}
        </button>
    }
}