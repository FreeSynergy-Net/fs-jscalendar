//! An alert relative to `End`: for a task that is its due time (RFC 8984).
//!
//! Pattern: Specification by example.

use fs_jscalendar::alert::{Alert, AlertAction, RelativeTo, Trigger};

#[test]
fn offset_relative_to_end_keeps_its_form() {
    let a = Alert {
        key: "due".into(),
        trigger: Trigger::Offset {
            seconds: -3600,
            relative_to: RelativeTo::End,
        },
        action: AlertAction::Display,
        acknowledged: None,
    };
    assert_eq!(a.clone(), a);
    assert!(matches!(
        a.trigger,
        Trigger::Offset {
            relative_to: RelativeTo::End,
            seconds: -3600
        }
    ));
    assert_ne!(RelativeTo::Start, RelativeTo::End);
}
