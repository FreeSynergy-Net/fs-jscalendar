# fs-jscalendar – Design

The parts `Event` (owner: `fs-calendars`) and `Task` (owner: `fs-tasks`) have in common, so neither copies them (ADR 0043 §4).
ADR 0050 allows exactly this: shared **types** of a standard may live in a plain library. It holds no data, stores nothing,
has no interface and is not a program. The point of truth for each object stays with its owner.

**What does not belong here:** `Event`, `Task`, repositories, the invitation flow and recurrence expansion (`fs-calendars`), relations and
task status (`fs-tasks`). A type moves here only when both owners really use it.

## State

Version 0.0.5. Taken from `fs-tasks-core` and `fs-calendars-core`, which held identical copies.

| Area | State |
|---|---|
| `time`: `Zone`, `ZoneSource` (system / bundled tzdb, newest wins), `Dur`, `LocalDateTime` / `UTCDateTime` | done |
| `model`: `Participant`, `RecurrenceRule`, `NDay`, aliases, constants, `next_participant_key` | done |
| `calendar_object!` / `typed_object!`: members and common methods of an object, lossless `extra` | done |
| `upgrade`: RFC 8984 to 2.0, idempotent, nothing dropped; fractions leave every date-time and duration of the object, durations round up; values in alerts and participants stay as they are | done |
| `validate`: `Ctx`, `common`, `shared`, recurrence rule; owners add their own rules | done |
| `ical`: `split_ics(text, Component)`, `unfolded_lines`, `parse_entries`, `write_ics` through `calcard-next =3.0.1`; components are found on unfolded lines, so a continuation line never ends one | done |
| Date-times: exactly `YYYY-MM-DDTHH:MM:SS`, second 60 refused; messages quote a value short and escaped (`quoted`) | done |
| `recurrence`: typed `RecurrenceRule` (calendar plan D5 subset), `Recurrence::{Rule, Original(text)}`, RRULE text reader (`parse_rrule`, `parse_rrule_in` with the zone for a UTC or date `UNTIL`; RFC 5545 3.3.10 combinations checked) and writer (`to_rrule`, `to_rrule_for(date_only)`, `to_rrule_in(zone)`) | done (0.0.2) |
| `origin`: `Origin`, `OriginChannel` (fs-objects `object_origin`) | done (0.0.2) |
| `alert`: `Alert`, `Trigger`, `RelativeTo`, `AlertAction` | done (0.0.2) |
| Tests | 34 here; the behaviour is also covered by the tests of both owners |

## Structure and pattern map (ADR 0043)

| Module | Pattern | Why |
|---|---|---|
| `model/macros.rs` | Template method (macro form) | fixes the skeleton of an object; owners list only their own members |
| `model/{participant,rule}.rs` | Value object | plain serde data, lossless `extra` |
| `recurrence/rule.rs`, `origin.rs`, `alert.rs` | Value object | typed shapes shared by calendar and tasks, field for field the ones both topic traits use; basic types from `fs-types` |
| `recurrence/{read,write}.rs` | Adapter | RRULE text <-> typed rule; outside the subset the text is kept (`Recurrence::Original`), no JSON in-between form (ADR 0030) |
| `time/source.rs`, `bundled.rs`, `system.rs` | Strategy + Template method | `ZoneSource`: where tz rules come from |
| `time/select.rs` | Factory | picks the newest of system and bundled tzdb at first use |
| `time/{zone,dur,format}.rs` | Value objects | `Zone`, nominal-day + exact-second `Dur`, wire formats |
| `validate/*` | Specification + Collecting parameter | rules report into one `Ctx`; all issues at once |
| `upgrade.rs` | Pipeline | small rewrite steps, idempotent, with notes |
| `ical/convert.rs` | Adapter | wraps `calcard-next`; passes JSON values, owners make them typed |
| `ical/split.rs` | Iterator-style scan | exact source text per UID, so the original stays byte for byte (ADR 0030) |
| `upgrade_times.rs` | Pipeline step | whole seconds for `upgrade`, every original kept |
| `error.rs`, `clock.rs`, `uid.rs` | none | plain error enum and pure helpers |

## Decisions

- **Errors:** owners turn `fs_jscalendar::Error` into their own with one `From` and keep the same variants.
- **Panic seam and golden test:** every call into `calcard-next` runs inside `shielded` (`catch_unwind`, a panic becomes `Error`); `tests/golden_recurrence.rs` fixes recurrence, UNTIL, EXDATE, RECURRENCE-ID and the Berlin DST change against hand-derived JSCalendar 2.0 values.
- **`calcard-next` is pinned to `=3.0.1`:** the maintained fork of calcard 0.3.14 (Apache-2.0 OR MIT); the own `upgrade.rs` stays (it keeps obsolete 1.0 data in the legacy property, rounds fractions and reports notes; `JSCalendar::upgrade` does none of that and works on typed values).
- **`uid`s without `fs-store`:** this crate stores nothing, so it does not depend on the store. `new_uid` makes the same `urn:uuid:` + UUIDv7 form.
- **Typed shared types (0.0.2):** `recurrence`, `origin` and `alert` are the single home of the types that calendar and tasks both need (decision 2026-11-16, no stand-in copies in `tasks_v1`). They are *not* the string-typed wire form of `model::RecurrenceRule`; that one stays for the JSON object. `alert::RelativeTo::End` is, for a task, its due time (RFC 8984).
- **Conversion from the locked calendar stand-ins** is lossless both ways (test `fs-jscalendar-umwandlung-test.rs` in the calendar pilot): `TryFrom` where `fs-types` is stricter than the stand-in (a refused value is an error, never changed). `Recurrence::Original` has no stand-in form; the way back reports the text.
- **Supported rule subset (D5):** `FREQ`, `INTERVAL`, `COUNT` or `UNTIL` (wall clock or date), `BYDAY`, `BYMONTHDAY`, `BYMONTH`, `BYSETPOS`, `WKST`. The typed rule has all RFC 8984 fields so the owners' engines can grow; the reader accepts only the subset.
- **`fs-types` pin:** `fs-types` is pinned to the tag `v0.0.4` of its own repository `FreeSynergy/fs-types` (ADR 0093) like the other pins; the lock holds exactly one `fs-types` source (ADR 0092 rule 2). Calendar and tasks must pin the same tag, otherwise `LocalDateTime` and `Utc` are different types.
- **Reference point with a time (`parse_rrule_in` / `to_rrule_in`):** a UTC `UNTIL` is read in the zone of the reference point through the crate's own `time::Zone` (the tz database recorded with every import); a date `UNTIL` is read tolerantly as the end of that day and reported in `ReadRule::until_adjusted` (loss/normalisation note); the export always writes the RFC 5545 form (UTC date-time). A date with `Z` is refused.
- **Text round trip:** a typed rule written and read back is the same rule, not the same text (`+1MO` becomes `1MO`, `INTERVAL=1` and `WKST=MO` fall away). "Unchanged" stays byte for byte through `Origin::original` (ADR 0085/0086). A typed rule with parts outside the readable subset (`BYHOUR`, `BYYEARDAY`, ...) reads back as `Original`.
