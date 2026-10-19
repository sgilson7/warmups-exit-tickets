# Warm-ups and exit tickets

A static browser tool for a middle-school teacher. The teacher picks one or
more source documents and a Depth of Knowledge (DOK) level from 1 to 4 and gets
a warm-up (3 questions) or an exit ticket (2 questions) from a question bank
written in advance. Any question can be swapped for another with the same
sources and level. The set can be printed or saved as a text file. A student
opens the same page on their own device, types the set code (or opens the saved
file), answers, and sees which answers match the key. Nothing is sent anywhere
and nothing is stored. The page carries Builder's privacy-kit
Content-Security-Policy.

If a browser blocks WebAssembly (some school devices do, through a policy that
blocks the JavaScript JIT), the page says so and says the school's IT can allow
it for sgilson7.github.io. Next to the save and open controls a short line says
that a school device may block saving or opening files.

The question bank in `data/bank.json` is SAMPLE content (3 sources x 4 DOK
levels x 2 questions on fractions), written to show how the tool works.

## Layout

| Path | What it is |
|------|------------|
| `crates/core` | Every rule: bank checks, set making, swaps, set codes, the saved file, answer checking. No input or output; depends only on serde and serde_json; no floats, hash maps or clocks. |
| `crates/wasm` | wasm-bindgen shim: six one-line functions that pass text to `warmups_core::api` and back. |
| `web/` | `index.html`, `app.js`, `style.css`: draws what the core sends. No framework, bundler or npm. |
| `data/bank.json` | The question bank. |
| `build.sh` | Builds the wasm module and assembles `dist/web/`. |
| `tests/browser_check.py` | Playwright check of `dist/web` in Chromium, Firefox and WebKit. |
| `tests/plant_leak.py`, `tests/prove-privacy.sh` | Privacy proof: plants 10 leaks, one at a time, in a copy and requires the checks to catch each. |
| `LICENSE` | MIT. |

## Build

Needs Rust stable with the `wasm32-unknown-unknown` target and
`wasm-bindgen-cli` 0.2.127.

```sh
./build.sh
```

This runs `cargo build --release --target wasm32-unknown-unknown -p
warmups-wasm`, then `wasm-bindgen --target web` into `dist/web/pkg/`, and
copies `web/*` and `data/*.json` into `dist/web/`.

## Test

```sh
cargo test                                   # 62 tests: rules, hand-computed cases, boundary scans
./build.sh                                   # the browser check reads dist/web
python tests/browser_check.py                # needs Python with Playwright (chromium, firefox, webkit)
PYTHON=python tests/prove-privacy.sh         # privacy proof, a few minutes
```

The browser check serves `dist/web` on 127.0.0.1 at a free port, makes the
A+B DOK 3 warm-up, swaps a question, saves the file, checks the print view,
opens the file as a student, answers, and expects "2 of 3 answers match the
key". It fails on any console error, page error or request to another origin,
any request at all after the page reported ready, a CSP violation, a
WebSocket, popup or worker, or anything left in cookies or browser storage. It
also opens the page with WebAssembly removed and with compiling refused and
expects the "blocks WebAssembly" message (hidden on a normal run).
`BUILDER_ENGINES` picks engines; `BUILDER_IDLE` sets the idle after ready
(default 10 s). The boundary test also requires exactly the privacy kit's
Content-Security-Policy and `referrer: no-referrer` in `web/index.html`.
`tests/prove-privacy.sh` plants each leak from `tests/plant_leak.py` (P1 a
same-origin request carrying the data, P2 localStorage, P3 an off-origin image,
P4 sendBeacon, P5 an off-origin fetch; each where the student opens a set and
where answers are checked) in a fresh copy and requires the boundary tests or
the browser check to catch it.
`--break console`, `--break request` and `--break score` plant a fault so the
check can be seen failing.

## Serve

`dist/web` is a plain static folder. Any static host works. Locally:

```sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory dist/web
```

then open http://127.0.0.1:8000/. The page must be served over http(s), not
opened as a file, because browsers do not load wasm modules from `file://`.
The server must send `.wasm` files as `application/wasm` (Python's server
does in recent versions; Python 3.14 was checked).

## Using it

- Teacher: tick one or more sources, choose a DOK level and a kind of set,
  press "Make the set". Each question has a Swap button and shows its key.
  "Print" prints the student copy (no keys, with answer lines). "Save as file"
  saves the student copy as a `.txt` file that ends with the set code.
- Student: choose "Student: answer a set", type the set code (any case) or open
  the saved file, answer, press "Check my answers".
- A warm-up for a single source cannot be made with this bank: each source has
  only 2 questions per level and a warm-up needs 3. The tool says so. Choose two
  or more sources, make an exit ticket, or add questions to the bank.

## Editing the question bank

Edit `data/bank.json` (or `dist/web/data/bank.json` on a built copy) and reload.
If the bank breaks a rule, the page lists every problem instead of making sets.
Rules: source ids are single capital letters; question ids are letters and
digits and unique; DOK is 1 to 4; every question has a prompt and a key; a
`choice` key is one of its option letters; a `fraction` key is a number and its
`form` is `value` (any equal number matches), `simplest` (equal and in lowest
terms) or `exact` (the same written form, e.g. `6/8`). Unknown fields are
refused. Changing questions changes what old set codes mean, so share new codes
after an edit.
