//! Thin wasm-bindgen shim. Each function passes the page's text to the core
//! and returns the core's JSON text unchanged. It decides nothing: the
//! boundary test `shim_only_passes_values_to_the_core` holds it to that.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn options(bank_json: &str) -> String {
    warmups_core::api::options(bank_json)
}

#[wasm_bindgen]
pub fn make_set(bank_json: &str, request_json: &str) -> String {
    warmups_core::api::make_set(bank_json, request_json)
}

#[wasm_bindgen]
pub fn swap(bank_json: &str, code: &str, number: u32) -> String {
    warmups_core::api::swap(bank_json, code, number)
}

#[wasm_bindgen]
pub fn open_set(bank_json: &str, code_or_file: &str) -> String {
    warmups_core::api::open_set(bank_json, code_or_file)
}

#[wasm_bindgen]
pub fn check(bank_json: &str, code: &str, answers_json: &str) -> String {
    warmups_core::api::check(bank_json, code, answers_json)
}

#[wasm_bindgen]
pub fn set_file(bank_json: &str, code: &str) -> String {
    warmups_core::api::set_file(bank_json, code)
}
