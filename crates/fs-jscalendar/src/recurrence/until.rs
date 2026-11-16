//! The `UNTIL` value of an RRULE: a wall clock, a date, or (with a zone) a UTC instant.
//!
//! Pattern: Adapter (RRULE `UNTIL` text to a wall-clock time; the zone arithmetic is `jiff`'s).

use fs_types::{BasicValue, Date, LocalDateTime, Zone, ZoneKind};

use super::RruleProblem;

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

/// The wall clock in `zone` at the UTC instant `utc` (a local time `YYYYMMDDTHHMMSS` read as UTC).
fn in_zone(utc: LocalDateTime, zone: &Zone) -> Result<LocalDateTime, RruleProblem> {
    let fail = || RruleProblem::Zone;
    if zone.kind() != ZoneKind::Iana {
        return Err(fail());
    }
    let tz = jiff::tz::TimeZone::get(zone.id()).map_err(|_| fail())?;
    let d = utc.date();
    let civil = jiff::civil::DateTime::new(
        i16::try_from(d.year()).map_err(|_| fail())?,
        i8::try_from(d.month()).map_err(|_| fail())?,
        i8::try_from(d.day()).map_err(|_| fail())?,
        i8::try_from(utc.hour()).map_err(|_| fail())?,
        i8::try_from(utc.minute()).map_err(|_| fail())?,
        i8::try_from(utc.second()).map_err(|_| fail())?,
        0,
    )
    .map_err(|_| fail())?;
    let instant = civil
        .to_zoned(jiff::tz::TimeZone::UTC)
        .map_err(|_| fail())?
        .timestamp();
    let local = tz.to_datetime(instant);
    let date = Date::parse(&format!(
        "{:04}-{:02}-{:02}",
        local.year(),
        local.month(),
        local.day()
    ))
    .map_err(|_| fail())?;
    let part = |v: i8| u8::try_from(v).map_err(|_| fail());
    LocalDateTime::new(
        date,
        part(local.hour())?,
        part(local.minute())?,
        part(local.second())?,
    )
    .map_err(|_| fail())
}

/// Read an `UNTIL` value. A trailing `Z` needs `zone`; without one the rule is outside the subset.
pub(super) fn until(text: &str, zone: Option<&Zone>) -> Result<LocalDateTime, RruleProblem> {
    match (text.strip_suffix('Z'), zone) {
        (Some(_), None) => Err(RruleProblem::End),
        (Some(utc), Some(zone)) => in_zone(wall_clock(utc)?, zone),
        (None, _) => wall_clock(text),
    }
}
