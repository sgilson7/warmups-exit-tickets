# HANDOFF

## What was built

A static page where a teacher makes a warm-up (3 questions) or an exit ticket
(2 questions) from chosen source documents at one DOK level, swaps questions,
prints the student copy or saves it as a text file, and where a student opens
the same set by its code or by the saved file, answers it, and sees which
answers match the key. Nothing is sent and nothing is stored.

- `crates/core` holds every rule: bank checks, set making (round-robin across
  the chosen sources, exact DOK level), swapping (next unused matching
  question, wrapping round), set codes such as `W3-AB-A3A-B3A-A3B`, the saved
  file, and answer checking with integer-only fractions (forms `value`,
  `simplest`, `exact`, plus multiple choice). It depends only on serde and
  serde_json and uses no floats, hash maps, clocks, or input/output.
- `crates/wasm` is six one-line pass-through functions.
- `web/` draws the core's replies with `textContent`. Its Content-Security-Policy
  is Builder's privacy-kit policy (own origin only; no fonts, frames, workers
  or manifest) with `referrer: no-referrer`. If WebAssembly is blocked, the
  page says so and that school IT can allow it for sgilson7.github.io.
- `data/bank.json` holds 24 SAMPLE questions on fractions (3 sources x 4 levels
  x 2), each with a key and, for most, a worked note for the teacher.
- Checks: `cargo test` runs 62 tests (hand-computed sets, swaps, codes, marks
  and scores; bank rules; boundary scans of the core's Cargo.toml and source,
  the shim, and the page). `build.sh` produces `dist/web`.
  `tests/browser_check.py` passes in Chromium, Firefox and WebKit (34
  expectations per engine), with no console errors, no other-origin request,
  no request after ready, no CSP violation and nothing stored.
  `tests/prove-privacy.sh`: PRIVACY PROOF PASSED, 10 of 10 planted leaks caught. Every test and the browser check were
  seen failing once on purpose; see CORRECTIONS.md.

## What is open

1. **A warm-up for one source cannot be made.** The brief asks for 2
   questions per source and level, and a warm-up needs 3, so the brief's own
   example (warm-up, source B, DOK 3) is refused with "Only 2 questions in the
   bank match source B at DOK 3, and a warm-up needs 3." Fix by content (add a
   third question per source and level), not by code. The rule was not
   loosened, because padding from another level or making a 2-question warm-up
   would break the brief's other rules.
2. **The key is readable on student devices.** A static page has to ship
   `data/bank.json`, keys included. Fine for practice, not for graded work.
3. **DOK 3 and 4 reasoning is not checked.** Those items ask for work on paper;
   only the final short answer is compared with the key.
4. **Old set codes follow bank edits.** A code names question ids, so editing a
   question changes what an old code opens.
5. **No printed key.** Print gives the student copy; keys are on screen only.
6. **Engines, not devices.** Chromium, Firefox and WebKit (Playwright builds)
   were checked; real iPads and managed Chromebooks were not. The blocked
   WebAssembly message was emulated, not seen under a real school policy.
7. **The page boundary test is a text scan.** It catches URLs, storage and
   network APIs and stray numbers, but not every way a rule could be copied
   into the page.

Nothing here needed a network request, a third-party script or service, or
student data leaving the device. If scores should reach the teacher, that is
such a change: ask before building it.

## What the teacher should check by hand

- Read all 24 questions, options, keys and notes in `data/bank.json`. They are
  sample content. Check that each DOK label fits your class, especially DOK 4
  items squeezed into a warm-up.
- Decide the form rule for each fraction key: `simplest` accepts `5/4` and
  `1 1/4`; `exact` (A1b `6/8`, B1b `2/8`) accepts only that written form.
- Print one warm-up on the school printer. The sheet should show the sources,
  questions, answer lines and set code, and no keys.
- On a student device, open the page, type a set code, answer, and check. Also
  try opening a saved `.txt` file on that device.
- Confirm students should see only "matches" or "does not match", not the
  right answer.
- Decide whether single-source warm-ups matter. If they do, write 12 more
  questions (one per source and level) or accept two-source warm-ups.
