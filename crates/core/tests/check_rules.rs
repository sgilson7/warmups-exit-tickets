// Answer checking and score (PLAN step 6). Expected values worked out by hand
// from the keys in data/bank.json.
mod common;

use common::shipped;
use warmups_core::check::{answer_matches, check};
use warmups_core::code::decode;

fn matches(id: &str, given: &str) -> bool {
    let b = shipped();
    answer_matches(&b.question(id).unwrap().answer, given)
}

#[test]
fn value_form_accepts_any_equal_number() {
    // A3a key 2/3, form value.
    assert!(matches("A3a", "2/3"));
    assert!(matches("A3a", "4/6"));
    assert!(matches("A3a", " 8 / 12 "));
    assert!(!matches("A3a", "1/3"));
    assert!(!matches("A3a", "0.67"));
    // C1b key 30, form value.
    assert!(matches("C1b", "30"));
    assert!(matches("C1b", "60/2"));
    assert!(matches("C1b", "30.0"));
    assert!(!matches("C1b", "29"));
}

#[test]
fn simplest_form_needs_an_equal_number_in_lowest_terms() {
    // A2a key 1 1/4, form simplest.
    assert!(matches("A2a", "1 1/4"));
    assert!(matches("A2a", "5/4"));
    assert!(!matches("A2a", "10/8"));
    assert!(!matches("A2a", "1 2/8"));
    assert!(!matches("A2a", "1.25"));
    assert!(!matches("A2a", "1 1/2"));
}

#[test]
fn exact_form_needs_the_same_written_form() {
    // A1b key 6/8, form exact.
    assert!(matches("A1b", "6/8"));
    assert!(matches("A1b", " 6 / 8 "));
    assert!(!matches("A1b", "3/4"));
    assert!(!matches("A1b", "12/16"));
}

#[test]
fn choice_compares_letters_in_any_case() {
    // A1a key B.
    assert!(matches("A1a", "B"));
    assert!(matches("A1a", " b "));
    assert!(!matches("A1a", "A"));
    assert!(!matches("A1a", "2/3 cup"));
}

#[test]
fn blank_never_matches() {
    for id in ["A1a", "A1b", "A2a", "A3a"] {
        assert!(!matches(id, ""), "{id}");
        assert!(!matches(id, "   "), "{id}");
    }
}

#[test]
fn score_for_the_swapped_a_b_dok_3_warm_up() {
    // Set A3a (key 2/3), B3b (key 1/8), A3b (key B).
    // Answers 4/6 (equal to 2/3: match), 1/4 (not 1/8: no match), B (match).
    let b = shipped();
    let set = decode(&b, "W3-AB-A3A-B3B-A3B").unwrap();
    let answers = vec!["4/6".to_string(), "1/4".to_string(), "B".to_string()];
    let score = check(&b, &set, &answers).unwrap();
    let got: Vec<(usize, bool, bool)> = score.marks.iter().map(|m| (m.number, m.answered, m.matches)).collect();
    assert_eq!(got, vec![(1, true, true), (2, true, false), (3, true, true)]);
    assert_eq!((score.matched, score.total), (2, 3));
    assert_eq!(score.summary(), "2 of 3 answers match the key.");
    assert_eq!(score.marks[1].label(), "Does not match the key");
}

#[test]
fn score_with_a_blank_answer() {
    // B3a key B, B3b key 1/8. Blank, then 0.125 (= 1/8, value form).
    let b = shipped();
    let set = decode(&b, "E3-B-B3A-B3B").unwrap();
    let score = check(&b, &set, &["".to_string(), "0.125".to_string()]).unwrap();
    assert_eq!(score.marks[0].label(), "No answer");
    assert_eq!(score.marks[1].label(), "Matches the key");
    assert_eq!(score.summary(), "1 of 2 answers match the key.");
}

#[test]
fn wrong_number_of_answers_is_refused() {
    let b = shipped();
    let set = decode(&b, "E3-B-B3A-B3B").unwrap();
    assert_eq!(
        check(&b, &set, &["B".to_string()]).unwrap_err(),
        "This set has 2 questions, but 1 answer was given."
    );
}
