# SECOND-ORDER notebook

Kinds: divergence (from the plan), finding, worklist (item for the teacher).
Status: done, open, teacher (assigned to the teacher).

| # | Kind | Note | Status |
|---|------|------|--------|
| 1 | finding | The bank shape the brief asks for (2 questions per source and level) can never fill a single-source warm-up (3 questions). The brief's own example, "warm-up for source B at DOK 3", has only B3a and B3b to draw on. Decision: refuse with a message that says 2 match and 3 are needed, rather than pad from another level or make a short set. | done |
| 2 | worklist | If single-source warm-ups are wanted, write a third question for each source and level (12 more), or accept that a warm-up needs two or more sources. | teacher |
| 3 | finding | A single-source exit ticket uses both matching questions, so nothing is left to swap in. The core says so and the swap button is disabled. | done |
| 4 | finding | A static page with no server ships the answer key to every device (it is in `data/bank.json`). A determined student can read it. Fine for warm-ups; not for graded tests. | teacher |
| 5 | worklist | DOK 3 and 4 questions ask for reasoning on paper; the tool checks only the final short answer. Review whether those items still earn their DOK label in a 5-minute warm-up. | teacher |
| 6 | divergence | The fraction reader (planned for step 6) was built in step 2, because checking that a bank's fraction keys are numbers needs it. | done |
| 7 | finding | `cargo test` stops at the first failing test binary, so a planted break in one file hid the failures of the next. Planted-break runs now use `--no-fail-fast` or name the test file. | done |
| 8 | finding | An exhaustive test that loops over "every set that can be made" passes when no set can be made. Guard: assert how many cases were checked (44, worked out by hand). | done |
| 9 | finding | The set code carries the chosen sources, not just the questions, so a teacher who reopens a code can keep swapping within the same sources. | done |
| 10 | finding | The page boundary test is a text scan, not a proof: it catches other-origin URLs, storage and network APIs, and numbers other than 0 and 1 in app.js (set sizes and DOK levels), but a rule written without numbers could still slip into the page. The browser check and code review cover the rest. | open |
| 11 | finding | A browser asks for /favicon.ico by itself; the 404 would be a console error. The page names an empty data: icon instead, so no extra request is made. | done |
