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
    /// Nothing is dropped, so this never fails. A UTC `UNTIL` is outside the subset here; use
    /// [`Recurrence::from_rrule_in`] when the zone of the reference point is known.
    #[must_use]
    pub fn from_rrule(text: &str) -> Self {
        RecurrenceRule::parse_rrule(text)
            .map_or_else(|_| Self::Original(text.to_owned()), Self::Rule)
    }

    /// Like [`Recurrence::from_rrule`], with the zone of the reference point so a UTC `UNTIL` is
    /// turned into that zone's wall clock.
    #[must_use]
    pub fn from_rrule_in(text: &str, zone: &fs_types::Zone) -> Self {
        RecurrenceRule::parse_rrule_in(text, zone)
            .map_or_else(|_| Self::Original(text.to_owned()), Self::Rule)
    }

    /// The RRULE text: the original as it is, or the written form of the rule.
    #[must_use]
    pub fn to_rrule(&self) -> String {
        self.to_rrule_for(false)
    }

    /// Like [`Recurrence::to_rrule`]; `date_only` when the reference point is a plain date, so
    /// `UNTIL` is written as a date too (RFC 5545: same value type as `DTSTART`).
    #[must_use]
    pub fn to_rrule_for(&self, date_only: bool) -> String {
        match self {
            Self::Rule(rule) => rule.to_rrule_for(date_only),
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

/// The original RRULE text of a recurrence that is no typed rule: the error of
/// `RecurrenceRule::try_from(Recurrence)`, so nothing is lost on the way.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("recurrence kept as original text: {0}")]
pub struct OriginalText(pub String);

impl TryFrom<Recurrence> for RecurrenceRule {
    type Error = OriginalText;

    fn try_from(value: Recurrence) -> Result<Self, Self::Error> {
        match value {
            Recurrence::Rule(rule) => Ok(rule),
            Recurrence::Original(text) => Err(OriginalText(text)),
        }
    }
}
