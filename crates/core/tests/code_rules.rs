// Set codes (PLAN step 5).
mod common;

use common::shipped;
use warmups_core::code::{decode, encode, find_code};
use warmups_core::select::{make_set, Kind};

fn s(list: &[&str]) -> Vec<String> {
    list.iter().map(|x| x.to_string()).collect()
}

#[test]
fn code_of_the_a_b_dok_3_warm_up() {
    let set = make_set(&shipped(), Kind::WarmUp, &s(&["B", "A"]), 3).unwrap();
    assert_eq!(encode(&set), "W3-AB-A3A-B3A-A3B");
}

#[test]
fn code_of_the_b_dok_3_exit_ticket() {
    let set = make_set(&shipped(), Kind::ExitTicket, &s(&["B"]), 3).unwrap();
    assert_eq!(encode(&set), "E3-B-B3A-B3B");
}

#[test]
fn code_round_trips() {
    let b = shipped();
    let set = make_set(&b, Kind::WarmUp, &s(&["A", "B", "C"]), 4).unwrap();
    assert_eq!(decode(&b, &encode(&set)).unwrap(), set);
}

#[test]
fn code_may_be_typed_in_lower_case_with_spaces() {
    let b = shipped();
    let set = decode(&b, "  w3 - ab - a3a - b3b - a3b ").unwrap();
    assert_eq!(set.questions, s(&["A3a", "B3b", "A3b"]));
    assert_eq!(set.sources, s(&["A", "B"]));
    assert_eq!(set.kind, Kind::WarmUp);
    assert_eq!(set.level, 3);
}

#[test]
fn broken_codes_are_refused_with_a_reason() {
    let b = shipped();
    let not_a_code = "That is not a set code. A set code looks like W3-AB-A3A-B3A-A3B.";
    assert_eq!(decode(&b, "hello").unwrap_err(), not_a_code);
    assert_eq!(decode(&b, "X3-AB-A3A-B3A-A3B").unwrap_err(), not_a_code);
    assert_eq!(decode(&b, "W5-AB-A3A-B3A-A3B").unwrap_err(), not_a_code);
    assert_eq!(decode(&b, "W3-AB--B3A-A3B").unwrap_err(), not_a_code);
    assert_eq!(
        decode(&b, "W3-AB-A3A-B3A").unwrap_err(),
        "A warm-up code needs 3 questions, but this code has 2."
    );
    assert_eq!(
        decode(&b, "E3-AB-A3A-Z9Z").unwrap_err(),
        "The set code names question Z9Z, which is not in the question bank."
    );
    assert_eq!(
        decode(&b, "E3-AB-A3A-C3A").unwrap_err(),
        "Question C3A does not match the sources and DOK level in this set code."
    );
    assert_eq!(
        decode(&b, "E3-AB-A2A-A3A").unwrap_err(),
        "Question A2A does not match the sources and DOK level in this set code."
    );
    assert_eq!(decode(&b, "E3-AB-A3A-A3A").unwrap_err(), "The set code names question A3A twice.");
    assert_eq!(
        decode(&b, "E3-AD-A3A-A3B").unwrap_err(),
        "There is no source \"D\" in the question bank."
    );
}

#[test]
fn find_code_reads_the_set_code_line_of_a_file() {
    let text = "Warm-up\n1. A question\n\nSet code: W3-AB-A3A-B3A-A3B\nMore text\n";
    assert_eq!(find_code(text), "W3-AB-A3A-B3A-A3B");
    assert_eq!(find_code("  e3-b-b3a-b3b \n"), "e3-b-b3a-b3b");
}
