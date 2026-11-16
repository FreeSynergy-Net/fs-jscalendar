//! Reading RRULE text into a typed rule, for the subset the owners support (calendar plan D5).
//!
//! Pattern: Adapter (RRULE text to the typed rule) with typed errors; anything outside the subset
//! is an [`RruleProblem`], and the caller keeps the original text ([`super::Recurrence::from_rrule`]).

use fs_types::Zone;

use super::until::until;
use super::valid::check_combination;
use super::{ByMonth, Frequency, NDay, RecurrenceRule, RuleEnd, Weekday};

/// A rule read for a reference point with a time, with what the reading normalised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadRule {
    /// The rule.
    pub rule: RecurrenceRule,
    /// A date `UNTIL` was read as the end of that day in the zone (normalisation / loss note).
    pub until_adjusted: bool,
}

/// Why RRULE text is not inside the subset. The text is kept by the caller, never dropped.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RruleProblem {
    /// A part that is not `NAME=value`, or a name given twice.
    #[error("malformed or repeated part")]
    Malformed,
    /// A part outside the subset (for example `RSCALE`, `BYHOUR`, `BYYEARDAY`).
    #[error("part outside the supported subset: {0}")]
    Unsupported(String),
    /// No `FREQ`, or a value of it that does not exist.
    #[error("missing or unknown FREQ")]
    Frequency,
    /// A value of a part that does not read or is out of range.
    #[error("bad value in {0}")]
    Value(&'static str),
    /// `COUNT` and `UNTIL` together, or an `UNTIL` in UTC read without a zone (use
    /// [`RecurrenceRule::parse_rrule_in`]: a UTC end is a point in time, not a wall clock).
    #[error("COUNT with UNTIL, or UNTIL in UTC without a zone")]
    End,
    /// A UTC `UNTIL` that cannot be turned into the wall clock of the zone (unknown or custom zone,
    /// or a time the zone cannot show).
    #[error("UNTIL in UTC cannot be converted into the zone")]
    Zone,
    /// Well-formed parts that RFC 5545 section 3.3.10 does not allow together (for example
    /// `BYSETPOS` alone, an ordinal `BYDAY` with `WEEKLY`, `BYMONTHDAY` with `WEEKLY`).
    #[error("combination not allowed by RFC 5545: {0}")]
    Combination(&'static str),
}

fn weekday(text: &str) -> Option<Weekday> {
    Some(match text {
        "MO" => Weekday::Mo,
        "TU" => Weekday::Tu,
        "WE" => Weekday::We,
        "TH" => Weekday::Th,
        "FR" => Weekday::Fr,
        "SA" => Weekday::Sa,
        "SU" => Weekday::Su,
        _ => return None,
    })
}

fn numbers<T: std::str::FromStr + Copy + PartialOrd>(
    text: &str,
    name: &'static str,
    nonzero: impl Fn(T) -> bool,
) -> Result<Vec<T>, RruleProblem> {
    text.split(',')
        .map(|p| {
            p.parse::<T>()
                .ok()
                .filter(|v| nonzero(*v))
                .ok_or(RruleProblem::Value(name))
        })
        .collect()
}

fn nday(text: &str) -> Result<NDay, RruleProblem> {
    let bad = RruleProblem::Value("BYDAY");
    let split = text
        .len()
        .checked_sub(2)
        .filter(|i| text.is_char_boundary(*i))
        .ok_or_else(|| bad.clone())?;
    let day = weekday(&text[split..]).ok_or_else(|| bad.clone())?;
    let nth = match &text[..split] {
        "" => None,
        n => Some(
            n.parse::<i16>()
                .ok()
                .filter(|v| *v != 0 && v.abs() <= 53)
                .ok_or(bad)?,
        ),
    };
    Ok(NDay { day, nth })
}

impl RecurrenceRule {
    /// Read RRULE text of the supported subset: `FREQ`, `INTERVAL`, `COUNT` or `UNTIL` (a wall
    /// clock or a date), `BYDAY`, `BYMONTHDAY`, `BYMONTH`, `BYSETPOS`, `WKST`.
    ///
    /// # Errors
    /// An [`RruleProblem`] for anything else; the caller keeps the text.
    pub fn parse_rrule(text: &str) -> Result<Self, RruleProblem> {
        Self::parse_with(text, None).map(|read| read.rule)
    }

    /// Like [`RecurrenceRule::parse_rrule`], for a reference point with a time in `zone`.
    ///
    /// - A UTC `UNTIL` (`...Z`, what RFC 5545 demands there) becomes the wall clock of `zone`; the
    ///   offset in force at the end instant is used. When the clock goes back (the overlap, e.g.
    ///   Berlin 2026-10-25) two instants an hour apart (00:30Z and 01:30Z) give the same wall
    ///   clock 02:30; this shows only with `HOURLY`/`MINUTELY` rules.
    /// - A date `UNTIL` is RFC-wrong here (the type must equal that of `DTSTART`) but common. It is
    ///   read tolerantly, as libical and Google do: the named day counts up to its end in `zone`
    ///   (`T23:59:59`), and [`ReadRule::until_adjusted`] says so, as a normalisation note for the
    ///   loss report. The export ([`RecurrenceRule::to_rrule_in`]) writes the RFC form.
    ///
    /// # Errors
    /// An [`RruleProblem`] (`Zone` for a zone that is unknown or custom); the caller keeps the text.
    pub fn parse_rrule_in(text: &str, zone: &Zone) -> Result<ReadRule, RruleProblem> {
        Self::parse_with(text, Some(zone))
    }

    fn parse_with(text: &str, zone: Option<&Zone>) -> Result<ReadRule, RruleProblem> {
        let mut seen = Vec::new();
        let mut rule = Self::every(Frequency::Daily, 1);
        let mut have_freq = false;
        let mut until_adjusted = false;
        for part in text.trim().trim_start_matches("RRULE:").split(';') {
            let (name, value) = part.split_once('=').ok_or(RruleProblem::Malformed)?;
            if value.is_empty() || seen.contains(&name) {
                return Err(RruleProblem::Malformed);
            }
            seen.push(name);
            match name {
                "FREQ" => {
                    rule.frequency = match value {
                        "YEARLY" => Frequency::Yearly,
                        "MONTHLY" => Frequency::Monthly,
                        "WEEKLY" => Frequency::Weekly,
                        "DAILY" => Frequency::Daily,
                        "HOURLY" => Frequency::Hourly,
                        "MINUTELY" => Frequency::Minutely,
                        "SECONDLY" => Frequency::Secondly,
                        _ => return Err(RruleProblem::Frequency),
                    };
                    have_freq = true;
                }
                "INTERVAL" => {
                    rule.interval = value
                        .parse()
                        .ok()
                        .filter(|n| *n >= 1)
                        .ok_or(RruleProblem::Value("INTERVAL"))?;
                }
                "COUNT" => {
                    rule.end = RuleEnd::Count(
                        value
                            .parse()
                            .ok()
                            .filter(|n| *n >= 1)
                            .ok_or(RruleProblem::Value("COUNT"))?,
                    );
                }
                "UNTIL" => {
                    let read = until(value, zone)?;
                    until_adjusted = read.adjusted;
                    rule.end = RuleEnd::Until(read.at);
                }
                "WKST" => {
                    rule.first_day_of_week = weekday(value).ok_or(RruleProblem::Value("WKST"))?;
                }
                "BYDAY" => rule.by_day = value.split(',').map(nday).collect::<Result<_, _>>()?,
                "BYMONTHDAY" => {
                    rule.by_month_day =
                        numbers(value, "BYMONTHDAY", |v: i16| v != 0 && v.abs() <= 31)?;
                }
                "BYSETPOS" => {
                    rule.by_set_position =
                        numbers(value, "BYSETPOS", |v: i32| v != 0 && v.abs() <= 366)?;
                }
                "BYMONTH" => {
                    rule.by_month = numbers(value, "BYMONTH", |v: u8| (1..=12).contains(&v))?
                        .into_iter()
                        .map(|month| ByMonth { month, leap: false })
                        .collect();
                }
                other => return Err(RruleProblem::Unsupported(other.to_owned())),
            }
        }
        if !have_freq {
            return Err(RruleProblem::Frequency);
        }
        if seen.contains(&"COUNT") && seen.contains(&"UNTIL") {
            return Err(RruleProblem::End);
        }
        check_combination(&rule, &seen)?;
        Ok(ReadRule {
            rule,
            until_adjusted,
        })
    }
}
