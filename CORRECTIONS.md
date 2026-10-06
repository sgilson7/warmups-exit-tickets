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
| 6 | planted: all 8 tests in `check_rules.rs` | crates/core/src/check.rs:9 (`answer_matches` always true), :57 (answer-count check switched off) | the hand-computed marks and scores | 8 FAILED (check_rules.rs:75 score 3 of 3 instead of 2 of 3; :97 wrong count accepted) | planted lines removed; 8 pass |
| 7 | planted: all 9 tests in `api_rules.rs` | crates/core/src/api.rs:14 (`reply` returned `{}`) | each reply's fields | 9 FAILED (api_rules.rs:19 `ok` missing, :31, :51 ...) | planted line removed; 9 pass |
| 8 | planted: `core_depends_only_on_serde_and_serde_json` | crates/core/Cargo.toml (added `itoa = "1"`) | failure naming the extra dependency | FAILED at boundary.rs:119: `lists dependencies outside ["serde", "serde_json"]: ["itoa"]` | line removed; passes |
| 9 | planted: `core_source_has_no_float_hash_map_clock_or_io` | crates/core/src/lib.rs:14-16 (an `f64` constant, a `HashMap` type, `std::time::Instant` and `println!`) | failure listing each by file and line | FAILED at boundary.rs:151 listing lib.rs:14 f64 and 0.5, :15 HashMap, :16 time, Instant, println | lines removed; passes |
| 10 | first run of `shim_only_passes_values_to_the_core` (shim not yet written) | crates/core/tests/boundary.rs:181 | 6 shim functions | FAILED: found 0 | shim written: 6 one-line functions; passes |
| 11 | planted: shim decides something | crates/wasm/src/lib.rs:19 (`if number == 0 { 1 } else { number }`) | failure | FAILED at boundary.rs:186: `fn swap ... decides something: ["if", "else"]` | line restored; passes |
| 12 | first run of the page boundary test (page not yet written) | crates/core/tests/boundary.rs:15 | page files | FAILED: cannot read web/index.html | page written; passes |
| 13 | planted: page reaches out, stores, and copies a rule | web/app.js:217-219 (`fetch('https://...')`, `localStorage`, `const WARM_UP_SIZE = 3`) | failure listing all three | FAILED at boundary.rs:234: `app.js:217 ://`, `:218 localStorage`, `2 fetch calls`, `:219 number 3` | lines removed; passes |
| 14 | planted: browser check `--break console` | tests/browser_check.py (init script calls `console.error`) | check FAILS | FAILED: `console error: planted console error` | plant is opt-in only; plain run passes |
| 15 | planted: browser check `--break request` (CSP bypassed so the request leaves the page) | tests/browser_check.py (init script fetches `http://localhost:<port>/data/bank.json`, another origin) | check FAILS | FAILED: `request to another origin: http://localhost:53467/data/bank.json`, plus the CORS console error | plain run passes |
| 16 | planted: browser check `--break score` | tests/browser_check.py (expects `3 of 3`) | check FAILS | FAILED: `score is '3 of 3 ...' (got: '2 of 3 answers match the key.')` | plain run passes: 20 expectations |
