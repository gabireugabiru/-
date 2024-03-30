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

pub fn dislice(v: &String) -> String {
    if v.starts_with("^") || v.starts_with("!") {
        v[1..v.len()].to_string()
    } else {
        v.to_owned()
    }
}

#[derive(Clone, PartialEq, Eq, Params)]
pub struct KanjiParams {
    kanji: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct KanjiInfo {
    pub strokes: u32,
    pub freq: Option<u32>,
    pub wk_meanings: Vec<String>,
    pub wk_readings_on: Vec<String>,
    pub wk_readings_kun: Vec<String>,
    pub wk_radicals: Vec<String>,
    pub wk_level: Option<u32>,
}

#[component]
pub fn KanjiInfo() -> impl IntoView {
    // let kanjis = use_context::<Resource<usize, File>>().expect_throw("kanjis resources");
    let (viewed, set_viewed) =
        use_context::<ViewedContext>().expect_throw("expected viewed context");
    let params = use_params::<KanjiParams>();
    let (times_learned, set_times_learned) =
        use_context::<(Signal<u32>, WriteSignal<u32>)>().expect_throw("times learned context");
    let character = move || match params.get() {
        Ok(a) => a.kanji,
        _ => None,
    };

    let info = create_resource(
        move || character(),
        move |a| async move {
            if let Some(character) = a {
                #[derive(Serialize)]
                pub struct T {
                    kanji: String,
                }

                let res =
                    invokers::<T, Option<KanjiInfo>>("get_kanji_reading", T { kanji: character })
                        .await
                        .unwrap_throw();

                return res;
            }
            None
        },
    );

    let has_learned = create_memo(move |_| {
        with!(move |viewed, info| {
            let level = match info {
                Some(Some(a)) if a.wk_level.is_some() => a.wk_level.unwrap() - 1,
                _ => return false,
            };
            let Some(character) = character() else {
                return false;
            };
            viewed
                .levels
                .get(level as usize)
                .map(|a| a.kanjis.iter().find(|a| a.0 == character).is_some())
                .unwrap_or_default()
        })
    });

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
                    .radical
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
                    .radical
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
            let character = character().unwrap_or_default();
            if character.is_empty()
                || level_viewed
                    .kanjis
                    .iter()
                    .find(|a| a.0 == character)
                    .is_some()
            {
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
            level_viewed.kanjis.push((character, 0));
        })
    };
    let info = move || info.get().flatten();
    view! {
        <section class="info kanji">
            <header>
                <h1>
                    {character}
                </h1>
                {move || info().map(|a| view! {
                    <div class="square">
                        Strokes
                        <span>
                            {a.strokes}
                        </span>
                    </div>

                    {a.freq.map(|a| view! {
                    <div class="square">
                        Frequency
                        <span> {a} </span>
                    </div>
                    })}

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
                                Meanings
                            </h3>
                            {info.wk_meanings.iter().map(|a| view! {
                                <span> - {dislice(a)} </span>
                            }).collect_view()}
                        </div>
                        <div>
                            <h3>
                                Radicals
                            </h3>
                            {info.wk_radicals.iter().map(|a| view! {
                                <span> - {dislice(a)} </span>
                            }).collect_view()}
                        </div>
                    </div>
                    <div class="readings">
                        <div>
                            <h3>
                                Kunyoumi
                            </h3>
                            {info.wk_readings_kun.iter().map(|a| view! {
                                <span title=dislice(a).to_romaji()> - {dislice(a)} </span>
                            }).collect_view()}
                        </div>
                        <div>
                            <h3>
                                Onyoumi
                            </h3>
                            {info.wk_readings_on.iter().map(|a| view! {
                                <span title=dislice(a).to_romaji()> - {dislice(a)} </span>
                            }).collect_view()}
                        </div>
                    </div>
                }
            })}

            {move || character().map(|character| view! {
                <section class="other_sources">
                    <h3>
                        Other sources for yo studying
                    </h3>
                    <div>
                        <ButtonLink
                            url=format!("https://www.wanikani.com/kanji/{}", character)
                            text="WaniKani".to_string()
                        />
                        <ButtonLink
                            url=format!("https://www.kanshudo.com/search?q={}", character)
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
