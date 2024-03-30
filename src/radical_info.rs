use chrono::Datelike;
use leptos::*;
use leptos_router::use_params;
use leptos_router::Params;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::MouseEvent;

use crate::button_link::ButtonLink;
use crate::has_acess_logic;
use crate::home::File;
use crate::home::Radical;
use crate::home::ShowRadical;
use crate::invoke::invokers;
use crate::utils::local_set;
use crate::ViewedContext;
use crate::LIMIT;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadicalFullInfo {
    character: String,
    wk_level: Option<u32>,
}

#[derive(Clone, PartialEq, Eq, Params)]
pub struct RadicalParams {
    radical_meaning: Option<String>,
}
#[component]
pub fn RadicalInfo() -> impl IntoView {
    let (viewed, set_viewed) =
        use_context::<ViewedContext>().expect_throw("expected viewed context");
    let params = use_params::<RadicalParams>();
    let (times_learned, set_times_learned) =
        use_context::<(Signal<u32>, WriteSignal<u32>)>().expect_throw("times learned context");
    let meaning = move || match params.get() {
        Ok(a) => a.radical_meaning,
        _ => None,
    };

    let info = create_resource(
        move || meaning(),
        move |a| async move {
            if let Some(meaning) = a {
                #[derive(Serialize)]
                pub struct T {
                    meaning: String,
                }

                let res = invokers::<T, Option<RadicalFullInfo>>("get_radical", T { meaning })
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
            let Some(meaning) = meaning() else {
                return false;
            };
            viewed
                .levels
                .get(level as usize)
                .map(|a| a.radicals.iter().find(|a| a.0 == meaning).is_some())
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
            if level <= 0 {
                return true;
            }
            #[derive(Serialize)]
            struct T {
                level: usize,
            }
            let Ok(Some(kanjis)) = invokers::<T, Option<File>>(
                "get_kanjis",
                T {
                    level: level as usize,
                },
            )
            .await
            else {
                return false;
            };
            has_acess_logic(level as usize, &Some(kanjis), &viewed).has_all()
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
            let meaning = meaning().unwrap_or_default();
            if meaning.is_empty()
                || level_viewed
                    .vocabs
                    .iter()
                    .find(|a| a.0 == meaning)
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
            level_viewed.radicals.push((meaning, 0));
        })
    };
    let info = move || info.get().flatten();
    view! {
        <section class="info radical">
            <header>
                {move || info().map(|info| meaning().map(|meaning| view! {
                    <h1>
                        <ShowRadical radical=Radical { character: info.character, meaning }/>
                    </h1>
                }))}
                {move || info().map(|a| view! {
                    {a.wk_level.map(|a| view! {
                    <div class="square">
                        Level
                        <span> {a} </span>
                    </div>
                    })}
                })}
            </header>
            <div class="meanings">
                <div>
                    <div>
                        <h3>Meaning</h3>
                        {move || meaning().map(|meaning| view! {
                            <span> - {meaning} </span>
                        })}
                    </div>
                </div>
            </div>
            {move || meaning().map(|meaning| view! {
                <section class="other_sources">
                    <h3>
                        Other sources for yo studying
                    </h3>
                    <div>
                        <ButtonLink
                            url=format!("https://www.wanikani.com/radicals/{}", meaning)
                            text="WaniKani".to_string()
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
