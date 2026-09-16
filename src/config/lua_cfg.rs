use std::{env, path::PathBuf};

use mlua::{Lua, Table};

use crate::Widget;

fn config_path() -> Option<PathBuf> {
    config_candidates().into_iter().find(|p| p.is_file())
}

fn config_candidates() -> Vec<PathBuf> {
    let mut v = vec![];
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        v.push(PathBuf::from(xdg).join("nock").join("nock.lua"));
    }
    if let Ok(home) = env::var("HOME") {
        v.push(
            PathBuf::from(home)
                .join(".config")
                .join("nock")
                .join("nock.lua"),
        );
    }
    v.push(PathBuf::from("nock.lua"));
    v
}

pub fn load_root() -> Widget {
    let Some(path) = config_path() else {
        return default_row();
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => widget_from_lua(&s).unwrap_or_else(|e| {
            eprintln!("nock: {path:?}: {e}");
            default_row()
        }),
        Err(e) => {
            eprintln!("nock: {path:?}: {e}");
            default_row()
        }
    }
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
        },
        "clock" => Widget::Clock {
            text: table.get("text").unwrap_or_else(|_| String::new()),
            id: table.get("id")?,
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

pub fn default_row() -> Widget {
    Widget::Row {
        children: vec![
            Widget::Label {
                text: "nock".into(),
                id: Some("brand".into()),
            },
            Widget::Spacer,
            Widget::Clock {
                text: "".into(),
                id: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget() {
        assert_eq!(
            widget_from_lua(r#"return { kind = "label", text = "nock" }"#).unwrap(),
            Widget::Label {
                text: "nock".to_string(),
                id: None,
            }
        );
    }
}
