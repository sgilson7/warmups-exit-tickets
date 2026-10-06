// Boundary tests (PLAN step 8). They read the project's own files and fail on:
// a core dependency other than serde and serde_json; a float, hash map, clock
// or input/output in the core's source; a shim function that does more than
// pass values to the core; a page file that could reach another origin, store
// student data, or carry a copy of a rule.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// Remove comments and the contents of string and character literals, keeping
/// line breaks so line numbers still match. `quotes` lists the string quote
/// characters of the language.
fn strip(src: &str, quotes: &[char]) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            i += 2;
            while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            i += 2;
        } else if quotes.contains(&c) {
            out.push(c);
            i += 1;
            while i < chars.len() && chars[i] != c {
                if chars[i] == '\\' {
                    i += 1;
                }
                if chars.get(i) == Some(&'\n') {
                    out.push('\n');
                }
                i += 1;
            }
            out.push(c);
            i += 1;
        } else if c == '\'' && quotes.contains(&'"') && !quotes.contains(&'\'') {
            // Rust: a char literal ('x' or '\n') is blanked; a lifetime ('a) is kept.
            let close = if next == Some('\\') { chars[i + 2..].iter().position(|&x| x == '\'').map(|p| i + 2 + p) } else if chars.get(i + 2) == Some(&'\'') { Some(i + 2) } else { None };
            match close {
                Some(end) => {
                    out.push_str("' '");
                    i = end + 1;
                }
                None => {
                    out.push(c);
                    i += 1;
                }
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Words (identifiers and number literals) with their line numbers.
fn words(code: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (n, line) in code.lines().enumerate() {
        let mut word = String::new();
        let chars: Vec<char> = line.chars().chain(std::iter::once(' ')).collect();
        for (i, &ch) in chars.iter().enumerate() {
            // A '.' belongs to a number only between digits (1.5), not in 0..n.
            let keeps_number = ch == '.'
                && word.chars().next().is_some_and(|f| f.is_ascii_digit())
                && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit());
            if ch.is_alphanumeric() || ch == '_' || keeps_number {
                word.push(ch);
            } else if !word.is_empty() {
                out.push((n + 1, std::mem::take(&mut word)));
            }
        }
    }
    out
}

#[test]
fn core_depends_only_on_serde_and_serde_json() {
    let toml = read("crates/core/Cargo.toml");
    let mut names = Vec::new();
    let mut in_deps = false;
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') {
            let header = line.trim_matches(|c| c == '[' || c == ']');
            in_deps = header.ends_with("dependencies");
            if let Some((section, name)) = header.rsplit_once('.') {
                if section.ends_with("dependencies") {
                    names.push(name.trim_matches('"').to_string());
                    in_deps = false;
                }
            }
        } else if in_deps && !line.is_empty() && !line.starts_with('#') {
            let name = line.split('=').next().unwrap().trim().trim_matches('"');
            names.push(name.to_string());
        }
    }
    let allowed = ["serde", "serde_json"];
    let outside: Vec<&String> = names.iter().filter(|n| !allowed.contains(&n.as_str())).collect();
    assert!(outside.is_empty(), "crates/core/Cargo.toml lists dependencies outside {allowed:?}: {outside:?}");
    assert_eq!(names, ["serde", "serde_json"], "expected exactly serde and serde_json");
}

#[test]
fn core_source_has_no_float_hash_map_clock_or_io() {
    let forbidden_words = [
        "f32", "f64", "HashMap", "HashSet", "hash_map", "hash_set", "RandomState", "Instant", "SystemTime",
        "UNIX_EPOCH", "Duration", "time", "fs", "io", "net", "env", "process", "thread", "println", "print",
        "eprintln", "eprint", "dbg", "include_str", "include_bytes", "File", "stdin", "stdout",
    ];
    let dir = root().join("crates/core/src");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    let mut found = Vec::new();
    let mut scanned = 0;
    for path in &files {
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        scanned += 1;
        let code = strip(&fs::read_to_string(path).unwrap(), &['"']);
        let name = path.file_name().unwrap().to_string_lossy();
        for (line, w) in words(&code) {
            let float_literal = w.starts_with(|c: char| c.is_ascii_digit())
                && (w.contains('.') || w.ends_with("f32") || w.ends_with("f64"));
            if forbidden_words.contains(&w.as_str()) || float_literal {
                found.push(format!("crates/core/src/{name}:{line}: {w}"));
            }
        }
    }
    assert!(scanned >= 8, "expected at least 8 core source files, scanned {scanned}");
    assert!(found.is_empty(), "float, hash map, clock or input/output in the core:\n{}", found.join("\n"));
}

#[test]
fn shim_only_passes_values_to_the_core() {
    let code = strip(&read("crates/wasm/src/lib.rs"), &['"']);
    let mut bodies = Vec::new();
    let mut rest = code.as_str();
    while let Some(at) = rest.find("fn ") {
        let after = &rest[at..];
        let open = after.find('{').expect("fn without body");
        let mut depth = 0;
        let mut end = open;
        for (i, ch) in after[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let signature = after[..open].trim().to_string();
        bodies.push((signature, after[open + 1..end].trim().to_string()));
        rest = &after[end..];
    }
    assert!(bodies.len() >= 6, "expected the 6 API functions in the shim, found {}", bodies.len());
    let deciding = ["if", "match", "for", "while", "loop", "let", "else", "unwrap", "expect", "unwrap_or"];
    for (signature, body) in &bodies {
        assert!(body.starts_with("warmups_core::api::"), "`{signature}` must only call warmups_core::api, body: {body}");
        let bad: Vec<String> = words(body).into_iter().map(|(_, w)| w).filter(|w| deciding.contains(&w.as_str())).collect();
        assert!(bad.is_empty(), "`{signature}` decides something: {bad:?}");
        for sym in [";", "?", "=>", "==", "!=", "&&", "||", "<", ">"] {
            assert!(!body.contains(sym), "`{signature}` contains `{sym}`: {body}");
        }
    }
}

#[test]
fn page_reaches_no_other_origin_stores_nothing_and_copies_no_rule() {
    let html = read("web/index.html");
    let js = read("web/app.js");
    let css = read("web/style.css");
    let mut found = Vec::new();
    for (name, text) in [("web/index.html", &html), ("web/app.js", &js), ("web/style.css", &css)] {
        for (n, line) in text.lines().enumerate() {
            for bad in ["://", "\"//", "'//", "(//", "url("] {
                if line.contains(bad) {
                    found.push(format!("{name}:{}: other-origin or external URL `{bad}`", n + 1));
                }
            }
            for bad in [
                "XMLHttpRequest", "WebSocket", "EventSource", "sendBeacon", "localStorage", "sessionStorage",
                "indexedDB", "document.cookie", "serviceWorker", "importScripts", "import(", "Math.random", "Date",
                "innerHTML", "@import",
            ] {
                if line.contains(bad) {
                    found.push(format!("{name}:{}: `{bad}`", n + 1));
                }
            }
        }
    }
    // The only fetch is the bank, from the page's own folder.
    let fetches = js.matches("fetch(").count();
    let bank_fetches = js.matches("fetch('data/bank.json')").count();
    if fetches != bank_fetches || fetches != 1 {
        found.push(format!("web/app.js: {fetches} fetch calls, {bank_fetches} of them fetch('data/bank.json'); expected exactly 1"));
    }
    // No number other than 0 and 1 in the page script: set sizes, DOK levels and
    // scores come from the core. Backticks are not used, so quotes are ' and ".
    assert!(!js.contains('`'), "web/app.js should not use template strings (the scan cannot read them)");
    for (line, w) in words(&strip(&js, &['\'', '"'])) {
        if w.starts_with(|c: char| c.is_ascii_digit()) && w != "0" && w != "1" {
            found.push(format!("web/app.js:{line}: number {w} (rules belong in the core)"));
        }
    }
    if !html.contains("Content-Security-Policy") || !html.contains("default-src 'self'") {
        found.push("web/index.html: no Content-Security-Policy limiting loads to the page's own origin".to_string());
    }
    assert!(found.is_empty(), "page boundary broken:\n{}", found.join("\n"));
}
