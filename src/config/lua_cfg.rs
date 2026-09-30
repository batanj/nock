use std::{
    env, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use mlua::{Lua, Table};

use crate::{
    Widget,
    runtime::{bar::Bar, nock::Nock},
};

fn config_path(root_widget: impl AsRef<Path>) -> Option<PathBuf> {
    config_candidates(root_widget)
        .into_iter()
        .find(|p| p.is_file())
}

fn config_candidates(root_widget: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut v = vec![];
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        v.push(
            PathBuf::from(xdg)
                .join("nock")
                .join(&root_widget)
                .with_extension("lua"),
        );
    }
    if let Ok(home) = env::var("HOME") {
        v.push(
            PathBuf::from(home)
                .join(".config")
                .join("nock")
                .join(&root_widget)
                .with_extension("lua"),
        );
    }
    v.push(root_widget.as_ref().with_extension("lua"));
    v
}

pub fn load_root(root_widget: impl AsRef<Path>) -> (Option<Widget>, Option<(PathBuf, SystemTime)>) {
    if let Some(path) = config_path(root_widget) {
        let meta = fs::metadata(&path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        return match std::fs::read_to_string(&path) {
            Ok(s) => (
                widget_from_lua(&s)
                    .map_err(|e| eprintln!("nock: {path:?}: {e}"))
                    .ok(),
                Some((path, meta)),
            ),
            Err(e) => {
                eprintln!("nock: {path:?}: {e}");
                (None, Some((path, SystemTime::now())))
            }
        };
    }
    (None, None)
}

pub fn reload_if_changed(nock: &mut Nock) {
    let Some(path) = nock.bar.config_file.as_ref() else {
        return;
    };
    let Ok(modified) = fs::metadata(path).and_then(|m| m.modified()) else {
        return;
    };
    if nock.bar.config_mtime.is_some_and(|t| modified <= t) {
        return;
    }

    let (root, meta) = load_root("bar");
    let (config_file, config_mtime) = match meta {
        Some((p, t)) => (Some(p), Some(t)),
        None => (None, None),
    };
    nock.bar.root = root.unwrap_or_else(Bar::default_row);
    nock.bar.config_mtime = config_mtime;
    nock.bar.config_file = config_file;
}

fn widget_from_lua(content: &str) -> Result<Widget, Box<dyn std::error::Error>> {
    let lua = Lua::new();
    let table: Table = lua.load(content).eval()?;

    widget_from_table(&table)
}

fn widget_from_table(table: &Table) -> Result<Widget, Box<dyn std::error::Error>> {
    let kind: String = table.get("kind")?;

    let widget = match kind.as_str() {
        "spacer" => Widget::Spacer,
        "label" => Widget::Label {
            text: table.get("text")?,
            id: table.get("id")?,
            on_click: table.get("on_click")?,
            command: table.get("command")?,
            pad: table.get("pad").unwrap_or(0),
        },
        "clock" => Widget::Clock {
            text: table.get("text").unwrap_or_else(|_| String::new()),
            id: table.get("id")?,
            on_click: table.get("on_click")?,
            command: table.get("command")?,
            pad: table.get("pad").unwrap_or(0),
        },
        "date" => Widget::Date {
            text: table.get("text").unwrap_or_else(|_| String::new()),
            id: table.get("id")?,
            on_click: table.get("on_click")?,
            command: table.get("command")?,
            pad: table.get("pad").unwrap_or(0),
        },
        "battery" => Widget::Battery {
            text: table.get("text").unwrap_or_else(|_| String::new()),
            id: table.get("id")?,
            on_click: table.get("on_click")?,
            command: table.get("command")?,
            pad: table.get("pad").unwrap_or(0),
        },
        "volume" => Widget::Volume {
            text: table.get("text").unwrap_or_else(|_| String::new()),
            id: table.get("id")?,
            on_click: table.get("on_click")?,
            command: table.get("command")?,
            pad: table.get("pad").unwrap_or(0),
        },
        "network" => Widget::Network {
            text: table.get("text").unwrap_or_else(|_| String::new()),
            id: table.get("id")?,
            on_click: table.get("on_click")?,
            command: table.get("command")?,
            pad: table.get("pad").unwrap_or(0),
        },
        "row" | "column" => {
            let children: Table = table.get("children")?;
            let mut vec = vec![];
            for i in 1..=children.len()? {
                let child: Table = children.get(i)?;
                vec.push(widget_from_table(&child)?)
            }

            match kind.as_str() {
                "row" => Widget::Row { children: vec },
                _ => Widget::Column { children: vec },
            }
        }
        _ => return Err("unknown kind".into()),
    };
    Ok(widget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget() {
        assert_eq!(
            widget_from_lua(r#"return { kind = "label", text = "nock", id = "brand", on_click = "toggle_theme", pad = "0" }"#).unwrap(),
            Widget::Label {
                text: "nock".to_string(),
                id: Some("brand".to_string()),
                on_click: Some("toggle_theme".to_string()),
                command: None,
                pad:0
            }
        );
    }

    #[test]
    fn command() {
        assert_eq!(
            widget_from_lua(
                r#"return { kind = "label", text = "nock", on_click = "spawn", command = "echo", pad = "0" }"#
            )
            .unwrap(),
            Widget::Label {
                text: "nock".to_string(),
                id: None,
                on_click: Some("spawn".to_string()),
                command: Some("echo".to_string()),
                pad: 0,
            }
        )
    }
}
