//! iCalendar <-> JSCalendar conversion through `calcard-next`.
//!
//! Pattern: Adapter — `calcard-next` (MIT OR Apache-2.0) maps iCalendar to JSCalendar and back; we hand
//! it JSON text and pass on JSON values. Imported text is kept as the original (ADR 0030).

use calcard_next::icalendar::ICalendar;
use calcard_next::jscalendar::JSCalendar;
use serde_json::{Value, json};

use crate::error::{Error, Result};
use crate::upgrade::upgrade;

/// Media type of the stored original.
pub const ICS_MEDIA_TYPE: &str = "text/calendar";

/// Convert one iCalendar text to upgraded JSCalendar 2.0 values of `@type` `at_type` (all other
/// entries are left out) plus upgrade notes.
pub fn parse_entries(text: &str, at_type: &str) -> Result<(Vec<Value>, Vec<String>)> {
    // Foreign files reach this parser; calcard-next may panic on malformed input (see `shielded`).
    let pretty = shielded(|| {
        let ical =
            ICalendar::parse(text).map_err(|e| Error::ICalendar(format!("cannot parse: {e:?}")))?;
        Ok(ical.into_jscalendar::<String, String>().to_string_pretty())
    })?;
    let value: Value = serde_json::from_str(&pretty)?;
    let up = upgrade(value);
    let entries = match up.value.get("entries") {
        Some(Value::Array(a)) => a.clone(),
        _ => vec![up.value.clone()],
    };
    let wanted = entries
        .into_iter()
        .filter(|e| e.get("@type").and_then(Value::as_str) == Some(at_type))
        .collect();
    Ok((wanted, up.notes))
}

/// Write JSCalendar values as one iCalendar stream, naming `prod_id` as the producer.
pub fn write_ics(objects: impl IntoIterator<Item = Value>, prod_id: &str) -> Result<String> {
    // calcard-next understands `version: "2.0"` on entries (0.3.14 did not), so nothing is removed.
    let entries: Vec<Value> = objects
        .into_iter()
        .map(|mut v| {
            prepare_participants(&mut v);
            v
        })
        .collect();
    let group = json!({
        "@type": "Group",
        "uid": crate::new_uid(),
        "updated": crate::now_utc(),
        "prodId": prod_id,
        "entries": entries,
    });
    let text = group.to_string();
    // calcard-next asserts internally on malformed input; a panic must never take the program down.
    shielded(|| {
        let js = JSCalendar::<String, String>::parse(&text)
            .map_err(|e| Error::ICalendar(format!("JSCalendar rejected: {e}")))?;
        let ical = js
            .into_icalendar()
            .map_err(|e| Error::ICalendar(format!("cannot be expressed as iCalendar: {e}")))?;
        Ok(ical.to_string())
    })
}

/// Run a call into the foreign converter; a panic inside it becomes `Error::ICalendar`, never
/// reaches the caller.
fn shielded<T>(f: impl FnOnce() -> Result<T> + std::panic::UnwindSafe) -> Result<T> {
    std::panic::catch_unwind(f).unwrap_or_else(|_| {
        Err(Error::ICalendar(
            "the converter failed on this input".into(),
        ))
    })
}

/// iCalendar attendees need an address: fill `calendarAddress` from `email` or `sendTo.imip`, and
/// leave out participants that have none (they cannot be expressed as ATTENDEE).
fn prepare_participants(v: &mut Value) {
    let Some(ps) = v.get_mut("participants").and_then(Value::as_object_mut) else {
        return;
    };
    ps.retain(|_, p| {
        let Some(o) = p.as_object_mut() else {
            return false;
        };
        if !o.contains_key("calendarAddress") {
            let addr = o
                .get("email")
                .and_then(Value::as_str)
                .map(|e| format!("mailto:{e}"))
                .or_else(|| {
                    o.get("sendTo")
                        .and_then(|s| s.get("imip"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                });
            if let Some(a) = addr {
                o.insert("calendarAddress".into(), Value::String(a));
            }
        }
        o.contains_key("calendarAddress")
    });
    if ps.is_empty() {
        v.as_object_mut().map(|m| m.remove("participants"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // No input is known that makes calcard-next 3.0.1 panic, so the seam itself is tested with a
    // closure that panics; `parse_entries` and `write_ics` both go through the same seam.
    #[test]
    fn a_panic_in_the_converter_becomes_an_error() {
        let r: Result<()> = shielded(|| panic!("boom"));
        assert!(matches!(r, Err(Error::ICalendar(m)) if m.contains("failed")));
    }
}
