use std::time::Duration;

use crate::{
    home::{File, Radical, ShowRadical},
    invoke::invokers,
    kanji_info::dislice,
    today,
    vocab_info::VocabFullInfo,
    LastReviewedContext, Viewed, ViewedContext, ViewedList,
};
use leptos::*;
use leptos_router::{use_navigate, use_query, NavigateOptions, Params};
use rand::Rng;
use rust_fuzzy_search::fuzzy_compare;
use serde::Serialize;
use wana_kana::{ConvertJapanese, IsJapaneseStr};
use wasm_bindgen::prelude::*;
use web_sys::SubmitEvent;
#[derive(Debug, Clone, PartialEq, Eq)]
struct QuestionStatus {
    question_type: QuestionType,
    is_correct: bool,
    identifier: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerStatus {
    HardError,
    SoftError(&'static str),
    Correct,
    CorrectBitOff,
}
impl AnswerStatus {
    fn is_correct(&self) -> bool {
        match self {
            Self::Correct | Self::CorrectBitOff => true,
            _ => false,
        }
    }
    fn is_wrong(&self) -> bool {
        *self == Self::HardError
    }
}
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
pub enum QuestionType {
    VocabularyReading,
    VocabularyMeaning,
    KanjiReadingOn,
    KanjiReadingKun,
    KanjiMeaning,
    Radical,
}

impl QuestionType {
    fn is_kana(&self) -> bool {
        match self {
            Self::KanjiReadingKun | Self::KanjiReadingOn | Self::VocabularyReading => true,
            _ => false,
        }
    }
    fn is_kanji(&self) -> bool {
        match self {
            Self::KanjiMeaning | Self::KanjiReadingKun | Self::KanjiReadingOn => true,
            _ => false,
        }
    }
    fn is_radical(&self) -> bool {
        self == &Self::Radical
    }
    fn is_vocab(&self) -> bool {
        match self {
            Self::VocabularyMeaning | Self::VocabularyReading => true,
            _ => false,
        }
    }
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Question {
    answers: Vec<String>,
    question_type: QuestionType,
    alert: Vec<String>,
    alert_kana: Vec<String>,
    question: String,
    identifier: String,
    radical_character: Option<String>,
}
#[derive(Params, PartialEq, Eq, Default)]
struct IncludeQuery {
    kanji: Option<bool>,
    vocab: Option<bool>,
    radical: Option<bool>,
}
#[derive(Default)]
struct ShouldInclude {
    kanji: bool,
    radical: bool,
    vocab: bool,
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

fn add_mastery(
    question_status: Vec<QuestionStatus>,
    viewed: &Viewed,
    level: usize,
) -> Option<ViewedList> {
    let mut mastery = viewed.levels.get(level)?.clone();
    for question in question_status {
        if question.is_correct {
            match question.question_type {
                QuestionType::KanjiMeaning
                | QuestionType::KanjiReadingOn
                | QuestionType::KanjiReadingKun => {
                    let Some(kanji) = mastery
                        .kanjis
                        .iter_mut()
                        .find(|a| a.0 == question.identifier)
                    else {
                        continue;
                    };
                    kanji.1 += 1;
                }
                QuestionType::VocabularyMeaning | QuestionType::VocabularyReading => {
                    let Some(vocab) = mastery
                        .vocabs
                        .iter_mut()
                        .find(|a| a.0 == question.identifier)
                    else {
                        continue;
                    };
                    vocab.1 += 1;
                }
                QuestionType::Radical => {
                    let Some(radical) = mastery
                        .radicals
                        .iter_mut()
                        .find(|a| a.0 == question.identifier)
                    else {
                        continue;
                    };
                    radical.1 += 1;
                }
            }
        }
    }
    Some(mastery)
}

fn get_question_queue(
    file: &File,
    viewed: &Viewed,
    level: usize,
    vocabs: &Vec<(String, VocabFullInfo)>,
    should_include: ShouldInclude,
) -> Option<Vec<Question>> {
    let mut queue = Vec::new();
    let level = viewed.levels.get(level)?;
    let mut total_questions = Vec::new();
    if should_include.kanji {
        for (value, _) in &level.kanjis {
            let Some(kanji) = file.kanjis.iter().find(|a| &a.character == value) else {
                continue;
            };
            if kanji.character.is_empty() {
                continue;
            }
            if !kanji.meanings.is_empty() {
                let mut alert_kana = Vec::new();
                alert_kana.extend(kanji.readings_kun.clone());
                alert_kana.extend(kanji.readings_on.clone());

                total_questions.push(Question {
                    alert_kana,
                    alert: Vec::new(),
                    answers: kanji.meanings.clone(),
                    question_type: QuestionType::KanjiMeaning,
                    question: String::from("The meaning"),
                    identifier: kanji.character.clone(),
                    radical_character: None,
                });
            }
            if !kanji.readings_kun.is_empty() {
                total_questions.push(Question {
                    answers: kanji.readings_kun.clone(),
                    question_type: QuestionType::KanjiReadingKun,
                    alert: kanji.meanings.clone(),
                    alert_kana: kanji.readings_on.clone(),
                    question: String::from("The kunyoumi reading"),
                    identifier: kanji.character.clone(),
                    radical_character: None,
                });
            }
            if !kanji.readings_on.is_empty() {
                total_questions.push(Question {
                    answers: kanji.readings_on.clone(),
                    question_type: QuestionType::KanjiReadingOn,
                    alert: kanji.meanings.clone(),
                    alert_kana: kanji.readings_kun.clone(),
                    question: String::from("The onyoumi reading"),
                    identifier: kanji.character.clone(),
                    radical_character: None,
                });
            }
        }
    }
    if should_include.vocab {
        for (character, vocab) in vocabs {
            total_questions.push(Question {
                alert: vocab.meanings.clone(),
                alert_kana: Vec::new(),
                answers: vocab.readings.clone(),
                identifier: character.clone(),
                question: format!("The reading"),
                question_type: QuestionType::VocabularyReading,
                radical_character: None,
            });
            total_questions.push(Question {
                alert: Vec::new(),
                alert_kana: vocab.readings.clone(),
                answers: vocab.meanings.clone(),
                identifier: character.clone(),
                question: format!("The meaning"),
                question_type: QuestionType::VocabularyMeaning,
                radical_character: None,
            });
        }
    }
    if should_include.radical {
        for (value, _) in &level.radicals {
            logging::log!("{}", value);
            let Some(radical) = file.radicals.iter().find(|a| &a.meaning == value) else {
                continue;
            };
            if radical.character.is_empty() {
                continue;
            }
            let question = Question {
                alert: Vec::new(),
                alert_kana: Vec::new(),
                answers: vec![radical.meaning.to_lowercase()],
                identifier: radical.meaning.clone(),
                radical_character: Some(radical.character.clone()),
                question: String::from("Thea name of the radical"),
                question_type: QuestionType::Radical,
            };
            total_questions.push(question)
        }
    }

    let mut thread = rand::thread_rng();
    let mut remaining = total_questions.len();
    while remaining != 0 {
        let index = thread.gen_range(0..remaining);
        queue.push(total_questions[index].clone());
        total_questions.remove(index);
        remaining -= 1;
    }
    if queue.is_empty() {
        return None;
    }
    Some(queue)
}

#[component]
pub fn LearningKanji() -> impl IntoView {
    let query = use_query::<IncludeQuery>();
    let level = use_context::<Signal<usize>>().expect_throw("level");
    let (last_reviewed, _) =
        use_context::<(Signal<u8>, WriteSignal<u8>)>().expect_throw("last_reviewed");
    let (_, set_last_reviewed) =
        use_context::<LastReviewedContext>().expect_throw("last reviewed context");
    let (viewed, set_viewed) = use_context::<ViewedContext>().expect_throw("Viewed context");
    create_effect(move |_| {
        if last_reviewed.get() as u32 == today() {
            let navigate = use_navigate();
            navigate("/", NavigateOptions::default());
        }
    });
    let res = create_resource(
        move || level.get(),
        move |level| async move {
            #[derive(Serialize)]
            struct T {
                level: usize,
            }
            let res = invokers::<T, Option<File>>("get_kanjis", T { level: level + 1 })
                .await
                .expect_throw("level always should be ok");

            res
        },
    );
    let vocabs = create_resource(
        move || (level.get(), viewed.get()),
        |(level, viewed)| async move {
            let range = viewed
                .levels
                .get(level)
                .map(|view| view.vocabs.iter().map(|a| a.0.clone()).collect())?;
            #[derive(Serialize)]
            struct T {
                range: Vec<String>,
            }
            let res: Option<Vec<(String, VocabFullInfo)>> =
                invokers("get_vocab_range", T { range })
                    .await
                    .expect_throw("failed to get_vocab_range");
            res
        },
    );

    let end_quiz = move |questions| {
        let new_mastery = with!(move |level, viewed| { add_mastery(questions, viewed, *level) });
        if let Some(new_mastery) = new_mastery {
            set_viewed.update(move |a| {
                a.levels.get_mut(level.get()).map(|a| {
                    *a = new_mastery;
                });
            });
            let location = use_navigate();
            set_last_reviewed.set(today() as u8);
            location("/", NavigateOptions::default());
        }
    };
    view! {
        {move || with!(move |query, viewed, res, level, vocabs| {
            let queue_view = |queue: Vec<Question>| view! {
                <Quiz queue on_end=end_quiz />
            }.into_view();
            let empty_view = view! {
                <div class="empty">
                    its empty
                </div>
            }
            .into_view();
            let should_include = match query {
                Ok(a) => ShouldInclude {
                    kanji: a.kanji.unwrap_or_default(),
                    radical: a.radical.unwrap_or_default(),
                    vocab: a.vocab.unwrap_or_default()
                },
                _ => ShouldInclude::default()
            };
            match (res, vocabs) {
                (Some(Some(res)), Some(Some(vocabs))) => {
                    let queue = get_question_queue(res, viewed, *level, vocabs, should_include);
                    match queue {
                        Some(a) => queue_view(a),
                        None => empty_view
                    }
                }
                (Some(Some(res)), _) => {
                    let queue = get_question_queue(res, viewed, *level, &Vec::new(), should_include);
                    match queue {
                        Some(a) => queue_view(a),
                        None => empty_view
                    }
                }
                _ => empty_view
            }
        } )}
    }
}

fn is_correct(question: Question, answer: String) -> AnswerStatus {
    match question.question_type {
        QuestionType::KanjiMeaning | QuestionType::VocabularyMeaning | QuestionType::Radical => {
            for i in question.answers {
                let similarity = fuzzy_compare(&dislice(&i.to_lowercase()), &answer);
                if dislice(&i.to_lowercase()) == answer {
                    return AnswerStatus::Correct;
                }
                if similarity > 0.75 {
                    return AnswerStatus::CorrectBitOff;
                }
            }
            for i in question.alert_kana {
                if answer.to_hiragana() == dislice(&i) {
                    return AnswerStatus::SoftError("We're asking the meaning");
                }
            }
        }
        _ => {
            for i in question.answers {
                let similarity = fuzzy_compare(&dislice(&i), &answer.to_hiragana());
                if similarity > 0.8 {
                    return AnswerStatus::Correct;
                }
            }
            let message = if question.question.contains("kunyoumi") {
                "We're asking the kunyoumi reading"
            } else {
                "We're asking the onyoumi reading"
            };
            for i in question.alert {
                if answer.to_romaji() == i.to_lowercase() {
                    return AnswerStatus::SoftError(message);
                }
            }
            for i in question.alert_kana {
                if answer.to_hiragana() == dislice(&i) {
                    return AnswerStatus::SoftError(message);
                }
            }
        }
    }
    AnswerStatus::HardError
}

#[component]
fn Quiz<T>(queue: Vec<Question>, on_end: T) -> impl IntoView
where
    T: Fn(Vec<QuestionStatus>) -> () + Copy + 'static,
{
    let queue = create_rw_signal(queue);
    let index = create_rw_signal(0);
    let answer = create_rw_signal(String::new());
    let answer_status = create_rw_signal(None::<AnswerStatus>);
    let current_question = move || queue.with(|queue| queue.get(index.get()).cloned());
    let question_status = create_rw_signal(Vec::<QuestionStatus>::new());
    let go_to_next = move || {
        let has_next = with!(move |queue, index| queue.get(*index + 1).is_some());
        let new_question_status = with!(move |index, queue, answer_status| {
            let current_question = queue.get(*index)?;
            Some(QuestionStatus {
                identifier: current_question.identifier.clone(),
                is_correct: answer_status.map(|a| a.is_correct()).unwrap_or_default(),
                question_type: current_question.question_type,
            })
        });
        if let Some(new_question_status) = &new_question_status {
            question_status.update(move |a| a.push(new_question_status.clone()));
        }
        if has_next {
            answer_status.update(|a| *a = None);
            answer.update(|a| *a = String::new());
            index.update(|a| *a += 1);
        } else {
            on_end(question_status.get());
        }
    };
    let reff: NodeRef<html::Input> = create_node_ref();
    create_effect(move |_| {
        let reff = reff.get();
        if current_question().is_some() {
            if let Some(reff) = reff {
                set_timeout(
                    move || {
                        reff.focus().expect_throw("Couldn't focus");
                    },
                    Duration::from_millis(1),
                )
            }
        }
    });
    let confirm = move |ev: SubmitEvent| {
        ev.prevent_default();
        let answer = answer.get().trim().to_lowercase();
        if answer.is_empty() {
            return;
        }
        if let Some(answer_status) = answer_status.get() {
            if answer_status.is_correct() || answer_status.is_wrong() {
                go_to_next();
                return;
            }
        }
        if let Some(question) = current_question() {
            answer_status.set(Some(is_correct(question, answer)));
        }
    };
    let next_5 = move || {
        queue.with(|queue| {
            (index.get() + 1..index.get() + 6)
                .flat_map(|index| {
                    queue.get(index).map(|a| {
                        (
                            a.identifier.clone(),
                            a.question_type,
                            a.radical_character.clone(),
                        )
                    })
                })
                .collect::<Vec<(String, QuestionType, Option<String>)>>()
        })
    };
    move || match current_question() {
        Some(a) => view! {
            <div class="queue">
                {next_5().iter().map(|a| view! {
                    <span class:kanji=a.1.is_kanji() class:vocab=a.1.is_vocab() class:radical=a.1.is_radical()>
                        {match &a.2 {
                            Some(character) => view! {
                                <ShowRadical radical=Radical { character: character.clone(), meaning: a.0.clone() } />
                            }.into_view(),
                            None => a.0.clone().into_view()
                        }}
                    </span>
                }).collect_view()}
                <div>
                    {move || {
                        let remaining = queue.with(|a| a.len() - index.get() - 1);
                        (remaining > 5).then(|| view! {
                            {remaining - 5}+
                        })
                    }}
                </div>
            </div>
            <form
                class:no=move || answer_status.get().map(|a| a.is_wrong()).unwrap_or(false)
                class:yes=move || answer_status.get().map(|a| a.is_correct()).unwrap_or(false)
                on:submit=confirm
            >
                <div class="char"
                    class:vocab=a.question_type.is_vocab()
                    class:kanji=a.question_type.is_kanji()
                    class:radical=a.question_type.is_radical()
                >
                    {match a.radical_character {
                        Some(character) => view! {
                            <ShowRadical radical=Radical { character, meaning: a.identifier } />
                        }.into_view(),
                        None => a.identifier.into_view()
                    }}
                </div>
                <span class="question">
                    {a.question.clone()}
                </span>
                <div>
                    <input
                        ref=reff
                        type="text"
                        prop:value=move || answer.get()
                        on:keyup=move |ev| answer.set(event_target_value(&ev))
                        on:change=move |ev| answer.set(event_target_value(&ev))
                    />

                    <button>
                        confirm
                    </button>
                </div>
                <span class="translate">
                    {move || (a.question_type.is_kana() && !answer.get().is_hiragana()).then(|| view! {
                        {answer.get().to_hiragana()}
                    })}
                </span>
            </form>
            <button class="next" on:click=move |_| go_to_next()>
                next
            </button>
            {move || answer_status.get().map(|status| {
                let valid_answer = view! {
                    <h4>
                        The valid answers were
                    </h4>
                    {a.answers.iter().map(|answer| view! {
                        <span>
                            - {dislice(answer)}
                        </span>
                    }).collect_view()}
                };
                match status {
                    AnswerStatus::SoftError(err) => view! {
                        <div class="error_message">
                            <span>
                                {err}
                            </span>
                        </div>
                    }.into_view(),
                    AnswerStatus::CorrectBitOff => view! {
                        <div class="message">
                            <span>
                                Your answer was a bit off
                            </span>
                            {valid_answer}
                        </div>
                    }.into_view(),
                    AnswerStatus::HardError => view! {
                        <div class="message">
                            {valid_answer}
                        </div>
                    }.into_view(),
                    AnswerStatus::Correct => view! {
                        <div class="message">
                            {valid_answer}
                        </div>
                    }.into_view()
                }
            })}
        },
        None => view! {
            <>
                <div class="message">
                    Everything is poggers
                </div>
            </>
        },
    }
}
