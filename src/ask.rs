// use leptos::*;
// use wana_kana::{ConvertJapanese, IsJapaneseStr};
// use wasm_bindgen::UnwrapThrowExt;
// use web_sys::{HtmlFormElement, SubmitEvent};

// fn contain_lowercased(a: &Vec<String>, value: String, is_kana: bool) -> bool {
//     for i in a {
//         let mut x = i.clone();
//         if i.starts_with("!") || i.starts_with("^") {
//             x = i[1..i.len()].to_string();
//         }
//         if is_kana {
//             if x.to_lowercase() == value.to_lowercase().to_hiragana() {
//                 return true
//             }
//         } else {
//             if x.to_lowercase() == value.to_lowercase() {
//                 return true
//             }
//         }

//     }
//     false
// }

// #[component]
// pub fn Ask<T>(question: String, answer: Vec<String>, alert: Vec<String>, when_answer: T, is_kana: bool, kun: bool) -> impl IntoView
// where T: Fn(bool) + 'static
// {
//     let is_correct = create_rw_signal(None::<bool>);
//     let text = create_rw_signal(String::default());
//     if answer.len() == 0 {
//         when_answer(true);
//         is_correct.set(Some(true));
//     }
//     let a = answer.iter().map(|a| view! {<span> {a.clone()} </span>}).collect_view();
//     view! {
//         <form class:yes=move || is_correct.get().unwrap_or(false) class:no=move || is_correct.get().map(|a| !a).unwrap_or(false) on:submit=move |ev| {
//             ev.prevent_default();
//             if !is_kana && text.get().is_hiragana() {
//                 window().alert_with_message("This input is the meaning").unwrap_throw();
//                 return
//             }
//             if !is_kana && contain_lowercased(&alert, text.get(), true) {
//                 window().alert_with_message("The meaning not the reading").unwrap_throw();
//                 return
//             }
//             if contain_lowercased(&alert, text.get(), is_kana) {
//                 window().alert_with_message(if kun {
//                     "The onyoumi reading"
//                 } else {
//                     "The kunyoumi reading"
//                 }).unwrap_throw();
//                 return
//             }
//             if is_correct.get().is_none() {
//                 let ans = contain_lowercased(&answer, text.get(), is_kana);
//                 is_correct.set(Some(ans));
//                 when_answer(ans);
//             }
//         }>
//             <span>
//                 {question}
//             </span>
//             <input prop:disabled=move || is_correct.get().is_some() type="text" prop:value=move || text.get() on:change=move |ev| text.set(event_target_value(&ev)) />
//             {is_kana.then(|| {
//                 view! {
//                     <span>{move || text.get().to_hiragana()}</span>
//                 }
//             })}
//             <button type="submit">
//                 {"next"}
//             </button>
//             {move || {
//                 is_correct.get().is_some().then(|| a.clone())
//             }}
//         </form>
//     }
// }
