const fs = require('node:fs');
const new_json = {

}
// let new_level = []
// for (let i = 0; i < 61; i++) {
//     new_level.push({});
// }

// let levels = {};
// fs.readFile("src-tauri/kanjis.json", (err, data) => {
// const json = JSON.parse(data);
// for (key in json) {
//     new_json[key] = {
//         wk_meanings: json[key].wk_meanings,
//         wk_readings_on: json[key].wk_readings_on,
//         wk_readings_kun: json[key].wk_readings_kun,
//         wk_radicals: json[key].wk_radicals,
//         wk_level: json[key].wk_level,
//         freq: json[key].freq,
//         strokes: json[key].strokes
//     } 
// }
// fs.writeFile("src-tauri/kanjis2.json", JSON.stringify(new_json), err => {});

// console.log(new_json["々"]);
// for (let i = 1; i < 61; i++) {
//     fs.readFile(`src-tauri/levels2/${i}.json`, (err, data) => {
//         const json = JSON.parse(data);
//         levels[i] = json;
// levels.push(json);
// new_level[i].vocabs = json.vocabs;
// new_level[i].radicals = json.radicals;
// new_level[i].kanjis = [];
// for (const kanji of json.kanjis) {
//     console.log(kanji.character);
//     let x =new_json[kanji.character];
//     let has_x = false;
//     if (x) {
//         if (x.wk_meanings.length > 0 ) {
//             has_x = true;
//         }
//     }

//     new_level[i].kanjis.push({
//         character: kanji.character,
//         meanings: has_x ? new_json[kanji.character].wk_meanings : [kanji.meaning],
//         readings_kun: new_json[kanji.character] ? new_json[kanji.character].wk_readings_kun : [kanji.reading],
//         readings_on: new_json[kanji.character] ? new_json[kanji.character].wk_readings_on : [kanji.reading]

//     })
// }
// fs.writeFile(`src-tauri/levels2/${i}.json`, JSON.stringify(new_level[i]), err=> {
//     console.log(err);
// });
//         fs.writeFile("src-tauri/blob.json", JSON.stringify(levels), err => {});
//     });
// }
// })
let n = 0;
const radicals = {

}
fs.readFile(`src-tauri/blob.json`, (err, data) => {
    let json = JSON.parse(data);
    for (let i = 1; i < 61; i++) {
        for (const vocab of json[i].vocabs) {
            if (radicals[vocab.character]) {
                console.log("found a match");
                continue;
            }
            radicals[vocab.character] = { meaning: vocab.character, wk_level: i };
        }
    }
    console.log(radicals);
});