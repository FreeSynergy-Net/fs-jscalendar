# Changelog

Versions follow ADR 0064 (0.0.N). No tag is set by the work branch.

## 0.0.3

- Only the pin changes: `fs-types` points to the tag `v0.0.26` of `fs-interfaces` (was `v0.0.22`), so that a program that uses `fs-jscalendar`, `fs-objects` and `fs-rpc` has exactly one `fs-types` in its lock (ADR 0092 draft). No code change.
- Auf Deutsch: Nur der Pin aendert sich (`fs-types` aus `fs-interfaces` v0.0.26 statt v0.0.22); kein Code geaendert.

## 0.0.2 (unreleased)

- New modules `recurrence` (typed `RecurrenceRule`, `Recurrence::{Rule, Original}`, RRULE reader/writer for the D5 subset), `origin` (`Origin`, `OriginChannel`) and `alert` (`Alert`, `Trigger`, `RelativeTo`, `AlertAction`): the typed types calendar and tasks share. Conversion from the calendar stand-ins is lossless both ways.
- New dependency `fs-types`, pinned to the tag `v0.0.22` of `fs-interfaces`.
- `parse_rrule_in` (UTC `UNTIL` to the zone's wall clock), `to_rrule_for(date_only)`, RFC 5545 combination checks, typed `OriginalText` error.
- `to_rrule_in(zone)` (UTC `UNTIL` export), `parse_rrule_in` returns `ReadRule` with `until_adjusted` (date `UNTIL` with a timed start counts the whole day), `YYYYMMDDZ` refused; zone arithmetic through the crate's own `time::Zone`.
- Existing parser, writer and `model` are unchanged.
