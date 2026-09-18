use chrono::{Local, Timelike};

pub fn format_now(hour12: bool) -> String {
    let now = Local::now();
    let mut hour = now.hour();
    let minute = now.minute();
    let second = now.second();
    let mut is_pm = false;

    if hour12 {
        (is_pm, hour) = now.hour12();
    }

    let am_pm = match (hour12, is_pm) {
        (true, true) => " pm",
        (true, false) => " am",
        (false, _) => "",
    };
    format!("{hour:02}:{minute:02}:{second:02}{}", am_pm)
}

pub fn format_date() -> String {
    let now = Local::now();
    now.format("%a %b %d").to_string()
}
