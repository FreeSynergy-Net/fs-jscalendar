//! Recurrence shared by calendar and tasks: the typed rule, or the original text of a rule the
//! owners cannot interpret.
//!
//! Pattern: Facade (this file only declares and re-exports); the value object `Recurrence` lives
//! in `keep.rs`.

mod keep;
mod read;
mod rule;
mod write;

pub use keep::Recurrence;
pub use read::RruleProblem;
pub use rule::{ByMonth, Frequency, NDay, RecurrenceRule, RuleEnd, Skip, Weekday};
