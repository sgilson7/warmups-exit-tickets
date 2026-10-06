# CORRECTIONS: failing tests and checks

Every failure is written here: the file and line, what was expected, what
happened, and the fix. Deliberate breaks (rule f: each new test is watched
failing once before it is kept) are marked "planted". Fixes to rules are made
in the core, never in the page.

| # | Test or check | File:line | Expected | Happened | Fix |
|---|---------------|-----------|----------|----------|-----|
| 1 | planted: all 15 tests in `bank_rules.rs` | crates/core/src/bank.rs:88 (`problems()` returned `["planted"]`), :77 (JSON error text replaced) | each test's stated problems / a clean load | all 15 FAILED (e.g. bank_rules.rs:45 left `["planted"]` right `["Question id \"a1A\" is used twice."]`; common/mod.rs:10 shipped bank refused) | planted lines removed; 15 pass |
| 2 | planted: all 4 tests in `fraction_rules.rs` | crates/core/src/fraction.rs:27 (`normalize` returned input), :76 (`parse` returned 0 for anything) | `"6/8"`, `5/1`, `None` for junk | all 4 FAILED (fraction_rules.rs:14 left `"  6 /  8 "`; :21 left `0/1` right `5/1`; :46 left `Some(..)` right `None`) | planted lines removed; 4 pass |
