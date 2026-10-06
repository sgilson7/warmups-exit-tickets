// The JSON API the page uses (PLAN step 7), and the saved file (step 5).
mod common;

use common::BANK_JSON;
use serde_json::{json, Value};
use warmups_core::api;

fn parse(text: String) -> Value {
    serde_json::from_str(&text).expect("API replies are JSON")
}

fn ids(v: &Value) -> Vec<String> {
    v["set"]["questions"].as_array().unwrap().iter().map(|q| q["id"].as_str().unwrap().to_string()).collect()
}

#[test]
fn options_lists_sources_levels_and_kinds() {
    let v = parse(api::options(BANK_JSON));
    assert_eq!(v["ok"], true);
    assert_eq!(v["sources"].as_array().unwrap().len(), 3);
    assert_eq!(v["levels"][2], json!({"value": "3", "label": "DOK 3: Strategic thinking", "default": false}));
    assert_eq!(v["kinds"][0], json!({"value": "warmup", "label": "Warm-up (3 questions)", "default": true}));
    assert_eq!(v["kinds"][1]["label"], "Exit ticket (2 questions)");
    assert!(v["notice"].as_str().unwrap().starts_with("SAMPLE CONTENT"));
}

#[test]
fn make_set_returns_the_teacher_view_with_keys() {
    let req = json!({"kind": "warmup", "sources": ["B", "A"], "level": "3"}).to_string();
    let v = parse(api::make_set(BANK_JSON, &req));
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["set"]["code"], "W3-AB-A3A-B3A-A3B");
    assert_eq!(ids(&v), ["A3a", "B3a", "A3b"]);
    assert_eq!(v["set"]["title"], "Warm-up: DOK 3 (Strategic thinking), sources A and B");
    let q1 = &v["set"]["questions"][0];
    assert_eq!(q1["key"], "2/3 (any equal number matches)");
    assert_eq!(q1["input"], "text");
    assert_eq!(q1["can_swap"], true);
    let q3 = &v["set"]["questions"][2];
    assert_eq!(q3["input"], "choice");
    assert_eq!(q3["options"][1]["letter"], "B");
    assert_eq!(q3["key"], "B) Yes, because the total is 2 1/6 cups.");
    assert_eq!(v["set"]["swap_note"], "");
    assert_eq!(v["set"]["file_name"], "warm-up-W3-AB-A3A-B3A-A3B.txt");
}

#[test]
fn make_set_reports_the_shortfall() {
    let req = json!({"kind": "warmup", "sources": ["B"], "level": "3"}).to_string();
    let v = parse(api::make_set(BANK_JSON, &req));
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["error"],
        "Only 2 questions in the bank match source B at DOK 3, and a warm-up needs 3. \
         Choose another source as well, or make an exit ticket."
    );
}

#[test]
fn single_source_exit_ticket_view_says_nothing_can_be_swapped() {
    let req = json!({"kind": "exit", "sources": ["B"], "level": "3"}).to_string();
    let v = parse(api::make_set(BANK_JSON, &req));
    assert_eq!(v["set"]["questions"][0]["can_swap"], false);
    assert!(v["set"]["swap_note"].as_str().unwrap().starts_with("No other question"));
}

#[test]
fn swap_through_the_api() {
    let v = parse(api::swap(BANK_JSON, "W3-AB-A3A-B3A-A3B", 2));
    assert_eq!(v["set"]["code"], "W3-AB-A3A-B3B-A3B");
    let v = parse(api::swap(BANK_JSON, "W3-AB-A3A-B3A-A3B", 7));
    assert_eq!(v, json!({"ok": false, "error": "There is no question 7 in this set."}));
}

#[test]
fn student_view_has_no_key() {
    let v = parse(api::open_set(BANK_JSON, "w3-ab-a3a-b3b-a3b"));
    assert_eq!(v["ok"], true);
    assert_eq!(v["set"]["teacher"], false);
    for q in v["set"]["questions"].as_array().unwrap() {
        assert!(q.get("key").is_none() && q.get("note").is_none() && q.get("can_swap").is_none(), "{q}");
    }
}

#[test]
fn check_through_the_api() {
    let v = parse(api::check(BANK_JSON, "W3-AB-A3A-B3B-A3B", r#"["4/6", "1/4", "b"]"#));
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["result"]["summary"], "2 of 3 answers match the key.");
    assert_eq!(v["result"]["marks"][1]["label"], "Does not match the key");
}

#[test]
fn saved_file_carries_the_code_and_no_key() {
    let v = parse(api::set_file(BANK_JSON, "W3-AB-A3A-B3A-A3B"));
    assert_eq!(v["file_name"], "warm-up-W3-AB-A3A-B3A-A3B.txt");
    let text = v["text"].as_str().unwrap();
    assert!(text.starts_with("Warm-up: DOK 3 (Strategic thinking), sources A and B\n"));
    assert!(text.contains("\nSet code: W3-AB-A3A-B3A-A3B\n"));
    assert!(text.contains("   B) Yes, because the total is 2 1/6 cups.\n"));
    assert!(text.contains("SAMPLE CONTENT"));
    // No key or teacher note: A3a's note and the key wording must not appear.
    assert!(!text.contains("1/2 divided by 3/4"));
    assert!(!text.contains("matches)"));
    // The student page reads the file back to the same set.
    let opened = parse(api::open_set(BANK_JSON, text));
    assert_eq!(opened["set"]["code"], "W3-AB-A3A-B3A-A3B");
}

#[test]
fn bad_bank_and_bad_input_get_error_replies() {
    let v = parse(api::options("{}"));
    assert_eq!(v["ok"], false);
    let mut broken: Value = serde_json::from_str(BANK_JSON).unwrap();
    broken["questions"][1]["id"] = "A1A".into();
    let v = parse(api::options(&broken.to_string()));
    assert_eq!(v["problems"], json!(["Question id \"A1A\" is used twice."]));
    let v = parse(api::make_set(BANK_JSON, "not json"));
    assert_eq!(v["ok"], false);
    let v = parse(api::make_set(BANK_JSON, &json!({"kind": "quiz", "sources": ["A"], "level": "1"}).to_string()));
    assert_eq!(v["error"], "Choose a warm-up or an exit ticket.");
    let v = parse(api::make_set(BANK_JSON, &json!({"kind": "exit", "sources": [], "level": "1"}).to_string()));
    assert_eq!(v["error"], "Choose at least one source document.");
    let v = parse(api::check(BANK_JSON, "W3-AB-A3A-B3B-A3B", "[1, 2, 3]"));
    assert_eq!(v["ok"], false);
    let v = parse(api::open_set(BANK_JSON, "nonsense"));
    assert_eq!(v["error"], "That is not a set code. A set code looks like W3-AB-A3A-B3A-A3B.");
}
