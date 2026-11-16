//! One recurrence of an object: a rule to compute with, or text to keep.
//!
//! Pattern: Value object. A rule outside the supported subset is kept as its original RRULE text
//! (ADR 0030, ADR 0085), never as a JSON in-between form, so an unchanged object exports the same
//! text again.

use super::{RecurrenceRule, RruleProblem};

/// A recurrence read for a reference point with a time, with what the reading normalised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadRecurrence {
    /// The recurrence.
    pub recurrence: Recurrence,
    /// A date `UNTIL` was read as the end of that day in the zone (normalisation / loss note).
    pub until_adjusted: bool,
}

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

    /// Like [`Recurrence::from_rrule`], for a reference point with a time in `zone`: a UTC
    /// `UNTIL` becomes that zone's wall clock, a date `UNTIL` the end of that day (see
    /// [`RecurrenceRule::parse_rrule_in`]; `until_adjusted` carries the note).
    #[must_use]
    pub fn from_rrule_in(text: &str, zone: &fs_types::Zone) -> ReadRecurrence {
        match RecurrenceRule::parse_rrule_in(text, zone) {
            Ok(read) => ReadRecurrence {
                recurrence: Self::Rule(read.rule),
                until_adjusted: read.until_adjusted,
            },
            Err(_) => ReadRecurrence {
                recurrence: Self::Original(text.to_owned()),
                until_adjusted: false,
            },
        }
    }

    /// The RRULE text for a reference point with a time in `zone` (UTC `UNTIL`, see
    /// [`RecurrenceRule::to_rrule_in`]); an original is returned as it is.
    ///
    /// # Errors
    /// `RruleProblem::Zone` for a zone that is unknown or custom, when a rule has an `UNTIL`.
    pub fn to_rrule_in(&self, zone: &fs_types::Zone) -> Result<String, RruleProblem> {
        match self {
            Self::Rule(rule) => rule.to_rrule_in(zone),
            Self::Original(text) => Ok(text.clone()),
        }
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
