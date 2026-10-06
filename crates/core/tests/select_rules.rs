// Selection and swap rules (PLAN steps 3 and 4). Expected sets were worked out
// by hand from data/bank.json: candidates are round-robin across the chosen
// sources (bank order A, B, C), ids in order within a source.
mod common;

use common::shipped;
use warmups_core::select::{alternatives, make_set, swap, Kind, ProblemSet, SetError};

fn s(list: &[&str]) -> Vec<String> {
    list.iter().map(|x| x.to_string()).collect()
}

fn ids(set: &ProblemSet) -> Vec<&str> {
    set.questions.iter().map(|q| q.as_str()).collect()
}

#[test]
fn warm_up_for_source_b_at_dok_3_is_refused_two_match_three_needed() {
    // Only B3a and B3b are B at DOK 3; a warm-up needs 3.
    let err = make_set(&shipped(), Kind::WarmUp, &s(&["B"]), 3).unwrap_err();
    assert_eq!(err, SetError::Shortfall { kind: Kind::WarmUp, sources: s(&["B"]), level: 3, matching: 2 });
    assert_eq!(
        err.message(),
        "Only 2 questions in the bank match source B at DOK 3, and a warm-up needs 3. \
         Choose another source as well, or make an exit ticket."
    );
}

#[test]
fn exit_ticket_for_source_b_at_dok_3_is_b3a_b3b() {
    let set = make_set(&shipped(), Kind::ExitTicket, &s(&["B"]), 3).unwrap();
    assert_eq!(ids(&set), ["B3a", "B3b"]);
}

#[test]
fn warm_up_for_a_and_b_at_dok_3_is_a3a_b3a_a3b() {
    // Candidate order: A3a, B3a, A3b, B3b. First three.
    let set = make_set(&shipped(), Kind::WarmUp, &s(&["A", "B"]), 3).unwrap();
    assert_eq!(ids(&set), ["A3a", "B3a", "A3b"]);
    assert_eq!(set.sources, s(&["A", "B"]));
}

#[test]
fn warm_up_for_all_sources_at_dok_1_takes_one_from_each() {
    let set = make_set(&shipped(), Kind::WarmUp, &s(&["A", "B", "C"]), 1).unwrap();
    assert_eq!(ids(&set), ["A1a", "B1a", "C1a"]);
}

#[test]
fn exit_ticket_for_a_and_c_at_dok_4_is_a4a_c4a() {
    let set = make_set(&shipped(), Kind::ExitTicket, &s(&["C", "A"]), 4).unwrap();
    assert_eq!(ids(&set), ["A4a", "C4a"]);
}

#[test]
fn source_order_case_and_repeats_do_not_matter() {
    let b = shipped();
    let one = make_set(&b, Kind::WarmUp, &s(&["c", "A", "a"]), 2).unwrap();
    let two = make_set(&b, Kind::WarmUp, &s(&["A", "C"]), 2).unwrap();
    assert_eq!(one, two);
    assert_eq!(ids(&one), ["A2a", "C2a", "A2b"]);
}

#[test]
fn same_inputs_give_the_same_set() {
    let b = shipped();
    let first = make_set(&b, Kind::WarmUp, &s(&["A", "B", "C"]), 3).unwrap();
    for _ in 0..5 {
        assert_eq!(make_set(&b, Kind::WarmUp, &s(&["A", "B", "C"]), 3).unwrap(), first);
    }
}

#[test]
fn missing_or_unknown_source_and_bad_level_are_refused() {
    let b = shipped();
    assert_eq!(make_set(&b, Kind::WarmUp, &[], 1), Err(SetError::NoSource));
    assert_eq!(make_set(&b, Kind::WarmUp, &s(&[" "]), 1), Err(SetError::NoSource));
    assert_eq!(make_set(&b, Kind::WarmUp, &s(&["D"]), 1), Err(SetError::UnknownSource("D".into())));
    assert_eq!(make_set(&b, Kind::WarmUp, &s(&["A", "B"]), 0), Err(SetError::BadLevel));
    assert_eq!(make_set(&b, Kind::WarmUp, &s(&["A", "B"]), 5), Err(SetError::BadLevel));
    assert_eq!(SetError::BadLevel.message(), "Choose a DOK level from 1 to 4.");
}

#[test]
fn swap_question_2_gives_b3b_then_b3a_again() {
    let b = shipped();
    let set = make_set(&b, Kind::WarmUp, &s(&["A", "B"]), 3).unwrap();
    let once = swap(&b, &set, 2).unwrap();
    assert_eq!(ids(&once), ["A3a", "B3b", "A3b"]);
    let twice = swap(&b, &once, 2).unwrap();
    assert_eq!(ids(&twice), ["A3a", "B3a", "A3b"]);
}

#[test]
fn swap_question_1_skips_questions_already_in_the_set() {
    // After A3a come B3a and A3b (both in the set), then B3b.
    let b = shipped();
    let set = make_set(&b, Kind::WarmUp, &s(&["A", "B"]), 3).unwrap();
    assert_eq!(ids(&swap(&b, &set, 1).unwrap()), ["B3b", "B3a", "A3b"]);
}

#[test]
fn single_source_exit_ticket_has_nothing_to_swap() {
    let b = shipped();
    let set = make_set(&b, Kind::ExitTicket, &s(&["B"]), 3).unwrap();
    assert_eq!(alternatives(&b, &set), 0);
    let err = swap(&b, &set, 1).unwrap_err();
    assert_eq!(
        err.message(),
        "No other question in the bank matches source B at DOK 3: every matching question is already in this set."
    );
}

#[test]
fn swap_number_must_be_in_the_set() {
    let b = shipped();
    let set = make_set(&b, Kind::WarmUp, &s(&["A", "B"]), 3).unwrap();
    assert_eq!(swap(&b, &set, 0), Err(SetError::BadNumber { number: 0 }));
    assert_eq!(swap(&b, &set, 4), Err(SetError::BadNumber { number: 4 }));
}

#[test]
fn alternatives_count_matching_questions_left_out() {
    let b = shipped();
    let three = make_set(&b, Kind::WarmUp, &s(&["A", "B", "C"]), 2).unwrap();
    assert_eq!(alternatives(&b, &three), 3);
    let two = make_set(&b, Kind::WarmUp, &s(&["A", "B"]), 2).unwrap();
    assert_eq!(alternatives(&b, &two), 1);
}

#[test]
fn every_swap_keeps_sources_and_level_and_never_repeats() {
    let b = shipped();
    let combos: [&[&str]; 7] = [&["A"], &["B"], &["C"], &["A", "B"], &["A", "C"], &["B", "C"], &["A", "B", "C"]];
    let mut checked = 0;
    for kind in Kind::ALL {
        for chosen in combos {
            for level in 1..=4u8 {
                let Ok(mut set) = make_set(&b, kind, &s(chosen), level) else { continue };
                checked += 1;
                for round in 0..6 {
                    let n = round % set.questions.len() + 1;
                    if let Ok(next) = swap(&b, &set, n) {
                        set = next;
                    }
                    let mut seen: Vec<&String> = Vec::new();
                    for id in &set.questions {
                        let q = b.question(id).unwrap();
                        assert_eq!(q.dok, level);
                        assert!(chosen.contains(&q.source.as_str()), "{id} not in {chosen:?}");
                        assert!(!seen.contains(&id), "{id} repeated");
                        seen.push(id);
                    }
                    assert_eq!(set.questions.len(), kind.size());
                }
            }
        }
    }
    // By hand: warm-ups fill only for 2 or 3 sources (4 combos x 4 levels = 16);
    // exit tickets fill for all 7 combos x 4 levels = 28. Total 44, so this test
    // cannot pass by checking nothing.
    assert_eq!(checked, 44);
}
