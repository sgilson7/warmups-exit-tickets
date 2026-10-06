//! Set codes such as `W3-AB-A3A-B3A-A3B`: kind letter and DOK level, chosen
//! sources, then the question ids. A student types the code (or opens the
//! saved file that carries it) to get the same set on their own device.

use crate::bank::{Bank, LEVELS};
use crate::select::{normalize_sources, Kind, ProblemSet};

const NOT_A_CODE: &str = "That is not a set code. A set code looks like W3-AB-A3A-B3A-A3B.";

pub fn encode(set: &ProblemSet) -> String {
    let ids: Vec<String> = set.questions.iter().map(|q| q.to_uppercase()).collect();
    format!("{}{}-{}-{}", set.kind.letter(), set.level, set.sources.concat(), ids.join("-"))
}

/// Read a code typed in any case, with or without stray spaces.
pub fn decode(bank: &Bank, code: &str) -> Result<ProblemSet, String> {
    let compact: String = code.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_uppercase();
    let parts: Vec<&str> = compact.split('-').collect();
    if parts.len() < 3 || parts.iter().any(|p| p.is_empty()) {
        return Err(NOT_A_CODE.to_string());
    }
    let mut head = parts[0].chars();
    let kind = head.next().and_then(Kind::from_letter).ok_or_else(|| NOT_A_CODE.to_string())?;
    let level: u8 = head.as_str().parse().map_err(|_| NOT_A_CODE.to_string())?;
    if !LEVELS.contains(&level) {
        return Err(NOT_A_CODE.to_string());
    }
    let chosen: Vec<String> = parts[1].chars().map(String::from).collect();
    let sources = normalize_sources(bank, &chosen).map_err(|e| e.message())?;
    let ids = &parts[2..];
    if ids.len() != kind.size() {
        return Err(format!(
            "{} code needs {} questions, but this code has {}.",
            capitalized(kind.with_article()),
            kind.size(),
            ids.len()
        ));
    }
    let mut questions: Vec<String> = Vec::new();
    for id in ids {
        let q = bank
            .question(id)
            .ok_or_else(|| format!("The set code names question {id}, which is not in the question bank."))?;
        let fits = q.dok == level && sources.iter().any(|s| s.eq_ignore_ascii_case(&q.source));
        if !fits {
            return Err(format!("Question {id} does not match the sources and DOK level in this set code."));
        }
        if questions.iter().any(|seen| seen.eq_ignore_ascii_case(&q.id)) {
            return Err(format!("The set code names question {id} twice."));
        }
        questions.push(q.id.clone());
    }
    Ok(ProblemSet { kind, level, sources, questions })
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The code inside a saved set file (its `Set code:` line), or the whole text
/// when there is no such line (the student typed the code itself).
pub fn find_code(text: &str) -> &str {
    for line in text.lines() {
        let t = line.trim();
        if let Some(label) = t.get(..9) {
            if label.eq_ignore_ascii_case("set code:") {
                return t[9..].trim();
            }
        }
    }
    text.trim()
}
