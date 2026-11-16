//! A reminder (JSCalendar `alerts`), shared by calendar and tasks.
//!
//! Pattern: Value object — plain data. The shape is field for field the one both topic traits
//! use, so their stand-ins convert without loss.

use fs_types::Utc;

/// What an offset trigger counts from. RFC 8984 knows only `start` and `end`; for a task `End`
/// means its due time (a task has no end), so tasks use `End` where they say "due".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelativeTo {
    /// The start.
    Start,
    /// The end (a task: the due time).
    End,
}

/// What a reminder does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertAction {
    /// Show it.
    Display,
    /// Send an e-mail.
    Email,
}

/// When a reminder fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// `seconds` from the start or end (negative: before).
    Offset {
        /// Signed distance in seconds.
        seconds: i64,
        /// What it counts from.
        relative_to: RelativeTo,
    },
    /// At a fixed instant.
    Absolute {
        /// The instant.
        at: Utc,
    },
}

/// A reminder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alert {
    /// Key in the object (JSCalendar `Id`).
    pub key: String,
    /// When it fires.
    pub trigger: Trigger,
    /// What it does.
    pub action: AlertAction,
    /// When it was acknowledged.
    pub acknowledged: Option<Utc>,
}
