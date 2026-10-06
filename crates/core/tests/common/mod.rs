// Shared helpers for the integration tests (included with `mod common;`).
#![allow(dead_code)]

use serde_json::{json, Value};
use warmups_core::bank::{self, Bank};

pub const BANK_JSON: &str = include_str!("../../../../data/bank.json");

pub fn shipped() -> Bank {
    bank::load(BANK_JSON).expect("shipped bank should load")
}

/// A small valid bank: one source "A", two DOK 1 questions.
pub fn tiny() -> Value {
    json!({
        "bank_version": "t",
        "notice": "test",
        "sources": [{"id": "A", "title": "Test source", "text": "x"}],
        "questions": [
            {"id": "A1a", "source": "A", "dok": 1, "prompt": "p",
             "answer": {"kind": "choice", "options": ["x", "y"], "key": "B"}},
            {"id": "A1b", "source": "A", "dok": 1, "prompt": "q",
             "answer": {"kind": "fraction", "key": "3/4", "form": "simplest"}}
        ]
    })
}

pub fn problems_of(v: &Value) -> Vec<String> {
    match bank::load(&v.to_string()) {
        Ok(_) => Vec::new(),
        Err(p) => p,
    }
}
