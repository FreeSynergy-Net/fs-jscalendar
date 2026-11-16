//! One recurrence of an object: a rule to compute with, or text to keep.
//!
//! Pattern: Value object. A rule outside the supported subset is kept as its original RRULE text
//! (ADR 0030, ADR 0085), never as a JSON in-between form, so an unchanged object exports the same
//! text again.

use super::RecurrenceRule;

/// One recurrence of an object: a rule to compute with, or text to keep.
// The rule is not boxed: its shape is the one both owners' stand-ins have, and a series has few of them.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recurrence {
    /// A rule inside the supported subset.
    Rule(RecurrenceRule),
    /// A rule outside it: the original RRULE text, byte for byte. Shown as "series only partly
    /// representable"; editing it is `Unsupported`; export returns the text unchanged.
    Original(String),
}

impl Recurrence {
    /// Read RRULE text: the typed rule when it is inside the subset, otherwise the text itself.
    /// Nothing is dropped, so this never fails.
    #[must_use]
    pub fn from_rrule(text: &str) -> Self {
        RecurrenceRule::parse_rrule(text)
            .map_or_else(|_| Self::Original(text.to_owned()), Self::Rule)
    }

    /// The RRULE text: the original as it is, or the written form of the rule.
    #[must_use]
    pub fn to_rrule(&self) -> String {
        match self {
            Self::Rule(rule) => rule.to_rrule(),
            Self::Original(text) => text.clone(),
        }
    }

    /// The rule, when it can be computed with.
    #[must_use]
    pub fn rule(&self) -> Option<&RecurrenceRule> {
        match self {
            Self::Rule(rule) => Some(rule),
            Self::Original(_) => None,
        }
    }
}

impl From<RecurrenceRule> for Recurrence {
    fn from(rule: RecurrenceRule) -> Self {
        Self::Rule(rule)
    }
}

impl TryFrom<Recurrence> for RecurrenceRule {
    /// The original text, so nothing is lost on the way.
    type Error = String;

    fn try_from(value: Recurrence) -> Result<Self, Self::Error> {
        match value {
            Recurrence::Rule(rule) => Ok(rule),
            Recurrence::Original(text) => Err(text),
        }
    }
}
