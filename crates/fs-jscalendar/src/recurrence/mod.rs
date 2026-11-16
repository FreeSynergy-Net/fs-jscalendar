//! Recurrence shared by calendar and tasks: the typed rule, or the original text of a rule the
//! owners cannot interpret.
//!
//! Pattern: Facade (this file only declares and re-exports); the value object `Recurrence` lives
//! in `keep.rs`.

mod keep;
mod read;
mod rule;
mod until;
mod valid;
mod write;

pub use keep::{OriginalText, ReadRecurrence, Recurrence};
pub use read::{ReadRule, RruleProblem};
pub use rule::{ByMonth, Frequency, NDay, RecurrenceRule, RuleEnd, Skip, Weekday};
