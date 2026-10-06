// Bank format and validation rules (PLAN step 2).
mod common;

use common::{problems_of, shipped, tiny};
use warmups_core::bank;

#[test]
fn shipped_bank_loads() {
    let b = bank::load(common::BANK_JSON);
    assert!(b.is_ok(), "problems: {:?}", b.err());
}

#[test]
fn shipped_bank_has_24_questions_two_per_source_and_level() {
    let b = shipped();
    assert_eq!(b.questions.len(), 24);
    assert_eq!(b.sources.len(), 3);
    for s in ["A", "B", "C"] {
        for level in 1..=4u8 {
            let n = b.questions.iter().filter(|q| q.source == s && q.dok == level).count();
            assert_eq!(n, 2, "source {s} DOK {level}");
        }
    }
}

#[test]
fn shipped_bank_is_marked_sample() {
    let b = shipped();
    assert!(b.notice.contains("SAMPLE"));
    for s in &b.sources {
        assert!(s.title.contains("SAMPLE"), "source {} title", s.id);
    }
}

#[test]
fn tiny_bank_has_no_problems() {
    assert_eq!(problems_of(&tiny()), Vec::<String>::new());
}

#[test]
fn duplicate_question_id_is_refused_ignoring_case() {
    let mut v = tiny();
    v["questions"][1]["id"] = "a1A".into();
    let p = problems_of(&v);
    assert_eq!(p, vec!["Question id \"a1A\" is used twice.".to_string()]);
}

#[test]
fn dok_outside_one_to_four_is_refused() {
    for dok in [0, 5] {
        let mut v = tiny();
        v["questions"][0]["dok"] = dok.into();
        let p = problems_of(&v);
        assert_eq!(p, vec![format!("Question A1a has DOK {dok}; DOK must be 1, 2, 3 or 4.")]);
    }
}

#[test]
fn unknown_source_is_refused() {
    let mut v = tiny();
    v["questions"][0]["source"] = "Z".into();
    let p = problems_of(&v);
    assert_eq!(p, vec!["Question A1a names source \"Z\", which is not in the bank.".to_string()]);
}

#[test]
fn empty_prompt_is_refused() {
    let mut v = tiny();
    v["questions"][0]["prompt"] = "  ".into();
    assert_eq!(problems_of(&v), vec!["Question A1a has no prompt.".to_string()]);
}

#[test]
fn choice_key_must_be_an_option_letter() {
    let mut v = tiny();
    v["questions"][0]["answer"]["key"] = "C".into();
    assert_eq!(
        problems_of(&v),
        vec!["Question A1a has key \"C\", which is not one of its option letters.".to_string()]
    );
}

#[test]
fn choice_needs_two_options() {
    let mut v = tiny();
    v["questions"][0]["answer"]["options"] = serde_json::json!(["only"]);
    v["questions"][0]["answer"]["key"] = "A".into();
    assert_eq!(problems_of(&v), vec!["Question A1a needs between 2 and 26 options.".to_string()]);
}

#[test]
fn fraction_key_must_be_a_number() {
    let mut v = tiny();
    v["questions"][1]["answer"]["key"] = "three quarters".into();
    assert_eq!(
        problems_of(&v),
        vec!["Question A1b has key \"three quarters\", which is not a number.".to_string()]
    );
}

#[test]
fn simplest_key_must_itself_be_simplest() {
    let mut v = tiny();
    v["questions"][1]["answer"]["key"] = "6/8".into();
    assert_eq!(
        problems_of(&v),
        vec!["Question A1b asks for simplest form but its key \"6/8\" is not in simplest form.".to_string()]
    );
}

#[test]
fn unknown_form_is_refused() {
    let mut v = tiny();
    v["questions"][1]["answer"]["form"] = "roughly".into();
    let p = problems_of(&v);
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("unknown variant `roughly`"), "{p:?}");
}

#[test]
fn source_id_must_be_one_capital_letter() {
    let mut v = tiny();
    v["sources"][0]["id"] = "AB".into();
    v["questions"][0]["source"] = "AB".into();
    v["questions"][1]["source"] = "AB".into();
    assert_eq!(problems_of(&v), vec!["Source id \"AB\" must be one capital letter.".to_string()]);
}

#[test]
fn every_problem_is_listed_at_once() {
    let mut v = tiny();
    v["questions"][0]["dok"] = 9.into();
    v["questions"][1]["prompt"] = "".into();
    assert_eq!(problems_of(&v).len(), 2);
}
