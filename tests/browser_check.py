#!/usr/bin/env python3
"""Browser check (Python Playwright) for dist/web in Chromium, Firefox and WebKit.

For each engine it serves dist/web on 127.0.0.1 at a free port, waits for
<body data-ready="yes"> (set after the wasm module and data/bank.json have
loaded), idles, then: sees a one-source warm-up refused, makes the A+B DOK 3
warm-up, swaps question 2, saves the file, checks the print view, opens the
saved file as a student, answers, and checks the score. Expected values were
worked out by hand from data/bank.json.

Fails on any console error, page error, failed request, HTTP error, or request
to another origin, and (privacy kit, builder-setting kits/privacy) on any
request at all after the page reported ready (blob: and data: allowed), a
Content-Security-Policy violation, a WebSocket, popup, worker or service
worker, or anything in cookies, localStorage, sessionStorage or IndexedDB at
the end. It also opens the page with WebAssembly removed and with compiling
refused (what a school policy blocking the JavaScript JIT does) and expects
the "this browser blocks WebAssembly" message; on a normal run it must stay
hidden.

A pass shows what the page did on this path, in these engines, on this run.

    python tests/browser_check.py               # the real check
    python tests/browser_check.py --break console   # plants a console error
    python tests/browser_check.py --break request   # plants an other-origin request
    python tests/browser_check.py --break score     # expects a wrong score

Each --break run must FAIL; that is how the check itself is watched failing.

Environment:
    BUILDER_ENGINES  space-separated engines (default "chromium firefox webkit")
    BUILDER_IDLE     seconds to idle after ready (default 10)
"""

import argparse
import functools
import http.server
import os
import pathlib
import sys
import threading
import urllib.parse

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parent.parent
SITE = ROOT / "dist" / "web"
ENGINES = os.environ.get("BUILDER_ENGINES", "chromium firefox webkit").split()
IDLE_MS = int(float(os.environ.get("BUILDER_IDLE", "10")) * 1000)

# A request the CSP blocks never becomes a request event, so the violation
# itself is recorded, from the first script on the page.
CSP_RECORDER = (
    "window.__csp = [];"
    "document.addEventListener('securitypolicyviolation',"
    " e => window.__csp.push(e.violatedDirective + ' ' + e.blockedURI));"
)
# Two ways a browser can refuse WebAssembly: the object is gone, or it is there
# and every compile and instantiate rejects.
BLOCKED_MODES = {
    "absent": "delete window.WebAssembly;",
    "refused": (
        "(() => { const no = () => Promise.reject(new WebAssembly.CompileError('blocked by policy'));"
        " WebAssembly.compile = no; WebAssembly.instantiate = no; WebAssembly.instantiateStreaming = no;"
        " WebAssembly.validate = () => false; })();"
    ),
}
BLOCKED_TEXT = "This tool could not start because this browser blocks WebAssembly"


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
    }

    def log_message(self, *args):
        pass


def serve():
    handler = functools.partial(Handler, directory=str(SITE))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server


def run_engine(p, engine, origin, plant, port, expect, problems):
    print(f"[{engine}] {origin}/")
    ready = {"yes": False}
    late = []  # requests after ready
    extras = {"sockets": [], "popups": [], "workers": []}

    def on_request(req):
        if ready["yes"] and not req.url.startswith(("blob:", "data:")):
            late.append(req.url)
        parts = urllib.parse.urlsplit(req.url)
        if parts.scheme == "data":
            return
        if parts.scheme == "blob":
            inner = urllib.parse.urlsplit(req.url[len("blob:"):])
            if f"{inner.scheme}://{inner.netloc}" != origin:
                problems.append(f"[{engine}] blob from another origin: {req.url}")
            return
        if f"{parts.scheme}://{parts.netloc}" != origin:
            problems.append(f"[{engine}] request to another origin: {req.url}")

    browser = getattr(p, engine).launch()
    # The request plant bypasses the page's CSP so the request really leaves
    # the page and the request listener (not the CSP) must catch it.
    context = browser.new_context(accept_downloads=True, bypass_csp=plant == "request")
    context.add_init_script(CSP_RECORDER)
    page = context.new_page()
    page.on("console", lambda m: problems.append(f"[{engine}] console error: {m.text}") if m.type == "error" else None)
    page.on("pageerror", lambda e: problems.append(f"[{engine}] page error: {e}"))
    page.on("request", on_request)
    page.on("requestfailed", lambda r: problems.append(f"[{engine}] request failed: {r.url} {r.failure}"))
    page.on("response", lambda r: problems.append(f"[{engine}] HTTP {r.status}: {r.url}") if r.status >= 400 else None)
    page.on("websocket", lambda ws: extras["sockets"].append(ws.url))
    page.on("worker", lambda w: extras["workers"].append(w.url))
    page.on("popup", lambda pg: extras["popups"].append(pg.url))

    if plant == "console":
        page.add_init_script("addEventListener('DOMContentLoaded', () => console.error('planted console error'))")
    if plant == "request":
        other = f"http://localhost:{port}/data/bank.json"  # same machine, different origin
        page.add_init_script(f"addEventListener('DOMContentLoaded', () => fetch('{other}').catch(() => null))")

    try:
        page.goto(origin + "/index.html")
        page.wait_for_selector("body[data-ready=yes]", state="attached", timeout=15000)
        ready["yes"] = True
        expect(page.locator("#wasm-blocked").is_hidden(), "blocked-WebAssembly message hidden when WebAssembly works")
        page.wait_for_timeout(IDLE_MS)
        expect(not late, f"no request during {IDLE_MS // 1000}s idle after ready", late)
        expect(page.locator("input[name=source]").count() == 3, "three sources offered")
        notice = page.inner_text("#notice")
        expect(notice.startswith("SAMPLE CONTENT"), "sample notice shown", notice)

        def set_ids(box):
            return page.eval_on_selector_all(f"{box} li.question", "els => els.map(e => e.dataset.id)")

        # 1. Warm-up for source B at DOK 3: refused, only B3a and B3b match.
        page.check("input[name=source][value=B]")
        page.check("input[name=level][value='3']")
        page.check("input[name=kind][value=warmup]")
        page.click("#make")
        msg = page.inner_text("#teacher-message")
        expect("Only 2 questions in the bank match source B at DOK 3, and a warm-up needs 3." in msg,
               "B-only DOK 3 warm-up refused with the shortfall", msg)
        expect(set_ids("#teacher-set") == [], "no set drawn after a refusal", set_ids("#teacher-set"))

        # 2. Warm-up for A and B at DOK 3: A3a, B3a, A3b.
        page.check("input[name=source][value=A]")
        page.click("#make")
        ids = set_ids("#teacher-set")
        expect(ids == ["A3a", "B3a", "A3b"], "A+B DOK 3 warm-up is A3a, B3a, A3b", ids)
        code = page.inner_text("#teacher-set .code")
        expect(code == "Set code: W3-AB-A3A-B3A-A3B", "set code shown", code)
        keys = page.locator("#teacher-set .key").count()
        expect(keys == 3, "teacher sees three keys", keys)

        # 3. Swap question 2: B3b comes in.
        page.click("#teacher-set li.question[data-number='2'] button.swap")
        ids = set_ids("#teacher-set")
        expect(ids == ["A3a", "B3b", "A3b"], "swap of question 2 gives B3b", ids)

        # 4. Save as file: the student copy, with the code and no key.
        with page.expect_download() as info:
            page.click("#save")
        download = info.value
        saved = pathlib.Path(download.path()).read_text(encoding="utf-8")
        expect(download.suggested_filename == "warm-up-W3-AB-A3A-B3B-A3B.txt", "file name",
               download.suggested_filename)
        expect("Set code: W3-AB-A3A-B3B-A3B" in saved, "file carries the set code")
        expect("Key:" not in saved and "matches)" not in saved, "file has no key")

        # 5. Print view: controls and keys hidden, answer lines shown.
        page.emulate_media(media="print")
        expect(page.locator("#make-form").is_hidden(), "print hides the form")
        expect(page.locator("#teacher-set .key").first.is_hidden(), "print hides the key")
        expect(page.locator("#teacher-set .answer-line").first.is_visible(), "print shows answer lines")
        page.emulate_media(media="screen")

        # 6. Student opens the saved file, answers, and checks.
        page.click("#mode-student")
        page.set_input_files("#file-input", download.path())
        page.wait_for_selector("#student-set li.question", timeout=5000)
        ids = set_ids("#student-set")
        expect(ids == ["A3a", "B3b", "A3b"], "student file opens the same set", ids)
        expect(page.locator("#student-set .key").count() == 0, "student sees no key")
        # By hand: 4/6 equals key 2/3 (match); 1/4 is not 1/8 (no match); B is key B (match).
        page.fill("#student-set li[data-number='1'] input[type=text]", "4/6")
        page.fill("#student-set li[data-number='2'] input[type=text]", "1/4")
        page.check("#student-set li[data-number='3'] input[value=B]")
        page.click("#check")
        summary = page.inner_text("#summary")
        want = "3 of 3 answers match the key." if plant == "score" else "2 of 3 answers match the key."
        expect(summary == want, f"score is '{want}'", summary)
        marks = page.eval_on_selector_all("#student-set .mark", "els => els.map(e => e.textContent)")
        expect(marks == ["Matches the key", "Does not match the key", "Matches the key"], "marks per question", marks)

        # 7. A typed code, in lower case, opens a set too.
        page.fill("#code-input", "e3-b-b3a-b3b")
        page.click("#open")
        ids = set_ids("#student-set")
        expect(ids == ["B3a", "B3b"], "typed code opens the B DOK 3 exit ticket", ids)
        expect(page.inner_text("#summary") == "", "old score cleared", page.inner_text("#summary"))
        help_save = page.inner_text("#teacher-actions")
        expect("your school's IT can allow downloads" in help_save,
               "save controls say a school device may block saving files", help_save)
        help_open = page.inner_text("#open-form")
        expect("your school's IT can allow file selection" in help_open,
               "open controls say a school device may block opening files", help_open)
        page.wait_for_timeout(500)
        csp = page.evaluate("() => window.__csp || []")
        expect(not csp, "no Content-Security-Policy violation", csp)
        stored = page.evaluate("""async () => {
            const idb = indexedDB.databases ? (await indexedDB.databases()).length : 0;
            return {cookie: document.cookie, local: localStorage.length, session: sessionStorage.length, idb};
        }""")
        cookies = context.cookies()
        expect(stored == {"cookie": "", "local": 0, "session": 0, "idb": 0} and not cookies,
               "nothing kept in cookies or browser storage", [stored, cookies])
        sw = page.evaluate("() => navigator.serviceWorker ? navigator.serviceWorker.getRegistrations().then(r => r.length) : 0")
        expect(not extras["sockets"] and not extras["popups"] and not extras["workers"] and sw == 0,
               "no WebSocket, popup, worker or service worker", [extras, sw])
        expect(not late, "no request of any kind after the page reported ready", late)
    except Exception as exc:  # a missing element or a timeout
        expect(False, f"[{engine}] every step ran", f"step raised: {exc}")
    finally:
        browser.close()


def run_blocked(p, engine, origin, mode, expect):
    """Open the page in a browser that refuses WebAssembly; expect the message."""
    browser = getattr(p, engine).launch()
    context = browser.new_context()
    context.add_init_script(BLOCKED_MODES[mode])
    page = context.new_page()
    errors = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    page.goto(origin + "/index.html")
    try:
        page.wait_for_selector("#wasm-blocked", state="visible", timeout=10000)
        shown = page.inner_text("#wasm-blocked")
    except Exception as exc:  # the message never appeared
        shown = f"(not shown: {exc.__class__.__name__})"
    expect(BLOCKED_TEXT in shown and "sgilson7.github.io" in shown,
           f"WebAssembly {mode}: the page says the browser blocks WebAssembly and IT can allow it", shown[:80])
    expect(page.locator("main").is_hidden(), f"WebAssembly {mode}: the tool's controls are hidden")
    expect(not errors, f"WebAssembly {mode}: no uncaught page error", errors)
    browser.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--break", dest="plant", choices=["console", "request", "score"])
    args = parser.parse_args()

    if not (SITE / "index.html").exists():
        print("FAIL: dist/web/index.html is missing; run ./build.sh first")
        return 1

    server = serve()
    port = server.server_address[1]
    origin = f"http://127.0.0.1:{port}"
    problems = []  # console errors, page errors, bad requests
    failures = []  # expectations not met
    passed = []
    engine_now = {"name": ""}

    def expect(ok, what, got=None):
        label = f"[{engine_now['name']}] {what}"
        if ok:
            passed.append(label)
        else:
            failures.append(f"{label} (got: {got!r})")

    with sync_playwright() as p:
        for engine in ENGINES:
            engine_now["name"] = engine
            run_engine(p, engine, origin, args.plant, port, expect, problems)
            for mode in BLOCKED_MODES:
                run_blocked(p, engine, origin, mode, expect)
    server.shutdown()

    for line in passed:
        print(f"ok    {line}")
    for line in failures:
        print(f"FAIL  {line}")
    for line in problems:
        print(f"FAIL  {line}")
    if failures or problems:
        print(f"BROWSER CHECK FAILED: {len(failures)} expectation(s), {len(problems)} console/request problem(s)")
        return 1
    print(f"BROWSER CHECK PASSED in {', '.join(ENGINES)}: {len(passed)} expectations, no console errors, "
          "no other-origin requests, no request after ready, nothing stored")
    return 0


if __name__ == "__main__":
    sys.exit(main())
