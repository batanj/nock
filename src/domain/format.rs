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
        .unwrap_or_default()
        .trim()
        .to_string();

    let charge = fs::read_to_string(path.join("capacity"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok());

    let icon = match (charge, status.as_str()) {
        (Some(0..=24), "Charging") => "󰢜",
        (Some(25..=50), "Charging") => "󰂈",
        (Some(51..=75), "Charging") => "󰂉",
        (Some(76..=99), "Charging") => "󰂊",
        (Some(100), "Charging") => "󰂄",

        (Some(0..=24), "Discharging") => "󰁺",
        (Some(25..=50), "Discharging") => "󰁻",
        (Some(51..=75), "Discharging") => "󰁽",
        (Some(76..=99), "Discharging") => "󰁿",
        (Some(100), "Discharging") => "󰁹",

        (Some(0..=99), "Not charging") => "󰚥",
        (Some(100), "Not charging") => "󰂄",

        _ => "󰂑",
    };

    icon.to_string()
}

pub fn volume_text() -> String {
    let Ok(output) = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    else {
        return String::new();
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let is_muted = text.contains("[MUTED]");

    let volume = text
        .split_whitespace()
        .find_map(|word| word.parse::<f32>().ok())
        .map(|vol| (vol * 100.0).round() as u32);

    match (volume, is_muted) {
        (Some(0..=20), false) => String::from("󰕿"),
        (Some(21..=50), false) => String::from("󰖀"),
        (Some(51..=100), false) => String::from("󰕾"),
        (Some(_), _) => String::from("󰝟"),
        (None, _) => String::new(),
    }
}

pub fn network_text() -> String {
    match Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
        .output()
    {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.lines().any(|line| line.contains("yes:")) {
                "󰖩"
            } else {
                "󰖪"
            }
            .to_string()
        }
        _ => String::new(),
    }
}

pub fn brightness_text() -> String {
    let Ok(current) = Command::new("brightnessctl").args(["g"]).output() else {
        return String::new();
    };

    let Ok(max) = Command::new("brightnessctl").args(["m"]).output() else {
        return String::new();
    };

    let parse_brightness = |output: &std::process::Output| -> f32 {
        std::str::from_utf8(output.stdout.trim_ascii())
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    };

    let current_val = parse_brightness(&current);
    let max_val = parse_brightness(&max);

    if max_val <= 0.0 {
        return String::new();
    }

    let percentage = (current_val / max_val) * 100.0;

    let icon = match percentage as u32 {
        0..=39 => "󰃞",
        40..=74 => "󰃟",
        75..=100 => "󰃠",
        _ => "󰅚",
    };

    icon.to_string()
}
