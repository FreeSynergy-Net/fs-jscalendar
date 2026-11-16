//! Typed recurrence with a reference point in a time zone: UTC and date `UNTIL`, and the export.
//!
//! Pattern: Specification by example — Europe/Berlin, summer, winter and the change days.

use fs_jscalendar::recurrence::{Recurrence, RecurrenceRule, RruleProblem, RuleEnd};

fn rule(text: &str) -> RecurrenceRule {
    RecurrenceRule::parse_rrule(text).expect("inside the subset")
}

fn berlin() -> fs_types::Zone {
    fs_types::Zone::iana("Europe/Berlin").expect("zone")
}

fn until_of(text: &str) -> String {
    let RuleEnd::Until(t) = RecurrenceRule::parse_rrule_in(text, &berlin())
        .expect("rule")
        .rule
        .end
    else {
        panic!("until")
    };
    format!(
        "{}T{:02}:{:02}:{:02}",
        fs_types::BasicValue::canonical(&t.date()),
        t.hour(),
        t.minute(),
        t.second()
    )
}

#[test]
fn utc_until_becomes_the_wall_clock_of_the_zone() {
    // Summer (CEST, +2) and winter (CET, +1).
    assert_eq!(
        until_of("FREQ=DAILY;UNTIL=20260715T073000Z"),
        "2026-07-15T09:30:00"
    );
    assert_eq!(
        until_of("FREQ=DAILY;UNTIL=20261215T073000Z"),
        "2026-12-15T08:30:00"
    );
}

#[test]
fn utc_until_at_the_daylight_saving_edge() {
    // Berlin 2026-03-29: 02:00 CET jumps to 03:00 CEST (01:00 UTC).
    assert_eq!(
        until_of("FREQ=DAILY;UNTIL=20260328T230000Z"),
        "2026-03-29T00:00:00"
    );
    assert_eq!(
        until_of("FREQ=DAILY;UNTIL=20260329T005959Z"),
        "2026-03-29T01:59:59"
    );
    assert_eq!(
        until_of("FREQ=DAILY;UNTIL=20260329T010000Z"),
        "2026-03-29T03:00:00"
    );
    // Back on 2026-10-25: 03:00 CEST (01:00 UTC) becomes 02:00 CET.
    assert_eq!(
        until_of("FREQ=DAILY;UNTIL=20261025T005959Z"),
        "2026-10-25T02:59:59"
    );
    assert_eq!(
        until_of("FREQ=DAILY;UNTIL=20261025T010000Z"),
        "2026-10-25T02:00:00"
    );
}

#[test]
fn utc_until_with_an_unusable_zone_keeps_the_text() {
    let text = "FREQ=DAILY;UNTIL=20260715T073000Z";
    let custom = fs_types::Zone::custom("/mine").expect("zone");
    assert_eq!(
        RecurrenceRule::parse_rrule_in(text, &custom),
        Err(RruleProblem::Zone)
    );
    assert_eq!(
        Recurrence::from_rrule_in(text, &custom).recurrence,
        Recurrence::Original(text.to_owned())
    );
    assert!(
        Recurrence::from_rrule_in(text, &berlin())
            .recurrence
            .rule()
            .is_some()
    );
    assert_eq!(RecurrenceRule::parse_rrule(text), Err(RruleProblem::End));
}

#[test]
fn until_is_written_as_a_date_when_the_reference_point_is_a_date() {
    let r = rule("FREQ=DAILY;UNTIL=20261231");
    assert_eq!(r.to_rrule(), "FREQ=DAILY;UNTIL=20261231T000000");
    assert_eq!(r.to_rrule_for(true), "FREQ=DAILY;UNTIL=20261231");
    assert_eq!(rule(&r.to_rrule_for(true)), r);
    assert_eq!(
        Recurrence::from(r).to_rrule_for(true),
        "FREQ=DAILY;UNTIL=20261231"
    );
    assert_eq!(Recurrence::Original("X".into()).to_rrule_for(true), "X");
}

#[test]
fn the_export_writes_the_utc_form_and_reads_back_the_same_wall_clock() {
    // Summer, winter and the change day (Berlin 2026-03-29, 02:30 does not exist: "compatible" = before the gap's offset).
    for (wall, utc) in [
        ("20260715T093000", "20260715T073000Z"),
        ("20261215T083000", "20261215T073000Z"),
        ("20260329T000000", "20260328T230000Z"),
        ("20260329T030000", "20260329T010000Z"),
        ("20261025T020000", "20261025T000000Z"),
    ] {
        let r = rule(&format!("FREQ=DAILY;UNTIL={wall}"));
        assert_eq!(
            r.to_rrule_in(&berlin()).expect("text"),
            format!("FREQ=DAILY;UNTIL={utc}"),
            "{wall}"
        );
    }
    let r = rule("FREQ=DAILY;COUNT=3");
    assert_eq!(
        r.to_rrule_in(&berlin()).expect("text"),
        "FREQ=DAILY;COUNT=3"
    );
    let custom = fs_types::Zone::custom("/mine").expect("zone");
    assert_eq!(
        rule("FREQ=DAILY;UNTIL=20260101T000000").to_rrule_in(&custom),
        Err(RruleProblem::Zone)
    );
    // Rundlauf: schreiben (UTC) und in derselben Zone lesen ergibt dieselbe Regel.
    let r = rule("FREQ=WEEKLY;UNTIL=20260715T093000");
    let back = RecurrenceRule::parse_rrule_in(&r.to_rrule_in(&berlin()).expect("text"), &berlin())
        .expect("rule");
    assert_eq!(back.rule, r);
    assert!(!back.until_adjusted);
}

#[test]
fn the_overlap_maps_two_instants_to_one_wall_clock() {
    // 2026-10-25 in Berlin: 00:30Z is 02:30 CEST, 01:30Z is 02:30 CET.
    assert_eq!(
        until_of("FREQ=HOURLY;UNTIL=20261025T003000Z"),
        "2026-10-25T02:30:00"
    );
    assert_eq!(
        until_of("FREQ=HOURLY;UNTIL=20261025T013000Z"),
        "2026-10-25T02:30:00"
    );
}

#[test]
fn a_date_with_z_is_refused() {
    let text = "FREQ=DAILY;UNTIL=20261231Z";
    assert_eq!(
        RecurrenceRule::parse_rrule_in(text, &berlin()),
        Err(RruleProblem::Value("UNTIL"))
    );
    assert_eq!(
        RecurrenceRule::parse_rrule(text),
        Err(RruleProblem::Value("UNTIL"))
    );
    assert_eq!(
        Recurrence::from_rrule_in(text, &berlin()).recurrence,
        Recurrence::Original(text.to_owned())
    );
}

#[test]
fn a_date_until_with_a_timed_reference_counts_the_whole_day_and_is_reported() {
    let read =
        RecurrenceRule::parse_rrule_in("FREQ=DAILY;UNTIL=20261231", &berlin()).expect("rule");
    assert!(read.until_adjusted);
    let RuleEnd::Until(t) = read.rule.end else {
        panic!("until")
    };
    assert_eq!((t.hour(), t.minute(), t.second()), (23, 59, 59));
    let kept = Recurrence::from_rrule_in("FREQ=DAILY;UNTIL=20261231", &berlin());
    assert!(kept.until_adjusted);
    // Export: always the RFC form for a timed start (UTC date-time; Berlin winter: 22:59:59Z).
    assert_eq!(
        read.rule.to_rrule_in(&berlin()).expect("text"),
        "FREQ=DAILY;UNTIL=20261231T225959Z"
    );
    // Without a zone (reference is a date) the date stays the plain day and nothing is adjusted.
    assert!(
        !RecurrenceRule::parse_rrule_in("FREQ=DAILY;UNTIL=20261231T235959", &berlin())
            .expect("rule")
            .until_adjusted
    );
    assert!(
        !Recurrence::from_rrule_in("FREQ=DAILY;UNTIL=20261231T235959Z", &berlin()).until_adjusted
    );
}
