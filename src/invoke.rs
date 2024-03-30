use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;

use crate::home::File;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub async fn invokers<T, A>(cmd: &str, value: T) -> Result<A, serde_wasm_bindgen::Error>
where
    T: serde::Serialize + Sized,
    A: DeserializeOwned,
{
    let value = serde_wasm_bindgen::to_value(&value)?;
    let res = invoke(cmd, value).await.unwrap_throw();
    serde_wasm_bindgen::from_value(res)
}
pub async fn get_kanjis(level: usize) -> Option<File> {
    #[derive(Serialize)]
    struct T {
        level: usize,
    }
    invokers("get_kanjis", T { level }).await.ok()
}
