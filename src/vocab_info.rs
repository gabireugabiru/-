use crate::button_link::ButtonLink;
use crate::has_acess_logic;
use crate::invoke::get_kanjis;
use crate::invoke::invokers;
use crate::utils::local_set;
use crate::ViewedContext;
use crate::LIMIT;
use chrono::Datelike;
use leptos::*;
use leptos_router::use_params;
use leptos_router::Params;
use serde::Deserialize;
use serde::Serialize;
use wana_kana::ConvertJapanese;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::MouseEvent;

#[derive(Clone, PartialEq, Eq, Params)]
pub struct VocabParams {
    vocab: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VocabFullInfo {
    pub meanings: Vec<String>,
    pub wk_level: Option<u32>,
    pub readings: Vec<String>,
    pub primary_reading: String,
    pub primary_meaning: String,
    pub another_form: Vec<String>,
}

#[component]
pub fn VocabInfo() -> impl IntoView {
    let (viewed, set_viewed) =
        use_context::<ViewedContext>().expect_throw("expected viewed context");
    let params = use_params::<VocabParams>();
    let (times_learned, set_times_learned) =
        use_context::<(Signal<u32>, WriteSignal<u32>)>().expect_throw("times learned context");
    let vocab = move || match params.get() {
        Ok(a) => a.vocab,
        _ => None,
    };

    let info = create_resource(
        move || vocab(),
        move |a| async move {
            if let Some(vocab) = a {
                #[derive(Serialize)]
                pub struct T {
                    vocab: String,
                }

                let res = invokers::<T, Option<VocabFullInfo>>("get_vocab", T { vocab })
                    .await
                    .unwrap_throw();

                return res;
            }
            None
        },
    );
    let has_acess = create_resource(
        move || (viewed.get(), info.get()),
        move |(viewed, info)| async move {
            let level = match info {
                Some(Some(a)) if a.wk_level.is_some() => a.wk_level.unwrap() - 1,
                _ => return false,
            };
            if level == 0 {
                let Some(kanjis) = get_kanjis(1).await else {
                    return false;
                };
                has_acess_logic(1, &Some(kanjis), &viewed)
                    .kanji
                    .partial_or_more()
            } else {
                let Some(kanjis) = get_kanjis(level as usize).await else {
                    return false;
                };
                let has_all = has_acess_logic(level as usize, &Some(kanjis), &viewed).has_all();
                let Some(kanjis_current) = get_kanjis(level as usize + 1).await else {
                    return false;
                };
                has_acess_logic(level as usize + 1, &Some(kanjis_current), &viewed)
                    .kanji
                    .partial_or_more()
                    && has_all
            }
        },
    );
    let add_to_learned = move |_: MouseEvent| {
        let Some(level) = info.with(|a| {
            match a.as_ref() {
                Some(Some(a)) => Some(a.wk_level.map(|a| a - 1)),
                _ => None,
            }
            .flatten()
        }) else {
            return;
        };
        if !has_acess.get().unwrap_or(false) {
            return;
        }
        set_viewed.update(|viewed| {
            let Some(level_viewed) = viewed.levels.get_mut(level as usize) else {
                return;
            };
            let vocab = vocab().unwrap_or_default();
            if vocab.is_empty() || level_viewed.vocabs.iter().find(|a| a.0 == vocab).is_some() {
                return;
            }
            if times_learned.get() > LIMIT {
                return;
            }
            let now = chrono::offset::Local::now();
            local_set("last_learned".to_string(), now.date_naive().day());
            set_times_learned.update(|a| {
                *a = *a + 1;
            });
            level_viewed.vocabs.push((vocab, 0));
        })
    };
    let has_learned = create_memo(move |_| {
        with!(move |viewed, info| {
            let level = match info {
                Some(Some(a)) if a.wk_level.is_some() => a.wk_level.unwrap() - 1,
                _ => return false,
            };
            let Some(vocab) = vocab() else {
                return false;
            };
            viewed
                .levels
                .get(level as usize)
                .map(|a| a.vocabs.iter().find(|a| a.0 == vocab).is_some())
                .unwrap_or_default()
        })
    });
    let info = move || info.get().flatten();
    view! {
        <section class="info vocab">
            <header>
                <h1>
                    {vocab}
                </h1>
                {move || info().map(|a| view! {
                    {a.wk_level.map(|a| view! {
                    <div class="square">
                        Level
                        <span> {a} </span>
                    </div>
                    })}
                })}

            </header>

            {move || info().map(|info| {
                view! {
                    <div class="meanings">
                        <div>
                            <h3>
                                Meaning
                            </h3>
                            <span>
                                - {info.primary_meaning.clone()}
                            </span>
                            {info.meanings.iter().take(8).flat_map(move |a|
                                (a.trim().to_lowercase() != info.primary_meaning.trim().to_lowercase()).then(|| view! {
                                    <span>
                                        - {a}
                                    </span>
                                })
                            ).collect_view()}
                        </div>
                        <div>
                            <h3>
                                Reading
                            </h3>
                            <span title=info.primary_reading.to_romaji()>
                                - {info.primary_reading.clone()}
                            </span>
                            {info.readings.iter().flat_map(move |a|
                                (a.trim() != info.primary_reading.trim()).then(|| view! {
                                    <span title=a.to_romaji()>
                                        - {a}
                                    </span>
                                })
                            ).collect_view()}
                        </div>
                    </div>
                    {move || (!info.another_form.is_empty()).then(|| view! {
                        <div class="meanings">
                        <div>
                            <h3>
                                Other forms
                            </h3>
                            {info.another_form.iter().map(|a| view! {
                                <span>
                                 - {a}
                                </span>
                            }).collect_view()}
                        </div>
                    </div>
                    })}

                }
            })}

            {move || vocab().map(|vocab| view! {
                <section class="other_sources">
                    <h3>
                        Other sources for yo studying
                    </h3>
                    <div>
                        <ButtonLink
                            url=format!("https://www.wanikani.com/vocabulary/{}", vocab)
                            text="WaniKani".to_string()
                        />
                        <ButtonLink
                            url=format!("https://www.kanshudo.com/search?q={}", vocab)
                            text="Kashudo".to_string()
                        />
                    </div>
                </section>
            })}
            {move || has_acess.get().unwrap_or(false).then(|| view! {
                <footer>
                    <button class="learn" class:learned=has_learned.get() prop:disabled=has_learned.get() on:click=add_to_learned>
                        "I am confident with this radical"
                    </button>
                </footer>
            })}

        </section>
    }
}
