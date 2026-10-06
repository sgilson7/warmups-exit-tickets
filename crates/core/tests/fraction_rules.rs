// Reading numbers written by students (PLAN step 2, used by step 6).
use warmups_core::fraction::{normalize, parse, Ratio};

fn value(text: &str) -> Ratio {
    parse(text).unwrap_or_else(|| panic!("{text:?} should parse")).value
}

fn r(num: i64, den: i64) -> Ratio {
    Ratio { num, den }
}

#[test]
fn normalize_trims_collapses_and_lowercases() {
    assert_eq!(normalize("  6 /  8 "), "6/8");
    assert_eq!(normalize("1   1/4"), "1 1/4");
    assert_eq!(normalize(" B "), "b");
}

#[test]
fn reads_whole_fraction_mixed_and_decimal() {
    assert_eq!(value("5"), r(5, 1));
    assert_eq!(value("6/8"), r(3, 4));
    assert_eq!(value("1 1/4"), r(5, 4));
    assert_eq!(value("0.75"), r(3, 4));
    assert_eq!(value(".5"), r(1, 2));
    assert_eq!(value("-1 1/2"), r(-3, 2));
    assert_eq!(value("0/5"), r(0, 1));
}

#[test]
fn simplest_form_is_judged_on_what_was_written() {
    let simplest = |t: &str| parse(t).unwrap().simplest;
    assert!(simplest("3/4"));
    assert!(simplest("5/4"));
    assert!(simplest("1 1/4"));
    assert!(simplest("7"));
    assert!(!simplest("6/8"));
    assert!(!simplest("4/1"));
    assert!(!simplest("1 2/8"));
    assert!(!simplest("0.75"));
}

#[test]
fn refuses_what_is_not_a_number() {
    for t in ["", "abc", "1/0", "1 5/4", "3/4/5", "1.", "1234567890", "1 1", "--3"] {
        assert_eq!(parse(t), None, "{t:?}");
    }
}
