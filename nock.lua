return {
    kind = "row",
    children = {
        { kind = "label", text = "nock from lua", id = "brand",              on_click = "toggle_theme", pad = 8 },
        { kind = "spacer" },
        { kind = "clock", id = "clock",           on_click = "toggle_clock", pad = 8 },
        { kind = "spacer" },
        { kind = "label", text = "Kitty",         id = "kitty",              on_click = "spawn",        command = "kitty --single-instance", pad = 8 },
    }
}
