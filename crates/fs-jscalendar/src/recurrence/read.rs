//! Reading RRULE text into a typed rule, for the subset the owners support (calendar plan D5).
//!
//! Pattern: Adapter (RRULE text to the typed rule) with typed errors; anything outside the subset
//! is an [`RruleProblem`], and the caller keeps the original text ([`super::Recurrence::from_rrule`]).

use fs_types::{BasicValue, Date, LocalDateTime};

use super::{ByMonth, Frequency, NDay, RecurrenceRule, RuleEnd, Weekday};

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
    /// `COUNT` and `UNTIL` together, or an `UNTIL` in UTC (it is a point in time, not a wall clock).
    #[error("COUNT with UNTIL, or UNTIL in UTC")]
    End,
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

fn until(text: &str) -> Result<LocalDateTime, RruleProblem> {
    let bad = RruleProblem::Value("UNTIL");
    if text.ends_with('Z') {
        return Err(RruleProblem::End);
    }
    let digits = |s: &str| s.len() == 8 && s.bytes().all(|b| b.is_ascii_digit());
    let (day, time) = match text.split_once('T') {
        Some((d, t)) if t.len() == 6 && t.bytes().all(|b| b.is_ascii_digit()) => {
            (d, (&t[0..2], &t[2..4], &t[4..6]))
        }
        None => (text, ("00", "00", "00")),
        Some(_) => return Err(bad),
    };
    if !digits(day) {
        return Err(bad);
    }
    let date = Date::parse(&format!("{}-{}-{}", &day[0..4], &day[4..6], &day[6..8]))
        .map_err(|_| bad.clone())?;
    let n = |s: &str| s.parse::<u8>().map_err(|_| bad.clone());
    LocalDateTime::new(date, n(time.0)?, n(time.1)?, n(time.2)?).map_err(|_| bad)
}

impl RecurrenceRule {
    /// Read RRULE text of the supported subset: `FREQ`, `INTERVAL`, `COUNT` or `UNTIL` (a wall
    /// clock or a date), `BYDAY`, `BYMONTHDAY`, `BYMONTH`, `BYSETPOS`, `WKST`.
    ///
    /// # Errors
    /// An [`RruleProblem`] for anything else; the caller keeps the text.
    pub fn parse_rrule(text: &str) -> Result<Self, RruleProblem> {
        let mut seen = Vec::new();
        let mut rule = Self::every(Frequency::Daily, 1);
        let mut have_freq = false;
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
                "UNTIL" => rule.end = RuleEnd::Until(until(value)?),
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
        Ok(rule)
    }
}
