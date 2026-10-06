//! The question bank: its shape, and the rules a bank must meet before any set
//! is made from it.

use crate::fraction;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub bank_version: String,
    pub notice: String,
    pub sources: Vec<Source>,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: String,
    pub source: String,
    pub dok: u8,
    pub prompt: String,
    pub answer: Answer,
    /// Teacher-facing working for the key. Never shown to students.
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Answer {
    Choice { options: Vec<String>, key: String },
    Fraction { key: String, form: Form },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    /// Any number equal to the key matches.
    Value,
    /// An equal number written in simplest form matches.
    Simplest,
    /// Only the same written form matches (after spacing and case are ignored).
    Exact,
}

pub const LEVELS: [u8; 4] = [1, 2, 3, 4];
pub const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// The letter for option `index` (0 gives "A").
pub fn letter(index: usize) -> String {
    LETTERS.chars().nth(index).map(String::from).unwrap_or_default()
}

impl Bank {
    pub fn source(&self, id: &str) -> Option<&Source> {
        self.sources.iter().find(|s| s.id.eq_ignore_ascii_case(id.trim()))
    }

    pub fn question(&self, id: &str) -> Option<&Question> {
        self.questions.iter().find(|q| q.id.eq_ignore_ascii_case(id.trim()))
    }
}

/// Read a bank from JSON and check every rule. On failure, every problem found
/// is listed so a teacher editing the file can fix them all at once.
pub fn load(json: &str) -> Result<Bank, Vec<String>> {
    let bank: Bank = serde_json::from_str(json)
        .map_err(|e| vec![format!("The question bank is not valid JSON for this tool: {e}")])?;
    let problems = problems(&bank);
    if problems.is_empty() {
        Ok(bank)
    } else {
        Err(problems)
    }
}

/// Every rule a bank must meet, as a list of problems (empty when it is fine).
pub fn problems(bank: &Bank) -> Vec<String> {
    let mut out = Vec::new();
    if bank.sources.is_empty() {
        out.push("The bank has no sources.".to_string());
    }
    for (i, s) in bank.sources.iter().enumerate() {
        let one_letter = s.id.len() == 1 && s.id.bytes().all(|b| b.is_ascii_uppercase());
        if !one_letter {
            out.push(format!("Source id \"{}\" must be one capital letter.", s.id));
        }
        if bank.sources[..i].iter().any(|t| t.id.eq_ignore_ascii_case(&s.id)) {
            out.push(format!("Source id \"{}\" is used twice.", s.id));
        }
        if s.title.trim().is_empty() {
            out.push(format!("Source {} has no title.", s.id));
        }
    }
    for (i, q) in bank.questions.iter().enumerate() {
        let plain_id = !q.id.is_empty() && q.id.bytes().all(|b| b.is_ascii_alphanumeric());
        if !plain_id {
            out.push(format!("Question id \"{}\" must be letters and digits only.", q.id));
        }
        if bank.questions[..i].iter().any(|t| t.id.eq_ignore_ascii_case(&q.id)) {
            out.push(format!("Question id \"{}\" is used twice.", q.id));
        }
        if bank.source(&q.source).is_none() {
            out.push(format!("Question {} names source \"{}\", which is not in the bank.", q.id, q.source));
        }
        if !LEVELS.contains(&q.dok) {
            out.push(format!("Question {} has DOK {}; DOK must be 1, 2, 3 or 4.", q.id, q.dok));
        }
        if q.prompt.trim().is_empty() {
            out.push(format!("Question {} has no prompt.", q.id));
        }
        match &q.answer {
            Answer::Choice { options, key } => {
                if options.len() < 2 || options.len() > LETTERS.len() {
                    out.push(format!("Question {} needs between 2 and 26 options.", q.id));
                }
                let key_ok = (0..options.len()).any(|i| letter(i) == key.trim().to_uppercase());
                if !key_ok {
                    out.push(format!("Question {} has key \"{}\", which is not one of its option letters.", q.id, key));
                }
            }
            Answer::Fraction { key, form } => match fraction::parse(key) {
                None => out.push(format!("Question {} has key \"{}\", which is not a number.", q.id, key)),
                Some(w) if *form == Form::Simplest && !w.simplest => out.push(format!(
                    "Question {} asks for simplest form but its key \"{}\" is not in simplest form.",
                    q.id, key
                )),
                Some(_) => {}
            },
        }
    }
    out
}
