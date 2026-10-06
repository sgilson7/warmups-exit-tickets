//! The words a set is shown and saved with: title, hints, key text, and the
//! plain-text file a teacher saves and a student can open.

use crate::bank::{letter, Answer, Bank, Form, Question};
use crate::code::encode;
use crate::select::{level_label, source_phrase, ProblemSet};

pub fn title(set: &ProblemSet) -> String {
    format!(
        "{}: DOK {} ({}), {}",
        set.kind.label(),
        set.level,
        level_label(set.level),
        source_phrase(&set.sources)
    )
}

/// What the student is told about how to write the answer.
pub fn hint(q: &Question) -> &'static str {
    match &q.answer {
        Answer::Choice { .. } => "Choose one letter.",
        Answer::Fraction { form: Form::Value, .. } => {
            "Write a number: a fraction, a mixed number, a whole number or a decimal."
        }
        Answer::Fraction { form: Form::Simplest, .. } => "Write your answer in simplest form.",
        Answer::Fraction { form: Form::Exact, .. } => "Write it in the form the question asks for.",
    }
}

/// The key as the teacher sees it.
pub fn key_text(q: &Question) -> String {
    match &q.answer {
        Answer::Choice { options, key } => {
            let k = key.trim().to_uppercase();
            let text = options.iter().enumerate().find(|(i, _)| letter(*i) == k).map(|(_, t)| t.as_str());
            format!("{}) {}", k, text.unwrap_or(""))
        }
        Answer::Fraction { key, form } => {
            let rule = match form {
                Form::Value => "any equal number matches",
                Form::Simplest => "an equal number in simplest form matches",
                Form::Exact => "only this written form matches",
            };
            format!("{key} ({rule})")
        }
    }
}

pub fn file_name(set: &ProblemSet) -> String {
    format!("{}-{}.txt", set.kind.file_slug(), encode(set))
}

/// The student copy as plain text: no key, and a `Set code:` line that the
/// student page can read back.
pub fn file_text(bank: &Bank, set: &ProblemSet) -> String {
    let mut out = String::new();
    out.push_str(&title(set));
    out.push_str("\n\n");
    out.push_str(&bank.notice);
    out.push_str("\n\n");
    for id in &set.sources {
        if let Some(s) = bank.source(id) {
            out.push_str(&format!("Source {}: {}\n{}\n\n", s.id, s.title, s.text));
        }
    }
    for (i, id) in set.questions.iter().enumerate() {
        let Some(q) = bank.question(id) else { continue };
        out.push_str(&format!("{}. {}\n", i + 1, q.prompt));
        if let Answer::Choice { options, .. } = &q.answer {
            for (j, o) in options.iter().enumerate() {
                out.push_str(&format!("   {}) {}\n", letter(j), o));
            }
        }
        out.push_str(&format!("   {}\n   Answer: ____________________\n\n", hint(q)));
    }
    out.push_str(&format!("Set code: {}\n", encode(set)));
    out.push_str("To answer on a device: open the tool, choose Student, and type the set code or open this file.\n");
    out
}
