use leptos_router::A;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use web_sys::MouseEvent;

use leptos::*;
use serde::{Deserialize, Serialize};

pub const UPPER_RADICAL: usize = 5;
pub const UPPER_KANJI: usize = 15;
pub const UPPER_VOCABULARY: usize = 10;

use crate::{svgs::Svgs, today, LastReviewedContext, Viewed, ViewedContext};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Kanji {
    pub character: String,
    pub meanings: Vec<String>,
    pub readings_kun: Vec<String>,
    pub readings_on: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Vocab {
    pub character: String,
    pub meaning: String,
    pub reading: String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Radical {
    pub character: String,
    pub meaning: String,
}
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct File {
    pub kanjis: Vec<Kanji>,
    pub vocabs: Vec<Vocab>,
    pub radicals: Vec<Radical>,
}
impl FromStr for File {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = serde_json::to_string(self);
        write!(f, "{}", str.unwrap_or("error".to_string()))
    }
}

#[component]
pub fn Home() -> impl IntoView {
    let level = use_context::<Signal<usize>>().expect_throw("failed to context");
    let kanjis = use_context::<Resource<usize, File>>().expect_throw("failed to kanjis resource");

    view! {
        {move || { view! {
            <Container level=level.get() kanjis/>
        }}}
    }
}
#[component]
pub fn ShowRadical(radical: Radical) -> impl IntoView {
    match radical.character.as_str() {
        "image" => view! {<Svgs  value=radical.meaning.to_lowercase()/>}.into_view(),
        _ => radical.character.into_view(),
    }
}
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MasteryType {
    Kanji,
    Vocabulary,
    Radical,
}

fn get_mastery(r#type: MasteryType, identifier: String, viewed: &Viewed, level: usize) -> usize {
    viewed
        .levels
        .get(level)
        .map(move |viewed| match r#type {
            MasteryType::Kanji => viewed
                .kanjis
                .iter()
                .find(|a| a.0 == identifier)
                .map(|a| a.1)
                .unwrap_or(0),
            MasteryType::Radical => viewed
                .radicals
                .iter()
                .find(|a| a.0 == identifier)
                .map(|a| a.1)
                .unwrap_or(0),
            MasteryType::Vocabulary => viewed
                .vocabs
                .iter()
                .find(|a| a.0 == identifier)
                .map(|a| a.1)
                .unwrap_or(0),
        })
        .unwrap_or_default()
}

#[component]
fn Container(level: usize, kanjis: Resource<usize, File>) -> impl IntoView {
    let (viewed, set_viewed) = use_context::<ViewedContext>().expect_throw("expected view context");
    let (last_reviewed, _) =
        use_context::<LastReviewedContext>().expect_throw("last reviewed context");
    let current_view = move || viewed.get().levels.get(level).cloned().unwrap_or_default();
    let has_acess =
        use_context::<Resource<(Viewed, usize), bool>>().expect_throw("cant get acess context");
    let clear_level = move |_: MouseEvent| {
        let confirmed = window()
            .confirm_with_message(
                "Are you sure you want to clear this level?\nThere will be no turning back",
            )
            .unwrap_or_default();
        if confirmed {
            let new_mastery = with!(move |kanjis, viewed| {
                let mut viewed = viewed.levels.get(level).cloned()?;
                let Some(kanjis) = kanjis else { return None };
                for i in &kanjis.radicals {
                    let pog = viewed.radicals.iter_mut().find(|a| a.0 == i.meaning);
                    if let Some(pog) = pog {
                        pog.1 = UPPER_RADICAL;
                    } else {
                        viewed.radicals.push((i.meaning.clone(), UPPER_RADICAL));
                    }
                }
                for i in &kanjis.kanjis {
                    let pog = viewed.kanjis.iter_mut().find(|a| a.0 == i.character);
                    if let Some(pog) = pog {
                        pog.1 = UPPER_KANJI;
                    } else {
                        viewed.kanjis.push((i.character.clone(), UPPER_KANJI));
                    }
                }
                for i in &kanjis.vocabs {
                    let pog = viewed.vocabs.iter_mut().find(|a| a.0 == i.character);
                    if let Some(pog) = pog {
                        pog.1 = UPPER_VOCABULARY;
                    } else {
                        viewed.vocabs.push((i.character.clone(), UPPER_VOCABULARY));
                    }
                }
                Some(viewed)
            });
            if let Some(new_mastery) = new_mastery {
                set_viewed.update(move |a| {
                    a.levels.get_mut(level).map(|a| {
                        *a = new_mastery;
                    });
                });
            }
        }
    };

    view! {

        {move || has_acess.get().unwrap_or_default().then(|| {

            if last_reviewed.get() as u32 == today() {
                view! {
                    <section class="empty_reviews">
                        You did your reviews for today
                    </section>
                }
            } else {
                view! {
                    <section class="reviews">
                        <A class="button" href="/learningkanji?kanji=true&vocab=true&radical=true">
                            Full review
                        </A>
                        <A class="button radical" href="/learningkanji?kanji=false&vocab=false&radical=true">
                            Radical review
                        </A>
                        <A class="button kanji" href="/learningkanji?kanji=true&vocab=false&radical=false">
                            Kanji review
                        </A>
                        <A class="button vocab" href="/learningkanji?kanji=false&vocab=true&radical=false">
                            Vocabulary review
                        </A>
                    </section>

                }
            }

        })}


        // <button on:click=clear_level>
        //     clear level
        // </button>
        // RADICALS GRID
        <h1>Radicals</h1>
        <section class="radicals">
            {move || kanjis.map(|a| {
                let current_view_radicals = current_view().radicals;
                a.radicals.clone().into_iter().map(move |radical| {
                    let Radical { meaning, ..} = radical.clone();
                    let meaning_2 = meaning.clone();
                    let meaning_3 = meaning.clone();
                    let is_radical_pog = current_view_radicals.clone().into_iter().find(move |a| a.0 == meaning_2.clone()).is_some();
                    let mastery = viewed.with(move |viewed| get_mastery(MasteryType::Radical, meaning_3.clone(),viewed, level));
                    view! {
                        <A class=move || is_radical_pog.then(|| "viewed").unwrap_or_default() href=format!("/radical/{meaning}")>
                            <div>
                                <ShowRadical radical=radical.clone() />
                            </div>
                            <div
                                style=format!("width: {}%;", ((100 as f32/ UPPER_RADICAL as f32)*mastery as f32).min(100.0))
                                >
                            </div>
                        </A>
                    }
                }).collect_view()
            })}
        </section>

        // KANJIS GRID
        <h1>Kanjis</h1>
        <section class="kanjis">
            {move || kanjis.map(|a| {
                let current_view_kanjis = current_view().kanjis;
                a.kanjis.clone().into_iter().map(move |kanji| {
                    let Kanji { character, ..} = kanji.clone();
                    let character_2 = character.clone();
                    let is_kanji_pog = current_view_kanjis.clone().into_iter().find(move |a| a.0 == kanji.character.clone()).is_some();
                    let mastery = viewed.with(move |viewed| get_mastery(MasteryType::Kanji, character_2.clone(),viewed, level));
                    logging::log!("{mastery}");
                    view! {
                        <A class=move || is_kanji_pog.then(|| "viewed").unwrap_or_default() href=format!("/kanji/{}", character)>
                            <div>
                                {character.clone()}
                            </div>
                            <div
                                style=format!("width: {}%;", ((100 as f32/ UPPER_KANJI as f32)*mastery as f32).min(100.0))
                            />
                        </A>
                    }
                }).collect_view()
            })}
        </section>

        //VOCABS GRID
        <h1>Vocabulary</h1>
        <section class="vocabs">
            {move || kanjis.map(|a| {
                let current_view_vocabs = current_view().vocabs;
                a.vocabs.clone().into_iter().map(move |vocab| {
                    let Vocab { character, reading, meaning} = vocab.clone();
                    let character_2 = character.clone();
                    let is_kanji_pog = current_view_vocabs.clone().into_iter().find(move |a| a.0 == vocab.character.clone()).is_some();
                    let mastery = viewed.with(move |viewed| get_mastery(MasteryType::Vocabulary, character_2.clone(),viewed, level));
                    view! {
                        <A class=move || is_kanji_pog.then(|| "viewed").unwrap_or_default() href=format!("/vocab/{}", character)>
                            <span>
                                {character.clone()}
                            </span>
                            <div>
                                <span>
                                    {meaning.clone()}
                                </span>
                                <span>
                                    {reading.clone()}
                                </span>
                            </div>
                        </A>
                        <div
                            style=format!("width: {}%;", ((100 as f32/ UPPER_VOCABULARY as f32)*mastery as f32).min(100.0))
                            class="vocab_percentage"
                        />
                    }
                }).collect_view()
            })}
        </section>
        <section class="danger">
            <button class="clear" on:click=clear_level>
                Clear this level
            </button>
        </section>
    }
}
