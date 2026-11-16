//! Typed recurrence: reading and writing RRULE text, and keeping what is not interpretable.
//!
//! Pattern: Specification by example — each case states one rule of the subset.

use fs_jscalendar::recurrence::{
    ByMonth, Frequency, NDay, OriginalText, Recurrence, RecurrenceRule, RruleProblem, RuleEnd,
    Skip, Weekday,
};

fn rule(text: &str) -> RecurrenceRule {
    RecurrenceRule::parse_rrule(text).expect("inside the subset")
}

#[test]
fn reads_the_subset() {
    let r = rule("FREQ=MONTHLY;INTERVAL=2;BYDAY=-1FR,MO;BYMONTH=3,9;BYSETPOS=1;WKST=SU;COUNT=10");
    assert_eq!(r.frequency, Frequency::Monthly);
    assert_eq!(r.interval, 2);
    assert_eq!(
        r.by_day,
        vec![
            NDay {
                day: Weekday::Fr,
                nth: Some(-1)
            },
            NDay {
                day: Weekday::Mo,
                nth: None
            }
        ]
    );
    assert_eq!(
        r.by_month,
        vec![
            ByMonth {
                month: 3,
                leap: false
            },
            ByMonth {
                month: 9,
                leap: false
            }
        ]
    );
    assert_eq!(r.by_set_position, vec![1]);
    assert_eq!(r.first_day_of_week, Weekday::Su);
    assert_eq!(r.end, RuleEnd::Count(10));
    assert_eq!(r.skip, Skip::Omit);
}

#[test]
fn until_wall_clock_and_date() {
    let a = rule("FREQ=DAILY;UNTIL=20261231T235959");
    let b = rule("FREQ=DAILY;UNTIL=20261231");
    let (RuleEnd::Until(a), RuleEnd::Until(b)) = (a.end, b.end) else {
        panic!("until")
    };
    assert_eq!((a.hour(), a.minute(), a.second()), (23, 59, 59));
    assert_eq!((b.hour(), b.minute(), b.second()), (0, 0, 0));
    assert_eq!(a.date(), b.date());
}

#[test]
fn write_then_read_is_the_same_rule() {
    for text in [
        "FREQ=DAILY",
        "FREQ=MONTHLY;INTERVAL=3;WKST=SU;BYDAY=MO,2WE;COUNT=4",
        "FREQ=YEARLY;BYMONTH=2;BYMONTHDAY=29;UNTIL=20300101T000000",
    ] {
        let r = rule(text);
        assert_eq!(rule(&r.to_rrule()), r, "{text}");
    }
}

#[test]
fn outside_the_subset_the_text_is_kept_byte_for_byte() {
    for text in [
        "FREQ=DAILY;BYHOUR=9",
        "FREQ=DAILY;UNTIL=20261231T000000Z",
        "FREQ=DAILY;COUNT=2;UNTIL=20261231",
        "RSCALE=HEBREW;FREQ=YEARLY",
        "FREQ=SOMETIMES",
        "INTERVAL=2",
        "nonsense",
        "FREQ=DAILY;FREQ=DAILY",
    ] {
        let kept = Recurrence::from_rrule(text);
        assert_eq!(kept, Recurrence::Original(text.to_owned()), "{text}");
        assert_eq!(kept.to_rrule(), text);
        assert!(kept.rule().is_none());
    }
}

#[test]
fn typed_problems() {
    assert_eq!(
        RecurrenceRule::parse_rrule("FREQ=DAILY;BYHOUR=9"),
        Err(RruleProblem::Unsupported("BYHOUR".into()))
    );
    assert_eq!(
        RecurrenceRule::parse_rrule("FREQ=DAILY;INTERVAL=0"),
        Err(RruleProblem::Value("INTERVAL"))
    );
    assert_eq!(
        RecurrenceRule::parse_rrule("FREQ=DAILY;BYDAY=XX"),
        Err(RruleProblem::Value("BYDAY"))
    );
    assert_eq!(
        RecurrenceRule::parse_rrule("FREQ=DAILY;UNTIL=20261301"),
        Err(RruleProblem::Value("UNTIL"))
    );
    assert_eq!(
        RecurrenceRule::parse_rrule("COUNT=3"),
        Err(RruleProblem::Frequency)
    );
}

#[test]
fn rule_with_text_conversions() {
    let r = rule("FREQ=DAILY");
    let wrapped = Recurrence::from(r.clone());
    assert_eq!(RecurrenceRule::try_from(wrapped), Ok(r));
    assert_eq!(
        RecurrenceRule::try_from(Recurrence::Original("X".into())),
        Err(OriginalText("X".to_owned()))
    );
}

fn berlin() -> fs_types::Zone {
    fs_types::Zone::iana("Europe/Berlin").expect("zone")
}

fn until_of(text: &str) -> String {
    let RuleEnd::Until(t) = RecurrenceRule::parse_rrule_in(text, &berlin())
        .expect("rule")
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
        Recurrence::from_rrule_in(text, &custom),
        Recurrence::Original(text.to_owned())
    );
    assert!(Recurrence::from_rrule_in(text, &berlin()).rule().is_some());
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
fn combinations_the_rfc_forbids_stay_original() {
    for (text, why) in [
        ("FREQ=MONTHLY;BYSETPOS=1", "BYSETPOS"),
        ("FREQ=WEEKLY;BYDAY=2MO", "ordinal"),
        ("FREQ=WEEKLY;BYMONTHDAY=1", "BYMONTHDAY"),
        ("FREQ=MONTHLY;BYDAY=6MO", "ordinal"),
        ("FREQ=DAILY;BYDAY=1MO", "ordinal"),
    ] {
        assert!(
            matches!(RecurrenceRule::parse_rrule(text), Err(RruleProblem::Combination(m)) if m.contains(why)),
            "{text}"
        );
        assert_eq!(
            Recurrence::from_rrule(text),
            Recurrence::Original(text.to_owned())
        );
    }
    assert!(RecurrenceRule::parse_rrule("FREQ=YEARLY;BYDAY=20MO").is_ok());
    assert!(RecurrenceRule::parse_rrule("FREQ=MONTHLY;BYDAY=MO;BYSETPOS=-1").is_ok());
}

#[test]
fn a_typed_rule_with_parts_outside_the_subset_reads_back_as_original() {
    let mut r = RecurrenceRule::every(Frequency::Daily, 1);
    r.by_hour = vec![9];
    let text = r.to_rrule();
    assert_eq!(text, "FREQ=DAILY;BYHOUR=9");
    assert_eq!(Recurrence::from_rrule(&text), Recurrence::Original(text));
}
