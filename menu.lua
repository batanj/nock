return {
    kind = "column",
    children = {
        { kind = "label", text = "Terminal", id = "terminal",           on_click = "spawn", command = "kitty", pad = 8 },
        { kind = "label", text = "Theme",    on_click = "toggle_theme", pad = 8 },
        { kind = "label", text = "Close",    on_click = "toggle_menu",  pad = 8 },
    }
}
