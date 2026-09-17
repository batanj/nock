return {
    kind = "row",
    children = {
        { kind = "label", text = "nock from lua", id = "brand", on_click = "toggle_theme" },
        { kind = "spacer" },
        { kind = "clock" },
        { kind = "label", text = "Kitty",         id = "kitty", on_click = "spawn",       command = "kitty" },
    }
}
