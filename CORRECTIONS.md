# CORRECTIONS: failing tests and checks

Every failure is written here: the file and line, what was expected, what
happened, and the fix. Deliberate breaks (rule f: each new test is watched
failing once before it is kept) are marked "planted". Fixes to rules are made
in the core, never in the page.

| # | Test or check | File:line | Expected | Happened | Fix |
|---|---------------|-----------|----------|----------|-----|
| 1 | planted: all 15 tests in `bank_rules.rs` | crates/core/src/bank.rs:88 (`problems()` returned `["planted"]`), :77 (JSON error text replaced) | each test's stated problems / a clean load | all 15 FAILED (e.g. bank_rules.rs:45 left `["planted"]` right `["Question id \"a1A\" is used twice."]`; common/mod.rs:10 shipped bank refused) | planted lines removed; 15 pass |
| 2 | planted: all 4 tests in `fraction_rules.rs` | crates/core/src/fraction.rs:27 (`normalize` returned input), :76 (`parse` returned 0 for anything) | `"6/8"`, `5/1`, `None` for junk | all 4 FAILED (fraction_rules.rs:14 left `"  6 /  8 "`; :21 left `0/1` right `5/1`; :46 left `Some(..)` right `None`) | planted lines removed; 4 pass |
| 3 | planted: 13 of 14 tests in `select_rules.rs`, all 6 in `code_rules.rs` | crates/core/src/select.rs:208 (`make_set` always refused); crates/core/src/code.rs:11, :18, :67 (`encode` empty, `decode` refused, `find_code` empty) | the hand-computed sets and codes | 19 FAILED (e.g. select_rules.rs:21 wrong error; code_rules.rs:45 left `"planted"`). One test PASSED: `every_swap_keeps_sources_and_level_and_never_repeats` | see row 4 |
| 4 | real defect in a test: `every_swap_keeps_sources_and_level_and_never_repeats` | crates/core/tests/select_rules.rs:139 | a failure when no set can be made | it passed, because `let Ok(..) else continue` skipped every case: the test could pass while checking nothing | test now counts the sets it checks and asserts the hand-computed total 44 (16 warm-ups + 28 exit tickets); re-run with the plant: FAILED at :162 left `0` right `44` |
| 5 | planted: swap repeats question 1 | crates/core/src/select.rs:241 | swaps never repeat a question | 3 FAILED: select_rules.rs:151 "A1a repeated", :89, :99 | planted line removed; 14 + 6 pass |
