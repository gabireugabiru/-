use leptos::window;
use wasm_bindgen::UnwrapThrowExt;
/// Return true if ok
pub fn local_set<T: serde::Serialize>(key: String, value: T) -> bool {
    let Ok(value_str) = serde_json::to_string(&value) else {
        return false
    };
    window().local_storage().expect_throw("Failed to local storage").expect_throw("Failed to storage")
        .set(&key, &value_str).is_ok()
} 
pub fn local_get<'a, T: serde::Deserialize<'a> + Clone>(key: String, buff: &'a mut String) -> Option<T> {
    let Ok(Some(res)) = window().local_storage().expect_throw("Failed to local storage").expect_throw("Failed to storage")
        .get(&key) else {
            return None
        };
    *buff = res.clone();
    let a: Result<T, _> = serde_json::from_str(buff.as_str());
    Some(a.unwrap().clone())
} 