use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/public/storage.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn load() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    pub async fn save(value: &str) -> Result<JsValue, JsValue>;
    pub fn today(offset: i32) -> String;
    #[wasm_bindgen(js_name=shiftDate)]
    pub fn shift_date(key: &str, offset: i32) -> String;
    #[wasm_bindgen(js_name=dateLabel)]
    pub fn date_label(key: &str) -> String;
    pub fn weekday(key: &str) -> u32;
    pub fn theme(palette: &str, dark: bool);
    pub fn download(value: &str);
    #[wasm_bindgen(catch, js_name=readBackup)]
    pub async fn read_backup() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name=confirmRestore)]
    pub fn confirm_restore() -> bool;
    pub fn init();
    #[wasm_bindgen(js_name=notifyDone)]
    pub fn notify_done();
}

pub fn id() -> String {
    format!("{}-{}", js_sys::Date::now(), js_sys::Math::random())
}

pub fn elapsed(t: &crate::model::Timer) -> u32 {
    crate::model::elapsed_at(t, js_sys::Date::now())
}
