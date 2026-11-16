# Changelog

Versions follow ADR 0064 (0.0.N). No tag is set by the work branch.

## 0.0.2 (unreleased)

- New modules `recurrence` (typed `RecurrenceRule`, `Recurrence::{Rule, Original}`, RRULE reader/writer for the D5 subset), `origin` (`Origin`, `OriginChannel`) and `alert` (`Alert`, `Trigger`, `RelativeTo`, `AlertAction`): the typed types calendar and tasks share. Conversion from the calendar stand-ins is lossless both ways.
- New dependency `fs-types` (provisional path dependency on the unmerged branch `unreviewed/fs-types-gemeinsam`; becomes a tag pin after the merge).
- `parse_rrule_in` (UTC `UNTIL` to the zone's wall clock), `to_rrule_for(date_only)`, RFC 5545 combination checks, typed `OriginalText` error.
- Existing parser, writer and `model` are unchanged.
