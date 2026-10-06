# PLAN: warm-ups and exit tickets by source and DOK

Each step names the decision, the reason for it, and the test that will show it
holds. Steps are in build order. Each core rule lands in the same commit as its
tests. Every new test is first run in a broken state (stub rule or wrong
expected value) and its failure is written in CORRECTIONS.md before it is kept
(rule f).

## 1. Workspace skeleton

- Decision: one Cargo workspace with `crates/core` (package `warmups-core`) and
  `crates/wasm` (package `warmups-wasm`, cdylib). `.gitignore` holds `target/`
  and `dist/`.
- Reason: the brief fixes this layout; one workspace lets a single `cargo test`
  run every test.
- Test: `cargo test` at the root builds both crates (checked in step 8 by the
  boundary tests, which read both crates).

## 2. Question bank format and validation (`data/bank.json`, `core::bank`)

- Decision: one file `data/bank.json` holding `bank_version`, a `notice` that
  marks the content as SAMPLE, three `sources` (id, title, text) and 24
  `questions` (id, source, dok, prompt, answer). Question ids read
  source-level-item, for example `B3a`.
- Decision: two answer kinds. `choice` (options plus a key letter) and
  `fraction` (a key such as `3/4` plus a `form`: `value` = any equal number,
  `simplest` = equal and written in lowest terms, `exact` = the same written
  form, used when the prompt asks for a given denominator).
- Decision: the core refuses a bank that breaks a rule and lists every problem:
  duplicate id (ignoring case), DOK outside 1-4, unknown source, empty prompt,
  choice with fewer than 2 options or a key that is not one of the option
  letters, fraction key that does not parse, unknown form.
- Reason: the page and every later rule can then trust the bank; a teacher who
  edits the JSON gets a list of what to fix instead of a broken page.
- Tests: the shipped bank loads, has 24 questions, exactly 2 per source and
  level, and is marked sample; one test per validation problem.

## 3. Selection rule (`core::select`)

- Decision: a warm-up holds 3 questions and an exit ticket 2. Candidates are the
  questions whose source is one of the chosen sources and whose DOK equals the
  chosen level exactly. Chosen sources are upper-cased, de-duplicated and put in
  bank order, so click order does not matter.
- Decision: candidate order is round-robin across the chosen sources (first
  question of each source, then the second of each), questions within a source
  ordered by id. The set is the first 3 (or 2) candidates. No randomness, so the
  same inputs always give the same set.
- Decision: when fewer candidates exist than the set needs, the core makes no
  set and says how many match and how many are needed. It does not pad from
  another level and does not shorten the set, because either would quietly
  break the brief's rules ("three questions", "the same sources and level").
- Reason: round-robin makes each chosen source appear when the set is big
  enough; strict matching keeps every question at the level the teacher chose.
- Tests (worked out by hand from the bank): warm-up B at DOK 3 is refused
  (2 match, 3 needed); exit ticket B at DOK 3 = B3a, B3b; warm-up A+B at DOK 3
  = A3a, B3a, A3b; warm-up A+B+C at DOK 1 = A1a, B1a, C1a; sources [C, A] give
  the same set as [A, C]; no source, unknown source and level 0 or 5 are
  refused.

## 4. Swap rule (`core::select`)

- Decision: swapping question n replaces it with the next candidate after it
  in candidate order that is not already in the set, wrapping round. If every
  candidate is already in the set, the core refuses and says so. Each question
  in a set view carries `can_swap`, decided by the core.
- Reason: deterministic, never duplicates, never leaves the sources and level.
- Tests: warm-up A+B DOK 3, swap 2 gives B3b, swap 2 again gives B3a back;
  exit ticket B DOK 3 cannot swap; question number 0 or 4 refused; every
  question after any swap still matches the sources and level.

## 5. Set code and set file (`core::code`, `core::sheet`)

- Decision: a set is named by a code such as `W3-AB-A3A-B3A-A3B` (kind and
  level, sources, question ids). The page keeps only this code. The core reads a
  code typed in any case with stray spaces, and refuses a code whose questions
  do not exist, repeat, have the wrong count, or do not match its sources and
  level.
- Decision: "save as file" gives a plain text file built by the core: title,
  source texts, questions, options and a `Set code:` line, without the key. The
  student page opens either a typed code or such a file (the core finds the
  code line).
- Reason: a student device needs the same set without any network; a code or a
  file the teacher shares through their usual channel does that, and the tool
  itself sends nothing.
- Tests: code of a hand-made set; round trip code to set to code; lower case
  and spaces accepted; each refusal; file text contains the code line and no
  key; opening the file text gives the same set.

## 6. Answer checking (`core::check`)

- Decision: answers are normalised (trim, collapse spaces, drop spaces around
  `/`, ignore case). Fractions are parsed with integers only: whole numbers,
  `a/b`, mixed numbers `w a/b`, decimals `0.75`, optional minus sign, with
  overflow guarded. Choice answers compare letters.
- Decision: a blank answer does not match. The result lists, per question,
  whether the answer matches the key, plus "n of m answers match the key". The
  student is not shown the key itself.
- Reason: integers keep results exact and the same on every device (rule e);
  the forms let a question ask for simplest form or a given denominator.
- Tests (hand-computed): `6/8` matches `3/4` by value but not under simplest;
  `1 1/4` and `5/4` both match a simplest key `1 1/4`; `0.75` matches `3/4` by
  value but not under simplest; `6 / 8` matches exact `6/8`; `b` matches key B;
  blank does not match; a given answer list scores 2 of 3.

## 7. API (`core::api`)

- Decision: every function the page needs takes and returns JSON text:
  `options`, `make_set`, `swap`, `open_set` (student view, no key), `check`,
  `set_file`. Replies are `{"ok":true,...}` or `{"ok":false,"error":...}`.
- Reason: the shim then only passes strings through, so it cannot decide
  anything.
- Tests: each API call on the shipped bank, including error replies.

## 8. Boundary tests (`crates/core/tests/boundary.rs`)

- Decision: tests that read files: the core's `Cargo.toml` may list only serde
  and serde_json; the core's `src/` (comments and strings removed) may not
  contain f32/f64 or float literals, HashMap/HashSet, clocks (std::time,
  Instant, SystemTime), or input/output (std::fs, std::io, std::net, std::env,
  print macros); every exported shim function is a single call into
  `warmups_core::api`; the page files contain no other-origin URL, no storage
  API, no network API other than the one same-origin fetch of the bank, and no
  number literal other than 0 and 1 (a rough guard against copying set sizes or
  DOK levels into the page).
- Tests: each scan is first broken by planting a violation and watched failing.

## 9. Shim (`crates/wasm`)

- Decision: `wasm-bindgen = "=0.2.127"`; six exported functions, each one line.
- Test: the shim scan in step 8; the browser check in step 12.

## 10. Page (`web/`)

- Decision: `index.html`, `app.js` (ES module), `style.css`. It fetches
  `data/bank.json` from its own origin, asks the core for everything, and draws
  with `textContent` only. A Content-Security-Policy meta tag limits every load
  to the page's own origin. Student answers live only in the page's inputs: no
  storage, no sending. Print uses print CSS that hides controls and keys.
- Test: web scan in step 8; browser check in step 12.

## 11. `build.sh`

- Decision: `cargo build --release --target wasm32-unknown-unknown -p
  warmups-wasm`, then `wasm-bindgen --target web` into `dist/web/pkg`, then copy
  `web/*` and `data/*.json` into `dist/web/`.
- Test: running it produces `dist/web/index.html`, `dist/web/pkg/*.wasm` and
  `dist/web/data/bank.json`; the browser check loads that folder.

## 12. Browser check (`tests/browser_check.py`)

- Decision: Python Playwright in Chromium; serves `dist/web` on 127.0.0.1 at a
  free port; makes the A+B DOK 3 warm-up (expects A3a, B3a, A3b), swaps
  question 2 (expects B3b), sees the B-only warm-up refused, saves the file,
  opens the set as a student, answers, and expects "2 of 3 answers match the
  key". Fails on any console error, page error, or request to another origin.
  A `--break` switch plants a console error or an other-origin request so the
  check can be watched failing.

## 13. README, HANDOFF, notebook

- README: build, test, serve. HANDOFF: built, open, what to check by hand.
  SECOND-ORDER.md kept throughout; three teacher-use assumptions checked at the
  end.
