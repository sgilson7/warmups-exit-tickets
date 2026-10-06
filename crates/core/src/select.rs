//! Making a warm-up or exit ticket from the bank, and swapping one question.

use crate::bank::{Bank, Question, LEVELS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    WarmUp,
    ExitTicket,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::WarmUp, Kind::ExitTicket];

    /// How many questions a set of this kind holds.
    pub fn size(self) -> usize {
        match self {
            Kind::WarmUp => 3,
            Kind::ExitTicket => 2,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Kind::WarmUp => "warmup",
            Kind::ExitTicket => "exit",
        }
    }

    pub fn letter(self) -> char {
        match self {
            Kind::WarmUp => 'W',
            Kind::ExitTicket => 'E',
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::WarmUp => "Warm-up",
            Kind::ExitTicket => "Exit ticket",
        }
    }

    pub fn with_article(self) -> &'static str {
        match self {
            Kind::WarmUp => "a warm-up",
            Kind::ExitTicket => "an exit ticket",
        }
    }

    pub fn file_slug(self) -> &'static str {
        match self {
            Kind::WarmUp => "warm-up",
            Kind::ExitTicket => "exit-ticket",
        }
    }

    pub fn from_id(id: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.id() == id.trim())
    }

    pub fn from_letter(c: char) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.letter() == c.to_ascii_uppercase())
    }
}

/// The plain-language name of a DOK level.
pub fn level_label(level: u8) -> &'static str {
    match level {
        1 => "Recall",
        2 => "Skill or concept",
        3 => "Strategic thinking",
        4 => "Extended thinking",
        _ => "",
    }
}

/// A set: its kind, level, chosen sources (bank order) and question ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemSet {
    pub kind: Kind,
    pub level: u8,
    pub sources: Vec<String>,
    pub questions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetError {
    NoKind,
    NoSource,
    UnknownSource(String),
    BadLevel,
    Shortfall {
        kind: Kind,
        sources: Vec<String>,
        level: u8,
        matching: usize,
    },
    BadNumber {
        number: usize,
    },
    NoAlternative {
        sources: Vec<String>,
        level: u8,
    },
}

/// "source B", "sources A and B", "sources A, B and C".
pub fn source_phrase(sources: &[String]) -> String {
    match sources {
        [] => "no source".to_string(),
        [one] => format!("source {one}"),
        [rest @ .., last] => format!("sources {} and {}", rest.join(", "), last),
    }
}

impl SetError {
    pub fn message(&self) -> String {
        match self {
            SetError::NoKind => "Choose a warm-up or an exit ticket.".to_string(),
            SetError::NoSource => "Choose at least one source document.".to_string(),
            SetError::UnknownSource(s) => format!("There is no source \"{s}\" in the question bank."),
            SetError::BadLevel => "Choose a DOK level from 1 to 4.".to_string(),
            SetError::Shortfall { kind, sources, level, matching } => {
                let found = match matching {
                    0 => "No question in the bank matches".to_string(),
                    1 => "Only 1 question in the bank matches".to_string(),
                    n => format!("Only {n} questions in the bank match"),
                };
                let advice = if *kind == Kind::WarmUp && *matching >= Kind::ExitTicket.size() {
                    "Choose another source as well, or make an exit ticket."
                } else {
                    "Choose another source as well."
                };
                format!(
                    "{found} {} at DOK {level}, and {} needs {}. {advice}",
                    source_phrase(sources),
                    kind.with_article(),
                    kind.size()
                )
            }
            SetError::BadNumber { number } => format!("There is no question {number} in this set."),
            SetError::NoAlternative { sources, level } => format!(
                "No other question in the bank matches {} at DOK {level}: every matching question is already in this set.",
                source_phrase(sources)
            ),
        }
    }
}

/// Chosen source ids, upper-cased, without repeats, in bank order.
pub fn normalize_sources(bank: &Bank, chosen: &[String]) -> Result<Vec<String>, SetError> {
    let chosen: Vec<&str> = chosen.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if chosen.is_empty() {
        return Err(SetError::NoSource);
    }
    for c in &chosen {
        if bank.source(c).is_none() {
            return Err(SetError::UnknownSource(c.to_string()));
        }
    }
    Ok(bank
        .sources
        .iter()
        .filter(|s| chosen.iter().any(|c| s.id.eq_ignore_ascii_case(c)))
        .map(|s| s.id.clone())
        .collect())
}

pub fn check_level(level: u8) -> Result<u8, SetError> {
    if LEVELS.contains(&level) {
        Ok(level)
    } else {
        Err(SetError::BadLevel)
    }
}

/// Every question that may appear in a set for these sources and level, in
/// candidate order: round-robin across the sources (bank order), questions
/// within one source ordered by id.
pub fn candidates<'a>(bank: &'a Bank, sources: &[String], level: u8) -> Vec<&'a Question> {
    let per_source: Vec<Vec<&Question>> = sources
        .iter()
        .map(|s| {
            let mut list: Vec<&Question> = bank
                .questions
                .iter()
                .filter(|q| q.source.eq_ignore_ascii_case(s) && q.dok == level)
                .collect();
            list.sort_by_key(|q| q.id.to_uppercase());
            list
        })
        .collect();
    let rounds = per_source.iter().map(|l| l.len()).max().unwrap_or(0);
    let mut out = Vec::new();
    for round in 0..rounds {
        for list in &per_source {
            if let Some(q) = list.get(round) {
                out.push(*q);
            }
        }
    }
    out
}

/// The set for this kind, sources and level: the first candidates in
/// candidate order. Refused when the bank holds too few matching questions.
pub fn make_set(bank: &Bank, kind: Kind, chosen: &[String], level: u8) -> Result<ProblemSet, SetError> {
    let sources = normalize_sources(bank, chosen)?;
    let level = check_level(level)?;
    let found = candidates(bank, &sources, level);
    if found.len() < kind.size() {
        return Err(SetError::Shortfall { kind, sources, level, matching: found.len() });
    }
    let questions = found.iter().take(kind.size()).map(|q| q.id.clone()).collect();
    Ok(ProblemSet { kind, level, sources, questions })
}

/// How many matching questions are not in the set (0 means nothing to swap).
pub fn alternatives(bank: &Bank, set: &ProblemSet) -> usize {
    candidates(bank, &set.sources, set.level)
        .iter()
        .filter(|q| !set.questions.iter().any(|id| id.eq_ignore_ascii_case(&q.id)))
        .count()
}

/// Replace question `number` (1 = first) with the next candidate after it, in
/// candidate order and wrapping round, that is not already in the set.
pub fn swap(bank: &Bank, set: &ProblemSet, number: usize) -> Result<ProblemSet, SetError> {
    if number == 0 || number > set.questions.len() {
        return Err(SetError::BadNumber { number });
    }
    let found = candidates(bank, &set.sources, set.level);
    let current = &set.questions[number - 1];
    let start = found.iter().position(|q| q.id.eq_ignore_ascii_case(current)).unwrap_or(0);
    for step in 1..=found.len() {
        let q = found[(start + step) % found.len()];
        let used = set.questions.iter().any(|id| id.eq_ignore_ascii_case(&q.id));
        if !used {
            let mut next = set.clone();
            next.questions[number - 1] = q.id.clone();
            return Ok(next);
        }
    }
    Err(SetError::NoAlternative { sources: set.sources.clone(), level: set.level })
}
