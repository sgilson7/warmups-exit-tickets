//! Checking a student's answers against the key, and the score.

use crate::bank::{Answer, Bank, Form};
use crate::fraction::{normalize, parse};
use crate::select::ProblemSet;

/// Whether one written answer matches the key. A blank answer never matches.
pub fn answer_matches(answer: &Answer, given: &str) -> bool {
    if given.trim().is_empty() {
        return false;
    }
    match answer {
        Answer::Choice { key, .. } => normalize(given) == normalize(key),
        Answer::Fraction { key, form: Form::Exact } => normalize(given) == normalize(key),
        Answer::Fraction { key, form } => match (parse(given), parse(key)) {
            (Some(g), Some(k)) => g.value == k.value && (*form == Form::Value || g.simplest),
            _ => false,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    pub number: usize,
    pub id: String,
    pub answered: bool,
    pub matches: bool,
}

impl Mark {
    pub fn label(&self) -> &'static str {
        match (self.answered, self.matches) {
            (false, _) => "No answer",
            (true, true) => "Matches the key",
            (true, false) => "Does not match the key",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    pub marks: Vec<Mark>,
    pub matched: usize,
    pub total: usize,
}

impl Score {
    pub fn summary(&self) -> String {
        let noun = if self.total == 1 { "answer matches" } else { "answers match" };
        format!("{} of {} {} the key.", self.matched, self.total, noun)
    }
}

/// Mark every answer in the set, in order. One answer per question.
pub fn check(bank: &Bank, set: &ProblemSet, answers: &[String]) -> Result<Score, String> {
    if answers.len() != set.questions.len() {
        let given = if answers.len() == 1 { "answer was" } else { "answers were" };
        return Err(format!(
            "This set has {} questions, but {} {given} given.",
            set.questions.len(),
            answers.len()
        ));
    }
    let mut marks = Vec::new();
    for (i, (id, given)) in set.questions.iter().zip(answers).enumerate() {
        let q = bank.question(id).ok_or_else(|| format!("Question {id} is not in the question bank."))?;
        marks.push(Mark {
            number: i + 1,
            id: q.id.clone(),
            answered: !given.trim().is_empty(),
            matches: answer_matches(&q.answer, given),
        });
    }
    let matched = marks.iter().filter(|m| m.matches).count();
    Ok(Score { total: marks.len(), matched, marks })
}
