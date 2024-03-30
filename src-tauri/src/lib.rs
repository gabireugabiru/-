use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
struct RadicalHash {
    character: String,
    wk_level: Option<u32>,
}
#[derive(Serialize, Deserialize, Clone)]
struct VocabHash {
    meanings: Vec<String>,
    wk_level: Option<u32>,
    readings: Vec<String>,
    primary_reading: String,
    primary_meaning: String,
    another_form: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone)]
struct Character {
    strokes: u32,
    freq: Option<u32>,
    wk_meanings: Vec<String>,
    wk_readings_on: Vec<String>,
    wk_readings_kun: Vec<String>,
    wk_radicals: Vec<String>,
    wk_level: Option<u32>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Kanji {
    character: String,
    meanings: Vec<String>,
    readings_kun: Vec<String>,
    readings_on: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Vocab {
    character: String,
    meaning: String,
    reading: String,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Radical {
    character: String,
    meaning: String,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct File {
    kanjis: Vec<Kanji>,
    vocabs: Vec<Vocab>,
    radicals: Vec<Radical>,
}

#[tauri::command]
fn get_kanjis<'a>(level: usize, levels: tauri::State<HashMap<usize, File>, 'a>) -> Option<File> {
    levels.get(&level).cloned()
}
#[tauri::command]
fn get_kanji_reading<'a>(
    kanji: String,
    all_kanjis: tauri::State<HashMap<String, Character>, 'a>,
) -> Option<Character> {
    all_kanjis.get(&kanji).cloned()
}
#[tauri::command]
async fn open_url(url: String) -> bool {
    webbrowser::open(&url).is_ok()
}
#[tauri::command]
fn get_radical<'a>(
    meaning: String,
    all_radicals: tauri::State<HashMap<String, RadicalHash>>,
) -> Option<RadicalHash> {
    all_radicals.get(&meaning).cloned()
}
#[tauri::command]
fn get_vocab<'a>(
    vocab: String,
    all_vocabs: tauri::State<HashMap<String, VocabHash>, 'a>,
) -> Option<VocabHash> {
    all_vocabs.get(&vocab).cloned()
}
#[tauri::command]
fn get_vocab_range<'a>(
    range: Vec<String>,
    all_vocabs: tauri::State<HashMap<String, VocabHash>, 'a>,
) -> Option<Vec<(String, VocabHash)>> {
    let mut vec = Vec::new();
    for i in range {
        if let Some(a) = all_vocabs.get(&i) {
            vec.push((i.clone(), a.clone()))
        }
    }

    (!vec.is_empty()).then(|| vec)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_kanjis,
            get_radical,
            get_kanji_reading,
            open_url,
            get_vocab,
            get_vocab_range
        ])
        .setup(|app| {
            let mut radicals = HashMap::new();
            let levels: HashMap<usize, File> =
                serde_json::from_str(include_str!("../levels.json")).expect("invalid file");
            let res: HashMap<String, Character> =
                serde_json::from_str(include_str!("../kanjis.json")).expect("invalid file");
            let vocabs: HashMap<String, VocabHash> =
                serde_json::from_str(include_str!("../vocabulary.json")).expect("invalid file");
            for (level, file) in &levels {
                for radical in &file.radicals {
                    radicals.insert(
                        radical.meaning.clone(),
                        RadicalHash {
                            character: radical.character.clone(),
                            wk_level: Some(*level as u32),
                        },
                    );
                }
            }
            app.manage(vocabs);
            app.manage(radicals);
            app.manage(res);
            app.manage(levels);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
