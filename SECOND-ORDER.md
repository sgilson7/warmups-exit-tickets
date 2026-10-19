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
| 12 | divergence | The page boundary test grew beyond the plan: it also refuses `innerHTML`, `Math.random`, `Date`, CSS `url(` and `@import`, and checks that the page's Content-Security-Policy limits loads to its own origin. | done |
| 13 | finding | Set codes name question ids, not question text. If the teacher edits a question in `data/bank.json`, an old code opens the edited question. `bank_version` is not part of the code. | teacher |
| 14 | worklist | Print gives the student copy only (keys hidden). A teacher who wants a printed key has to copy it from the screen. Decide whether a "print with key" option is wanted. | teacher |
| 15 | worklist | Simplest form accepts both `5/4` and `1 1/4`. If the class requires mixed numbers, that is a rule change in `crates/core/src/fraction.rs`. | teacher |
| 16 | finding | After checking, a student sees "matches" or "does not match" per question and the count, never the right answer, so they can try again. The brief asks only for "which answers match the key". | teacher |
| 17 | finding | The browser check ran in Chromium only (as the brief asks). Safari on iPads and Firefox were not tried. | open |
| 18 | worklist | Assumption 2 below: the set reaches students through the teacher's own channel (printed sheet with the set code, or the saved file). Confirm that fits the class. | teacher |
| 19 | worklist | Assumption 3 below: results stay on the student's screen; the teacher collects nothing through the tool. Gathering scores would need student data to leave the device, which the brief says to ask about first. | teacher |
| 20 | divergence | The CSP now follows Builder's privacy kit exactly (adds `font-src`, `frame-src`, `worker-src`, `manifest-src 'none'`) plus `<meta name="referrer" content="no-referrer">`. No extra directive was needed. `boundary.rs::page_sets_the_privacy_content_security_policy_and_no_referrer` fails on a missing, loosened or added directive. | done |
| 21 | divergence | The kit's browser checks were added to the existing `tests/browser_check.py`, which now runs Chromium, Firefox and WebKit (`BUILDER_ENGINES`, `BUILDER_IDLE`) and fails on any request after ready, a CSP violation, or anything in cookies or browser storage at the end. The `--break` modes are kept. | done |
| 22 | divergence | The "this browser blocks WebAssembly" message sits in `web/index.html`, hidden, and `app.js` unhides it. Every other message comes from the core, but this one cannot: the core is the WebAssembly that did not start. The page decides "blocked" by compiling the 8-byte empty module first, so a missing `pkg/` file still gets the old "could not start" line. | done |
| 23 | finding | A page cannot reliably tell when a school policy blocks downloads (Chrome `DownloadRestrictions`) or the file chooser (`AllowFileSelectionDialogs`); both fail silently. The save and open controls carry a static help line instead. | done |
| 24 | worklist | The blocked message and the help lines name `sgilson7.github.io`. A copy hosted elsewhere should change that host name in `web/index.html`. | teacher |
| 25 | finding | The blocked case is emulated in the browser check (WebAssembly deleted, or compile and instantiate rejected). It was not seen on a real Chromebook under `DefaultJavaScriptJitSetting = block`. | open |
| 26 | finding | Privacy proof (`tests/prove-privacy.sh`): 10 planted leaks, all caught by the browser check; P1, P2, P4 and P5 also by the boundary scan. Scripts live in `tests/` because the repository has no `scripts/` folder. | done |

## Three assumptions about how a teacher will use the tool, checked against the brief

1. **The teacher will sometimes want a warm-up from one source.**
   Brief: "picks one or more source documents" and the example "which
   questions a warm-up for source B at DOK 3 contains". Against the brief's own
   bank shape ("3 source documents x 4 DOK levels x 2 questions") this cannot be
   met: one source at one level has 2 questions and a warm-up needs 3. The
   brief contradicts itself here. The tool keeps the stated rules (three
   questions, same sources and level) and refuses with a message that says how
   many match and what to do. Result: assumption holds, the brief cannot
   satisfy it; rows 1 and 2, assigned to the teacher.

2. **The set reaches students through a channel the teacher already has (a
   printed sheet with its set code, or the saved file posted to the class), not
   through the tool.** Brief: "A student opens the same page on their own
   device" and "Nothing is sent anywhere". The brief does not say how the set
   gets to the student; sending it would need a network request, which the
   brief says not to add without asking. The set code on the printed sheet and
   the saved file both work offline. Result: consistent with the brief; the
   delivery channel is the teacher's choice (status: teacher).

3. **Students check their own answers for practice; the teacher does not
   collect scores through the tool.** Brief: the student "sees which answers
   match the key", and "any student data beyond what stays on the student's own
   device" is something I may not decide alone. The tool shows results only on
   the student's screen and keeps nothing after the page closes. If the teacher
   expects to gather scores, that needs student data to leave the device: stop
   and ask, do not build. Result: consistent with the brief; flagged in HANDOFF
   (status: teacher).
