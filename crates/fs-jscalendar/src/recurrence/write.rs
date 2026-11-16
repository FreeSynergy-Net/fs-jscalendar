//! Writing a typed rule as RRULE text.
//!
//! Pattern: Adapter (typed rule to the text form of RFC 5545 / RFC 7529).

use std::fmt::{Display, Write as _};

use fs_types::LocalDateTime;

use super::{Frequency, NDay, RecurrenceRule, RuleEnd, Skip, Weekday};

const fn weekday(day: Weekday) -> &'static str {
    match day {
        Weekday::Mo => "MO",
        Weekday::Tu => "TU",
        Weekday::We => "WE",
        Weekday::Th => "TH",
        Weekday::Fr => "FR",
        Weekday::Sa => "SA",
        Weekday::Su => "SU",
    }
}

const fn frequency(f: Frequency) -> &'static str {
    match f {
        Frequency::Yearly => "YEARLY",
        Frequency::Monthly => "MONTHLY",
        Frequency::Weekly => "WEEKLY",
        Frequency::Daily => "DAILY",
        Frequency::Hourly => "HOURLY",
        Frequency::Minutely => "MINUTELY",
        Frequency::Secondly => "SECONDLY",
    }
}

fn local(at: LocalDateTime) -> String {
    let date = at.date();
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}",
        date.year(),
        date.month(),
        date.day(),
        at.hour(),
        at.minute(),
        at.second()
    )
}

fn list<T: Display>(out: &mut String, name: &str, items: impl IntoIterator<Item = T>) {
    let joined: Vec<String> = items.into_iter().map(|i| i.to_string()).collect();
    if !joined.is_empty() {
        let _ = write!(out, ";{name}={}", joined.join(","));
    }
}

fn nday(n: NDay) -> String {
    n.nth.map_or_else(
        || weekday(n.day).to_owned(),
        |nth| format!("{nth}{}", weekday(n.day)),
    )
}

impl RecurrenceRule {
    /// The rule as RRULE text (RFC 5545; `RSCALE` and `SKIP` of RFC 7529 when set). Reading it
    /// back with [`RecurrenceRule::parse_rrule`] gives the same rule whenever the rule is inside
    /// the readable subset.
    #[must_use]
    pub fn to_rrule(&self) -> String {
        let mut out = String::new();
        if let Some(rscale) = &self.rscale {
            let _ = write!(out, "RSCALE={rscale};");
        }
        let _ = write!(out, "FREQ={}", frequency(self.frequency));
        if self.interval != 1 {
            let _ = write!(out, ";INTERVAL={}", self.interval);
        }
        match self.skip {
            Skip::Omit => {}
            Skip::Backward => out.push_str(";SKIP=BACKWARD"),
            Skip::Forward => out.push_str(";SKIP=FORWARD"),
        }
        if self.first_day_of_week != Weekday::Mo {
            let _ = write!(out, ";WKST={}", weekday(self.first_day_of_week));
        }
        list(&mut out, "BYDAY", self.by_day.iter().copied().map(nday));
        list(&mut out, "BYMONTHDAY", &self.by_month_day);
        list(
            &mut out,
            "BYMONTH",
            self.by_month
                .iter()
                .map(|m| format!("{}{}", m.month, if m.leap { "L" } else { "" })),
        );
        list(&mut out, "BYYEARDAY", &self.by_year_day);
        list(&mut out, "BYWEEKNO", &self.by_week_no);
        list(&mut out, "BYHOUR", &self.by_hour);
        list(&mut out, "BYMINUTE", &self.by_minute);
        list(&mut out, "BYSECOND", &self.by_second);
        list(&mut out, "BYSETPOS", &self.by_set_position);
        match self.end {
            RuleEnd::Never => {}
            RuleEnd::Count(n) => {
                let _ = write!(out, ";COUNT={n}");
            }
            RuleEnd::Until(at) => {
                let _ = write!(out, ";UNTIL={}", local(at));
            }
        }
        out
    }
}
