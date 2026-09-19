use std::{fs, path::PathBuf, process::Command};

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

pub fn battery_text() -> String {
    let path = (0..=1)
        .map(|i| PathBuf::from(format!("/sys/class/power_supply/BAT{}", i)))
        .find(|path| path.exists())
        .unwrap_or_default();

    let status = fs::read_to_string(path.join("status"))
        .ok()
        .unwrap_or_default();
    let charge = fs::read_to_string(path.join("capacity"))
        .ok()
        .unwrap_or_default();

    match (charge, status) {
        (ch, st) if !ch.is_empty() && !st.is_empty() => {
            format!(
                "{}%{}",
                ch.trim(),
                if st.trim() == "Charging" { "+" } else { "" }
            )
        }
        (ch, _) => ch.to_string(),
    }
}

pub fn volume_text() -> String {
    match Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let is_muted = text.contains("[MUTED]");

            let volume = text
                .split_whitespace()
                .find_map(|word| word.parse::<f32>().ok())
                .map(|vol| (vol * 100.0).round() as u32);

            match (volume, is_muted) {
                (Some(vol), true) => format!("{} [MUTED]", vol),
                (Some(vol), false) => vol.to_string(),
                (None, _) => String::new(),
            }
        }
        _ => String::new(),
    }
}

pub fn network_text() -> String {
    match Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
        .output()
    {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);

            text.lines()
                .find(|line| line.contains("yes:"))
                .and_then(|line| line.strip_prefix("yes:").map(str::trim))
                .unwrap_or_default()
                .to_string()
        }
        _ => String::new(),
    }
}
