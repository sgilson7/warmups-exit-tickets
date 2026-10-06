//! The only functions the page reaches (through the shim). Each takes text and
//! returns JSON text: `{"ok":true,...}` or `{"ok":false,"error":"..."}`. The
//! page draws these replies and decides nothing.

use crate::bank::{self, letter, Answer, Bank, LEVELS};
use crate::check;
use crate::code::{decode, encode, find_code};
use crate::select::{self, alternatives, level_label, Kind, ProblemSet, SetError};
use crate::sheet::{file_name, file_text, hint, key_text, title};
use serde::Deserialize;
use serde_json::{json, Value};

fn reply(bank_json: &str, f: impl FnOnce(&Bank) -> Result<Value, String>) -> String {
    let bank = match bank::load(bank_json) {
        Ok(b) => b,
        Err(problems) => {
            return json!({
                "ok": false,
                "error": "The question bank has problems, so no set can be made. Fix these in data/bank.json:",
                "problems": problems,
            })
            .to_string()
        }
    };
    match f(&bank) {
        Ok(mut body) => {
            body["ok"] = Value::Bool(true);
            body.to_string()
        }
        Err(error) => json!({ "ok": false, "error": error }).to_string(),
    }
}

/// Choices for the teacher's form: sources, DOK levels and kinds of set.
pub fn options(bank_json: &str) -> String {
    reply(bank_json, |b| {
        let sources: Vec<Value> = b.sources.iter().map(|s| json!({ "id": s.id, "title": s.title })).collect();
        let levels: Vec<Value> = LEVELS
            .iter()
            .map(|l| json!({ "value": l.to_string(), "label": format!("DOK {l}: {}", level_label(*l)), "default": *l == LEVELS[0] }))
            .collect();
        let kinds: Vec<Value> = Kind::ALL
            .iter()
            .map(|k| json!({ "value": k.id(), "label": format!("{} ({} questions)", k.label(), k.size()), "default": *k == Kind::WarmUp }))
            .collect();
        Ok(json!({
            "notice": b.notice,
            "bank_version": b.bank_version,
            "sources": sources,
            "levels": levels,
            "kinds": kinds,
        }))
    })
}

#[derive(Deserialize)]
struct SetRequest {
    kind: String,
    sources: Vec<String>,
    level: String,
}

/// Make a set from the teacher's choices. Reply: the teacher view (with key).
pub fn make_set(bank_json: &str, request_json: &str) -> String {
    reply(bank_json, |b| {
        let req: SetRequest =
            serde_json::from_str(request_json).map_err(|e| format!("The request could not be read: {e}"))?;
        let kind = Kind::from_id(&req.kind).ok_or_else(|| SetError::NoKind.message())?;
        let level: u8 = req.level.trim().parse().map_err(|_| SetError::BadLevel.message())?;
        let set = select::make_set(b, kind, &req.sources, level).map_err(|e| e.message())?;
        Ok(json!({ "set": view(b, &set, true) }))
    })
}

/// Swap question `number` (1 = first) of the set named by `code`.
pub fn swap(bank_json: &str, code: &str, number: u32) -> String {
    reply(bank_json, |b| {
        let set = decode(b, code)?;
        let next = select::swap(b, &set, number as usize).map_err(|e| e.message())?;
        Ok(json!({ "set": view(b, &next, true) }))
    })
}

/// Open a set for a student from a typed code or a saved file's text.
/// Reply: the student view (no key).
pub fn open_set(bank_json: &str, code_or_file: &str) -> String {
    reply(bank_json, |b| {
        let set = decode(b, find_code(code_or_file))?;
        Ok(json!({ "set": view(b, &set, false) }))
    })
}

/// Check a student's answers (a JSON list of strings, one per question).
pub fn check(bank_json: &str, code: &str, answers_json: &str) -> String {
    reply(bank_json, |b| {
        let set = decode(b, code)?;
        let answers: Vec<String> =
            serde_json::from_str(answers_json).map_err(|e| format!("The answers could not be read: {e}"))?;
        let score = check::check(b, &set, &answers)?;
        let marks: Vec<Value> = score
            .marks
            .iter()
            .map(|m| {
                json!({ "number": m.number, "id": m.id, "answered": m.answered, "matches": m.matches, "label": m.label() })
            })
            .collect();
        Ok(json!({
            "result": { "marks": marks, "matched": score.matched, "total": score.total, "summary": score.summary() }
        }))
    })
}

/// The student copy of the set as a text file: its name and its text.
pub fn set_file(bank_json: &str, code: &str) -> String {
    reply(bank_json, |b| {
        let set = decode(b, code)?;
        Ok(json!({ "file_name": file_name(&set), "text": file_text(b, &set) }))
    })
}

fn view(b: &Bank, set: &ProblemSet, teacher: bool) -> Value {
    let spare = alternatives(b, set);
    let sources: Vec<Value> = set
        .sources
        .iter()
        .filter_map(|id| b.source(id))
        .map(|s| json!({ "id": s.id, "title": s.title, "text": s.text }))
        .collect();
    let questions: Vec<Value> = set
        .questions
        .iter()
        .enumerate()
        .filter_map(|(i, id)| b.question(id).map(|q| (i, q)))
        .map(|(i, q)| {
            let (input, options): (&str, Vec<Value>) = match &q.answer {
                Answer::Choice { options, .. } => (
                    "choice",
                    options.iter().enumerate().map(|(j, o)| json!({ "letter": letter(j), "text": o })).collect(),
                ),
                Answer::Fraction { .. } => ("text", Vec::new()),
            };
            let mut v = json!({
                "number": i + 1,
                "id": q.id,
                "source": q.source,
                "level": q.dok,
                "prompt": q.prompt,
                "input": input,
                "options": options,
                "hint": hint(q),
            });
            if teacher {
                v["can_swap"] = Value::Bool(spare > 0);
                v["key"] = Value::String(key_text(q));
                v["note"] = Value::String(q.note.clone());
            }
            v
        })
        .collect();
    let swap_note = if teacher && spare == 0 {
        SetError::NoAlternative { sources: set.sources.clone(), level: set.level }.message()
    } else {
        String::new()
    };
    json!({
        "code": encode(set),
        "title": title(set),
        "notice": b.notice,
        "sources": sources,
        "questions": questions,
        "swap_note": swap_note,
        "file_name": file_name(set),
        "teacher": teacher,
    })
}
