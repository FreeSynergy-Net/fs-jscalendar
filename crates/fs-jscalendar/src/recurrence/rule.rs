//! The typed recurrence rule (RFC 8984 `RecurrenceRule`) that `fs-calendars` and `fs-tasks` share.
//!
//! Pattern: Value object — plain data with public fields; the rule *engine* is a strategy of the
//! owners. The shape is field for field the one both topic traits already use (ADR 0085), so
//! their stand-ins convert into it and back without loss. Not to be confused with
//! [`crate::model::RecurrenceRule`], the string-typed wire form of one JSCalendar object.

use fs_types::LocalDateTime;

/// Day of the week.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Weekday {
    Mo,
    Tu,
    We,
    Th,
    Fr,
    Sa,
    Su,
}

/// How often a rule repeats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Frequency {
    Yearly,
    Monthly,
    Weekly,
    Daily,
    Hourly,
    Minutely,
    Secondly,
}

/// What a calendar with leap months does with a date that does not exist (RFC 7529 `SKIP`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Skip {
    Omit,
    Backward,
    Forward,
}

/// A weekday with an optional place in the period (`-1` = the last).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NDay {
    /// The weekday.
    pub day: Weekday,
    /// Its place in the period, when given.
    pub nth: Option<i16>,
}

/// A month with the leap-month mark (RFC 7529 `L`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ByMonth {
    /// The month number, 1-based.
    pub month: u8,
    /// A leap month (`L`).
    pub leap: bool,
}

/// How a rule ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuleEnd {
    /// Never.
    Never,
    /// After this many occurrences.
    Count(u32),
    /// At the latest at this wall-clock time (a task with a plain date: that day at `T00:00:00`).
    Until(LocalDateTime),
}

/// A recurrence rule the owners can compute with (the subset they support, calendar plan D5).
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct RecurrenceRule {
    pub frequency: Frequency,
    pub interval: u32,
    pub rscale: Option<String>,
    pub skip: Skip,
    pub first_day_of_week: Weekday,
    pub by_day: Vec<NDay>,
    pub by_month_day: Vec<i16>,
    pub by_month: Vec<ByMonth>,
    pub by_year_day: Vec<i16>,
    pub by_week_no: Vec<i16>,
    pub by_hour: Vec<u8>,
    pub by_minute: Vec<u8>,
    pub by_second: Vec<u8>,
    pub by_set_position: Vec<i32>,
    pub end: RuleEnd,
}

impl RecurrenceRule {
    /// A plain rule: every `interval`-th period, weeks start on Monday, no filters, never ends.
    #[must_use]
    pub fn every(frequency: Frequency, interval: u32) -> Self {
        Self {
            frequency,
            interval,
            rscale: None,
            skip: Skip::Omit,
            first_day_of_week: Weekday::Mo,
            by_day: Vec::new(),
            by_month_day: Vec::new(),
            by_month: Vec::new(),
            by_year_day: Vec::new(),
            by_week_no: Vec::new(),
            by_hour: Vec::new(),
            by_minute: Vec::new(),
            by_second: Vec::new(),
            by_set_position: Vec::new(),
            end: RuleEnd::Never,
        }
    }
}
