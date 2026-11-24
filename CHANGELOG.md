# Changelog

Versions follow ADR 0064 (0.0.N). No tag is set by the work branch.

## Unreleased

- Only the pin changes: `fs-types` is lifted from the tag `v0.0.6` to `rev = 722d3ce0156711d40cb278f2e835e865d9448290` (v0.0.7 line), so the dependants see exactly one `fs-types` in their graph. No code change.
- Auf Deutsch: Nur der Pin aendert sich: `fs-types` steht statt auf Tag `v0.0.6` auf `rev = 722d3ce0156711d40cb278f2e835e865d9448290` (Linie v0.0.7), damit die Abhaengigen genau ein `fs-types` im Graph haben. Kein Code geaendert.
- Every call into `calcard-next` runs in a panic seam (`shielded`); new golden test `tests/golden_recurrence.rs` (Berlin DST, UNTIL, EXDATE, RECURRENCE-ID, written back to iCalendar) and tests that `upgrade()` lifts 1.0 `recurrenceRules` (one rule -> `recurrenceRule`; further rules kept in `freesynergy.net:legacy` with a note). No library code change in this correction.
- Auf Deutsch: Jeder Aufruf in `calcard-next` laeuft in einer Panic-Naht (`shielded`); neuer Golden-Test `tests/golden_recurrence.rs` und Tests, dass `upgrade()` die 1.0-`recurrenceRules` hebt (eine Regel -> `recurrenceRule`; weitere bleiben in `freesynergy.net:legacy` mit Notiz). Kein Bibliothekscode in dieser Korrektur geaendert.

## 0.0.7

- Only pins and the address change: the `fs-types` git URL is now `https://github.com/FreeSynergy-Net/fs-types` (tag `v0.0.6` unchanged), the workflow pins to `fs-release` are raised to `9eed5c95eb7311f2b68266a012ec778f51e3ac38` (v0.0.18). The lock holds exactly one `fs-types` source (the Net URL). No code change.
- Auf Deutsch: Nur Pins und Adresse aendern sich: die git-URL von `fs-types` ist jetzt `https://github.com/FreeSynergy-Net/fs-types` (Tag `v0.0.6` unveraendert), die Workflow-Pins auf `fs-release` stehen auf `9eed5c95eb7311f2b68266a012ec778f51e3ac38` (v0.0.18). Die Sperrdatei enthaelt genau eine `fs-types`-Quelle (die Net-URL). Kein Code geaendert.

## 0.0.6

- Only the pin changes: `fs-types` is lifted to the tag `v0.0.6` (was `v0.0.4`); the git URL stays `https://github.com/FreeSynergy/fs-types` (the URL change follows later in one wave). No code change.
- Auf Deutsch: Nur der Pin aendert sich: `fs-types` steht auf Tag `v0.0.6` (vorher `v0.0.4`); die git-URL bleibt `https://github.com/FreeSynergy/fs-types` (die URL-Umstellung folgt spaeter in einer Welle). Kein Code geaendert.

## 0.0.5

- Only the pin changes: `fs-types` is lifted to the tag `v0.0.4` of `FreeSynergy/fs-types` (was `v0.0.1`), so the chain (calendar, tasks, objects, rpc) stays on one `fs-types` (plan chain-lift, option A). The lock holds exactly one `fs-types` source. No code change was needed.
- Auf Deutsch: Nur der Pin aendert sich: `fs-types` steht jetzt auf Tag `v0.0.4` von `FreeSynergy/fs-types` (vorher `v0.0.1`), damit die Kette bei einem `fs-types` bleibt. Der Lock hat genau eine `fs-types`-Quelle. Kein Code geaendert.

## 0.0.4

- Only the pin changes: `fs-types` now comes directly from its own repository, the tag `v0.0.1` of `FreeSynergy/fs-types` (was the tag `v0.0.26` of `fs-interfaces`; ADR 0093). `fs-interfaces` is no longer in the lock, so the lock holds exactly one `fs-types` source. Workflow pin `fs-release` `@v0.0.10` (was `@main`); `.fs-lock-sources.toml` with `strict = true`. No code change.
- Auf Deutsch: Nur der Pin aendert sich: `fs-types` kommt direkt aus dem eigenen Repo (Tag `v0.0.1` von `FreeSynergy/fs-types`, vorher `fs-interfaces` v0.0.26; ADR 0093); `fs-interfaces` ist nicht mehr im Lock. Workflow-Pin `fs-release` `@v0.0.10`; Lock-Pruefung streng. Kein Code geaendert.

## 0.0.3

- Only the pin changes: `fs-types` points to the tag `v0.0.26` of `fs-interfaces` (was `v0.0.22`), so that a program that uses `fs-jscalendar`, `fs-objects` and `fs-rpc` has exactly one `fs-types` in its lock (ADR 0092, accepted). No code change.
- Auf Deutsch: Nur der Pin aendert sich (`fs-types` aus `fs-interfaces` v0.0.26 statt v0.0.22); kein Code geaendert.

## 0.0.2

- New modules `recurrence` (typed `RecurrenceRule`, `Recurrence::{Rule, Original}`, RRULE reader/writer for the D5 subset), `origin` (`Origin`, `OriginChannel`) and `alert` (`Alert`, `Trigger`, `RelativeTo`, `AlertAction`): the typed types calendar and tasks share. Conversion from the calendar stand-ins is lossless both ways.
- New dependency `fs-types`, pinned to the tag `v0.0.22` of `fs-interfaces`.
- `parse_rrule_in` (UTC `UNTIL` to the zone's wall clock), `to_rrule_for(date_only)`, RFC 5545 combination checks, typed `OriginalText` error.
- `to_rrule_in(zone)` (UTC `UNTIL` export), `parse_rrule_in` returns `ReadRule` with `until_adjusted` (date `UNTIL` with a timed start counts the whole day), `YYYYMMDDZ` refused; zone arithmetic through the crate's own `time::Zone`.
- Existing parser, writer and `model` are unchanged.
