return {
    kind = "row",
    children = {
        { kind = "label",   text = "nock from lua", id = "brand",              on_click = "toggle_theme",                              pad = 8 },
        { kind = "spacer" },
        { kind = "clock",   id = "clock",           on_click = "toggle_clock", pad = 8 },
        { kind = "date",    id = "date",            on_click = "toggle_panel", pad = 8 },
        { kind = "spacer" },
        { kind = "label",   text = "Kitty",         id = "kitty",              on_click = "spawn",                                     command = "kitty --single-instance", pad = 8 },
        { kind = "battery", id = "battery",         pad = 8 },
        { kind = "volume",  id = "volume",          on_click = "spawn",        command = "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle", pad = 8 },
        { kind = "network", text = "Network",       id = "network",            pad = 8 },
    }
}
