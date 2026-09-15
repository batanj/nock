pub struct Theme {
    pub bg: (f32, f32, f32),
    pub fg: (f32, f32, f32),
}

impl Theme {
    pub fn dark() -> Theme {
        Theme {
            bg: (0.12, 0.12, 0.14),
            fg: (1.0, 1.0, 1.0),
        }
    }
    pub fn light() -> Theme {
        Theme {
            bg: (1.0, 1.0, 1.0),
            fg: (0.0, 0.0, 0.0),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bg() {
        let theme = Theme::dark();

        assert_eq!((0.12, 0.12, 0.14), theme.bg)
    }
}
