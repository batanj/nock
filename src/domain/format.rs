use std::{fs, path::PathBuf, process::Command};

use chrono::{Local, Timelike};

use crate::error::AppError;

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

pub fn battery_text() -> Result<String, AppError> {
    let path = (0..=1)
        .map(|i| PathBuf::from(format!("/sys/class/power_supply/BAT{}", i)))
        .find(|path| path.exists())
        .ok_or(AppError::BatteryNotFound)?;

    let read_batt_file = |name: &str| -> Result<String, AppError> {
        let file_path = path.join(name);
        fs::read_to_string(&file_path).map_err(|source| AppError::ReadError {
            file: file_path,
            source,
        })
    };

    let status = read_batt_file("status")?.trim().to_string();

    let charge = read_batt_file("capacity")?
        .trim()
        .parse::<u32>()
        .map_err(AppError::ParseIntError)?;

    let icon = match (charge, status.as_str()) {
        (0..=24, "Charging") => "󰢜",
        (25..=50, "Charging") => "󰂈",
        (51..=75, "Charging") => "󰂉",
        (76..=99, "Charging") => "󰂊",
        (100, "Charging") => "󰂄",

        (0..=24, "Discharging") => "󰁺",
        (25..=50, "Discharging") => "󰁻",
        (51..=75, "Discharging") => "󰁽",
        (76..=99, "Discharging") => "󰁿",
        (100, "Discharging") => "󰁹",

        (0..=99, "Not charging") => "󰚥",
        (100, "Not charging") => "󰂄",

        _ => "󰂑",
    };

    Ok(icon.to_string())
}

pub fn volume_text() -> Result<String, AppError> {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let is_muted = text.contains("[MUTED]");

    let volume = text
        .split_whitespace()
        .find_map(|word| word.parse::<f32>().ok())
        .map(|vol| (vol * 100.0).round() as u32);

    let icon = match (volume, is_muted) {
        (Some(0..=20), false) => String::from("󰕿"),
        (Some(21..=50), false) => String::from("󰖀"),
        (Some(51..=100), false) => String::from("󰕾"),
        (Some(_), _) => String::from("󰝟"),
        (None, _) => String::new(),
    };

    Ok(icon)
}

pub fn network_text() -> Result<String, AppError> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let icon = if text.lines().any(|line| line.contains("yes:")) {
        "󰖩"
    } else {
        "󰖪"
    };

    Ok(icon.to_string())
}

pub fn brightness_text() -> Result<String, AppError> {
    let current = Command::new("brightnessctl").args(["g"]).output()?;
    let max = Command::new("brightnessctl").args(["m"]).output()?;

    let parse_brightness = |output: &std::process::Output| -> Result<f32, AppError> {
        let val = std::str::from_utf8(&output.stdout)?.trim().parse()?;
        Ok(val)
    };

    let current_val = parse_brightness(&current)?;
    let max_val = parse_brightness(&max)?;

    if max_val <= 0.0 {
        return Ok(String::new());
    }

    let percentage = (current_val / max_val) * 100.0;

    let icon = match percentage as u32 {
        0..=39 => "󰃞",
        40..=74 => "󰃟",
        75..=100 => "󰃠",
        _ => "󰅚",
    };

    Ok(icon.to_string())
}
