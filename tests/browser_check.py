#!/usr/bin/env python3
"""Browser check (Python Playwright, Chromium) for dist/web.

Serves dist/web on 127.0.0.1 at a free port, then: sees a one-source warm-up
refused, makes the A+B DOK 3 warm-up, swaps question 2, saves the file, checks
the print view, opens the saved file as a student, answers, and checks the
score. Expected values were worked out by hand from data/bank.json.

Fails on any console error, page error, failed request, HTTP error, or request
to another origin.

    python tests/browser_check.py               # the real check
    python tests/browser_check.py --break console   # plants a console error
    python tests/browser_check.py --break request   # plants an other-origin request
    python tests/browser_check.py --break score     # expects a wrong score

Each --break run must FAIL; that is how the check itself is watched failing.
"""

import argparse
import functools
import http.server
import pathlib
import sys
import threading
import urllib.parse

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parent.parent
SITE = ROOT / "dist" / "web"


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

    def expect(ok, what, got=None):
        if ok:
            passed.append(what)
        else:
            failures.append(f"{what} (got: {got!r})")

    def on_request(req):
        parts = urllib.parse.urlsplit(req.url)
        if parts.scheme == "data":
            return
        if parts.scheme == "blob":
            inner = urllib.parse.urlsplit(req.url[len("blob:"):])
            if f"{inner.scheme}://{inner.netloc}" != origin:
                problems.append(f"blob from another origin: {req.url}")
            return
        if f"{parts.scheme}://{parts.netloc}" != origin:
            problems.append(f"request to another origin: {req.url}")

    with sync_playwright() as p:
        browser = p.chromium.launch()
        # The request plant bypasses the page's CSP so the request really leaves
        # the page and the request listener (not the CSP) must catch it.
        context = browser.new_context(accept_downloads=True, bypass_csp=args.plant == "request")
        page = context.new_page()
        page.on("console", lambda m: problems.append(f"console error: {m.text}") if m.type == "error" else None)
        page.on("pageerror", lambda e: problems.append(f"page error: {e}"))
        page.on("request", on_request)
        page.on("requestfailed", lambda r: problems.append(f"request failed: {r.url} {r.failure}"))
        page.on("response", lambda r: problems.append(f"HTTP {r.status}: {r.url}") if r.status >= 400 else None)

        if args.plant == "console":
            page.add_init_script("addEventListener('DOMContentLoaded', () => console.error('planted console error'))")
        if args.plant == "request":
            other = f"http://localhost:{port}/data/bank.json"  # same machine, different origin
            page.add_init_script(f"addEventListener('DOMContentLoaded', () => fetch('{other}').catch(() => null))")

        try:
            page.goto(origin + "/index.html")
            page.wait_for_selector("body[data-ready=yes]", timeout=15000)
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
            want = "3 of 3 answers match the key." if args.plant == "score" else "2 of 3 answers match the key."
            expect(summary == want, f"score is '{want}'", summary)
            marks = page.eval_on_selector_all("#student-set .mark", "els => els.map(e => e.textContent)")
            expect(marks == ["Matches the key", "Does not match the key", "Matches the key"], "marks per question", marks)

            # 7. A typed code, in lower case, opens a set too.
            page.fill("#code-input", "e3-b-b3a-b3b")
            page.click("#open")
            ids = set_ids("#student-set")
            expect(ids == ["B3a", "B3b"], "typed code opens the B DOK 3 exit ticket", ids)
            expect(page.inner_text("#summary") == "", "old score cleared", page.inner_text("#summary"))
        except Exception as exc:  # a missing element or a timeout
            failures.append(f"step raised: {exc}")
        finally:
            browser.close()
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
    print(f"BROWSER CHECK PASSED: {len(passed)} expectations, no console errors, no other-origin requests")
    return 0


if __name__ == "__main__":
    sys.exit(main())
