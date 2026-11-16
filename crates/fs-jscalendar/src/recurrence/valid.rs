//! Which parts of an RRULE may stand together (RFC 5545 section 3.3.10).
//!
//! Pattern: Specification (one rule per combination); a failure keeps the text as `Original`.

use super::{Frequency, RecurrenceRule, RruleProblem};

/// Refuse combinations the RFC forbids, so a typed rule is always one the owners' engines can
/// define. `seen` are the part names of the text.
pub(super) fn check_combination(rule: &RecurrenceRule, seen: &[&str]) -> Result<(), RruleProblem> {
    let has = |name: &str| seen.contains(&name);
    let bad = |why| Err(RruleProblem::Combination(why));
    if has("BYSETPOS") && !(has("BYDAY") || has("BYMONTHDAY") || has("BYMONTH")) {
        return bad("BYSETPOS needs another BYxxx part");
    }
    if has("BYMONTHDAY") && rule.frequency == Frequency::Weekly {
        return bad("BYMONTHDAY with FREQ=WEEKLY");
    }
    let limit = match rule.frequency {
        Frequency::Yearly => 53,
        Frequency::Monthly => 5,
        _ => 0,
    };
    if rule
        .by_day
        .iter()
        .any(|n| n.nth.is_some_and(|v| v.abs() > limit))
    {
        return bad("ordinal BYDAY only with MONTHLY (up to 5) or YEARLY (up to 53)");
    }
    Ok(())
}
