//! The `UNTIL` value of an RRULE: a wall clock, a date, or (with a zone) a UTC instant.
//!
//! Pattern: Adapter (RRULE `UNTIL` text to a wall-clock time and back; the zone arithmetic is the
//! crate's own `time::Zone`, so the tz database is the one recorded with every import).

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use fs_types::{BasicValue, Date, LocalDateTime, Zone, ZoneKind};

use super::RruleProblem;
use crate::time::Zone as TimeZone;

/// An `UNTIL` as read, and whether the reading changed its type (a date taken as end of day).
pub(super) struct UntilRead {
    pub(super) at: LocalDateTime,
    pub(super) adjusted: bool,
}

fn digits(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_digit())
}

/// `YYYYMMDD` and optional `THHMMSS` to a wall clock; a date alone is that day at `T00:00:00`.
fn wall_clock(text: &str) -> Result<LocalDateTime, RruleProblem> {
    let bad = RruleProblem::Value("UNTIL");
    let (day, time) = match text.split_once('T') {
        Some((d, t)) if digits(t, 6) => (d, (&t[0..2], &t[2..4], &t[4..6])),
        None => (text, ("00", "00", "00")),
        Some(_) => return Err(bad),
    };
    if !digits(day, 8) {
        return Err(bad);
    }
    let date = Date::parse(&format!("{}-{}-{}", &day[0..4], &day[4..6], &day[6..8]))
        .map_err(|_| bad.clone())?;
    let n = |s: &str| s.parse::<u8>().map_err(|_| bad.clone());
    LocalDateTime::new(date, n(time.0)?, n(time.1)?, n(time.2)?).map_err(|_| bad)
}

fn time_zone(zone: &Zone) -> Result<TimeZone, RruleProblem> {
    if zone.kind() != ZoneKind::Iana {
        return Err(RruleProblem::Zone);
    }
    TimeZone::parse(zone.id()).map_err(|_| RruleProblem::Zone)
}

fn naive(at: LocalDateTime) -> Option<NaiveDateTime> {
    let d = at.date();
    NaiveDate::from_ymd_opt(d.year(), u32::from(d.month()), u32::from(d.day()))?.and_hms_opt(
        u32::from(at.hour()),
        u32::from(at.minute()),
        u32::from(at.second()),
    )
}

fn from_naive(n: NaiveDateTime) -> Result<LocalDateTime, RruleProblem> {
    let fail = || RruleProblem::Zone;
    let date = Date::parse(&format!("{:04}-{:02}-{:02}", n.year(), n.month(), n.day()))
        .map_err(|_| fail())?;
    let part = |v: u32| u8::try_from(v).map_err(|_| fail());
    LocalDateTime::new(date, part(n.hour())?, part(n.minute())?, part(n.second())?)
        .map_err(|_| fail())
}

/// Read an `UNTIL` value. A trailing `Z` needs `zone` (otherwise the rule is outside the subset)
/// and a time part (a date with `Z` is not RFC 5545). With a zone, a plain date is the whole day.
pub(super) fn until(text: &str, zone: Option<&Zone>) -> Result<UntilRead, RruleProblem> {
    match (text.strip_suffix('Z'), zone) {
        (Some(utc), _) if !utc.contains('T') => Err(RruleProblem::Value("UNTIL")),
        (Some(_), None) => Err(RruleProblem::End),
        (Some(utc), Some(zone)) => {
            let tz = time_zone(zone)?;
            let instant = naive(wall_clock(utc)?).ok_or(RruleProblem::Zone)?.and_utc();
            Ok(UntilRead {
                at: from_naive(tz.to_local(instant))?,
                adjusted: false,
            })
        }
        (None, Some(_)) if !text.contains('T') => {
            let day = wall_clock(text)?;
            let end = LocalDateTime::new(day.date(), 23, 59, 59)
                .map_err(|_| RruleProblem::Value("UNTIL"))?;
            Ok(UntilRead {
                at: end,
                adjusted: true,
            })
        }
        (None, _) => Ok(UntilRead {
            at: wall_clock(text)?,
            adjusted: false,
        }),
    }
}

/// The wall clock `at` in `zone` as an RRULE UTC value `YYYYMMDDTHHMMSSZ`.
pub(super) fn to_utc_text(at: LocalDateTime, zone: &Zone) -> Result<String, RruleProblem> {
    let tz = time_zone(zone)?;
    let utc = tz.to_utc(naive(at).ok_or(RruleProblem::Zone)?);
    Ok(utc.format("%Y%m%dT%H%M%SZ").to_string())
}
