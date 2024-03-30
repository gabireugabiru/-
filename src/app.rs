use crate::{
    home::{File, Home, UPPER_KANJI, UPPER_RADICAL, UPPER_VOCABULARY},
    invoke::{get_kanjis, invokers},
    kanji_info::KanjiInfo,
    learningkanji::LearningKanji,
    radical_info::RadicalInfo,
    utils::local_get,
    vocab_info::VocabInfo,
};
use chrono::Datelike;
use leptos::*;
use leptos_router::{use_location, Route, Routes, A};
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};
use wasm_bindgen::UnwrapThrowExt;
pub type ViewedContext = (Signal<Viewed>, WriteSignal<Viewed>);
pub type LastReviewedContext = (Signal<u8>, WriteSignal<u8>);
pub const LIMIT: u32 = 15;

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}
#[derive(Clone, PartialEq, Eq, Default, Deserialize, Serialize, Debug)]
pub struct ViewedList {
    pub kanjis: Vec<(String, usize)>,
    pub radicals: Vec<(String, usize)>,
    pub vocabs: Vec<(String, usize)>,
}
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize, Debug)]
pub struct Viewed {
    pub levels: Vec<ViewedList>,
}
impl Default for Viewed {
    fn default() -> Self {
        Self {
            levels: vec![ViewedList::default(); 60],
        }
    }
}
impl FromStr for Viewed {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
impl std::fmt::Display for Viewed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match serde_json::to_string(self) {
                Ok(a) => a,
                Err(_) => String::from("Invalid data"),
            }
        )
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AcessType {
    Partial,
    Full,
    None,
}
impl AcessType {
    pub fn partial_or_more(&self) -> bool {
        self == &Self::Partial || self == &Self::Full
    }
}
impl From<(usize, usize, usize)> for AcessType {
    fn from((has, partial, expected): (usize, usize, usize)) -> Self {
        logging::log!("{has} {partial} {expected}");
        if has >= expected {
            AcessType::Full
        } else if has >= partial {
            AcessType::Partial
        } else {
            AcessType::None
        }
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Acess {
    pub kanji: AcessType,
    pub vocab: AcessType,
    pub radical: AcessType,
}

impl Default for Acess {
    fn default() -> Self {
        Self {
            kanji: AcessType::None,
            radical: AcessType::None,
            vocab: AcessType::None,
        }
    }
}
impl Acess {
    pub fn has_all(&self) -> bool {
        self.kanji == AcessType::Full
            && self.vocab == AcessType::Full
            && self.radical == AcessType::Full
    }
}
pub fn has_acess_logic(level: usize, kanjis: &Option<File>, viewed: &Viewed) -> Acess {
    let Some(level_info) = kanjis else {
        return Acess::default();
    };

    let is_expected = viewed.levels.get(level - 1).map(|a| {
        let total_kanji_mastery = a.kanjis.iter().fold(0, |acc, value| acc + value.1);
        let total_vocab_mastery = a.vocabs.iter().fold(0, |acc, value| acc + value.1);
        let total_radical_mastery = a.radicals.iter().fold(0, |acc, value| acc + value.1);
        let expected_kanji_mastery = level_info.kanjis.len() * UPPER_KANJI;
        let expected_vocab_mastery = level_info.vocabs.len() * UPPER_VOCABULARY;
        let expected_radical_mastery = level_info.radicals.len() * UPPER_RADICAL;
        let partial_kanji_mastery = level_info.kanjis.len() * (UPPER_KANJI / 5);
        let partial_vocab_mastery = level_info.vocabs.len() * (UPPER_VOCABULARY / 5);
        let partial_radical_mastery = level_info.radicals.len() * (UPPER_RADICAL / 5);

        Acess {
            kanji: AcessType::from((
                total_kanji_mastery,
                partial_kanji_mastery,
                expected_kanji_mastery,
            )),
            vocab: AcessType::from((
                total_vocab_mastery,
                partial_vocab_mastery,
                expected_vocab_mastery,
            )),
            radical: AcessType::from((
                total_radical_mastery,
                partial_radical_mastery,
                expected_radical_mastery,
            )),
        }
    });
    is_expected.unwrap_or_default()
}

pub fn today() -> u32 {
    chrono::Local::now().date_naive().day()
}
#[component]
pub fn App() -> impl IntoView {
    let location = use_location();
    let (level, set_level, _) =
        use_local_storage::<usize, leptos_use::utils::FromToStringCodec>("level");
    let (viewed, set_viewed, _) =
        use_local_storage::<Viewed, leptos_use::utils::FromToStringCodec>("viewed");
    let (times_learned, set_times_learned, _) =
        use_local_storage::<u32, leptos_use::utils::FromToStringCodec>("times_learned");
    let (last_reviewed, set_last_reviewed, _) =
        use_local_storage::<u8, leptos_use::utils::FromToStringCodec>("last_reviewed");
    set_interval(
        move || {
            let Some(value) = local_get::<u32>("last_learned".to_string(), &mut String::new())
            else {
                return;
            };
            let today = today();

            if last_reviewed.get() as u32 != today && last_reviewed.get() != 255 {
                set_last_reviewed.set(255);
            }

            if today != value && times_learned.get() != 0 {
                set_times_learned.set(0);
            }
        },
        Duration::from_secs(1),
    );

    let kanjis = create_resource(
        move || level.get(),
        move |level| async move {
            #[derive(Serialize)]
            struct T {
                level: usize,
            }
            let res = invokers::<T, Option<File>>("get_kanjis", T { level: level + 1 })
                .await
                .expect_throw("Invalid value");
            res.expect_throw("Invalid level")
        },
    );

    let has_acess = create_resource(
        move || (viewed.get(), level.get()),
        move |(viewed, level)| async move {
            if level == 0 {
                return true;
            }
            let Some(kanjis) = get_kanjis(level).await else {
                return false;
            };
            has_acess_logic(level as usize, &Some(kanjis), &viewed).has_all()
        },
    );
    provide_context((times_learned, set_times_learned));
    provide_context((last_reviewed, set_last_reviewed));
    provide_context(level);
    provide_context((viewed, set_viewed));
    provide_context(kanjis);
    provide_context(has_acess);
    create_effect(move |_| {
        logging::log!("{}", location.pathname.get());
    });
    view! {
        <header class="main_header">
            <div>
                <a class="navigation" href="javascript:history.back()">
                <svg fill="#000000" version="1.1" id="Capa_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
                    width="800px" height="800px" viewBox="0 0 400.004 400.004"
                    xml:space="preserve">
                    <g>
                        <path d="M382.688,182.686H59.116l77.209-77.214c6.764-6.76,6.764-17.726,0-24.485c-6.764-6.764-17.73-6.764-24.484,0L5.073,187.757
                            c-6.764,6.76-6.764,17.727,0,24.485l106.768,106.775c3.381,3.383,7.812,5.072,12.242,5.072c4.43,0,8.861-1.689,12.242-5.072
                            c6.764-6.76,6.764-17.726,0-24.484l-77.209-77.218h323.572c9.562,0,17.316-7.753,17.316-17.315
                            C400.004,190.438,392.251,182.686,382.688,182.686z"/>
                    </g>
                </svg>
                </a>
                <A class="navigation" href="/">
                    <svg xmlns="http://www.w3.org/2000/svg"  viewBox="0 0 48 48" width="48px" height="48px"><path d="M39.5,43h-9c-1.381,0-2.5-1.119-2.5-2.5v-9c0-1.105-0.895-2-2-2h-4c-1.105,0-2,0.895-2,2v9c0,1.381-1.119,2.5-2.5,2.5h-9	C7.119,43,6,41.881,6,40.5V21.413c0-2.299,1.054-4.471,2.859-5.893L23.071,4.321c0.545-0.428,1.313-0.428,1.857,0L39.142,15.52	C40.947,16.942,42,19.113,42,21.411V40.5C42,41.881,40.881,43,39.5,43z"/></svg>
                </A>
            </div>
            <select prop:disabled=move || location.pathname.with(|a| a != "/") class="level" on:change=move |ev| {
                let value = event_target_value(&ev);
                match value.parse::<usize>() {
                    Ok(value) => {
                        set_level.set(value);
                    }
                    _ => {}
                }
            }>
                {(0..60).map(move |a| {
                    view! {
                        <option value=format!("{a}") selected=move || level.get() == a >{a + 1}</option>
                    }
                }).collect_view()}
            </select>
        </header>
        <main class="container">
            <Routes>
                <Route path="/" view=Home/>
                <Route path="/learningkanji" view=LearningKanji/>
                <Route path="/kanji/:kanji" view=KanjiInfo/>
                <Route path="/radical/:radical_meaning" view=RadicalInfo />
                <Route path="/vocab/:vocab" view=VocabInfo/>
            </Routes>
        </main>
    }
}
