//! Golden test: recurrence and zones through `calcard-next` (iCalendar <-> JSCalendar).
//!
//! The expected values are derived by hand from the RFCs, not copied from the output:
//! * 2026-03-29 is the DST change in Europe/Berlin (RFC 5545 TZID, tzdb): CET (UTC+1) before, CEST
//!   (UTC+2) after 02:00 local. A daily event at 09:00 local stays at 09:00 local (wall clock).
//! * `UNTIL=20260403T070000Z`: RFC 5545 section 3.3.10 demands UTC when DTSTART has a TZID; on
//!   2026-04-03 Berlin is CEST, so 07:00Z is 09:00 local. RFC 8984 section 4.3.3 states `until` as
//!   LocalDateTime in the event's zone: "2026-04-03T09:00:00".
//! * EXDATE becomes `recurrenceOverrides` with `excluded: true` (RFC 8984 section 4.3.2), keyed by
//!   the LocalDateTime of the occurrence; RECURRENCE-ID becomes an override keyed the same way
//!   carrying the changed properties (here `start` and `title`).
//! * Duration 10:00 - 09:00 is "PT1H" (RFC 8984 section 4.2.3 / 3.3.1).

use fs_jscalendar::ical::{parse_entries, write_ics};
use serde_json::json;

const ICS: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//EN\r\n\
BEGIN:VEVENT\r\nUID:g1\r\nDTSTAMP:20260101T000000Z\r\n\
DTSTART;TZID=Europe/Berlin:20260328T090000\r\nDTEND;TZID=Europe/Berlin:20260328T100000\r\n\
RRULE:FREQ=DAILY;UNTIL=20260403T070000Z\r\nEXDATE;TZID=Europe/Berlin:20260330T090000\r\n\
SUMMARY:Daily\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:g1\r\nDTSTAMP:20260101T000000Z\r\n\
RECURRENCE-ID;TZID=Europe/Berlin:20260331T090000\r\n\
DTSTART;TZID=Europe/Berlin:20260331T110000\r\nDTEND;TZID=Europe/Berlin:20260331T120000\r\n\
SUMMARY:Moved\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn the_event() -> serde_json::Value {
    let (events, _) = parse_entries(ICS, "Event").unwrap();
    assert_eq!(
        events.len(),
        1,
        "series and override are one event: {events:#?}"
    );
    events.into_iter().next().unwrap()
}

#[test]
fn berlin_series_across_dst_with_until_exdate_and_override() {
    let e = the_event();
    assert_eq!(e["uid"], "g1");
    assert_eq!(e["start"], "2026-03-28T09:00:00");
    assert_eq!(e["timeZone"], "Europe/Berlin");
    assert_eq!(e["duration"], "PT1H");
    // The rule itself: see the two tests below (key name deviates from RFC 8984).
    let o = &e["recurrenceOverrides"];
    assert_eq!(o["2026-03-30T09:00:00"], json!({"excluded": true}));
    assert_eq!(o["2026-03-31T09:00:00"]["start"], "2026-03-31T11:00:00");
    assert_eq!(o["2026-03-31T09:00:00"]["title"], "Moved");
}

/// REPORTED DEVIATION: RFC 8984 section 4.3.2 names the property `recurrenceRules` (array of
/// `RecurrenceRule`); calcard-next 3.0.1 emits the singular `recurrenceRule` object (JSCalendar 1.0 name).
/// The expectation is the RFC one and is NOT adapted; the test is ignored until upstream or our
/// adapter delivers `recurrenceRules`.
#[test]
#[ignore = "calcard-next 3.0.1 emits recurrenceRule (1.0), RFC 8984 says recurrenceRules"]
fn rule_is_the_rfc_8984_array() {
    let e = the_event();
    let rule = &e["recurrenceRules"][0];
    assert_eq!(rule["frequency"], "daily");
    assert_eq!(rule["until"], "2026-04-03T09:00:00");
}

/// What calcard-next delivers today under its own key; the VALUE is the hand-derived one.
#[test]
fn rule_value_under_the_current_key_is_right() {
    let e = the_event();
    let rule = &e["recurrenceRule"];
    assert_eq!(rule["frequency"], "daily");
    assert_eq!(rule["until"], "2026-04-03T09:00:00");
}

#[test]
fn berlin_series_written_back_keeps_zone_until_in_utc_exdate_and_recurrence_id() {
    let text = write_ics([the_event()], "-//t//EN").unwrap();
    let unfolded = text.replace("\r\n ", "");
    for want in [
        "DTSTART;TZID=Europe/Berlin:20260328T090000",
        "UNTIL=20260403T070000Z",
        "EXDATE;TZID=Europe/Berlin:20260330T090000",
        "RECURRENCE-ID;TZID=Europe/Berlin:20260331T090000",
        "SUMMARY:Moved",
    ] {
        assert!(unfolded.contains(want), "missing {want} in:\n{text}");
    }
}
