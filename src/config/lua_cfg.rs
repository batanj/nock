use mlua::{Lua, Result as LuaResult, Table, Value};

use crate::Widget;

pub fn widget_from_lua(content: &str) -> Result<Widget, Box<dyn std::error::Error>> {
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
        },
        "clock" => Widget::Clock {
            text: table.get("text").unwrap_or_else(|_| String::new()),
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
            },
            Widget::Spacer,
            Widget::Clock { text: "".into() },
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
                text: "nock".to_string()
            }
        );
    }
}
