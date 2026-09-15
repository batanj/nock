use std::{cmp::max, iter::zip};

use crate::domain::layout::{Constraints, Measure, Rect, Size};

#[derive(Debug)]
pub enum Widget {
    Label { text: String },
    Spacer,
    Clock { text: String },
    Row { children: Vec<Widget> },
    Column { children: Vec<Widget> },
}

impl Widget {
    pub fn layout(
        widget: &Widget,
        constraints: &Constraints,
        measurer: &impl Measure,
    ) -> (Size, Vec<(Rect, String)>) {
        match widget {
            Widget::Label { text } | Widget::Clock { text } => {
                let size = Size::new(measurer.text_width(text).min(constraints.max_width), 16);
                let rects = vec![(Rect::new(0, 0, size.width, size.height), text.clone())];
                (size, rects)
            }
            Widget::Spacer => {
                let size = Size::new(0, 16);
                let rects = vec![];
                (size, rects)
            }
            Widget::Row { children } => {
                let mut results = vec![];
                let mut used = 0;

                for child in children {
                    let laid = Widget::layout(child, constraints, measurer);
                    match child {
                        Widget::Spacer => {}
                        _ => used += laid.0.width,
                    }
                    results.push(laid);
                }

                let leftover = constraints.max_width.saturating_sub(used);

                let mut x = 0;
                let mut max_h = 0;
                let mut all_rects = vec![];
                for (child, (child_size, child_rects)) in zip(children, results) {
                    if let Widget::Spacer = child {
                        x += leftover;
                    }

                    for (r, s) in child_rects {
                        all_rects.push((
                            Rect {
                                x: r.x + x,
                                y: r.y,
                                width: r.width,
                                height: r.height,
                            },
                            s,
                        ));
                    }
                    x += child_size.width;
                    max_h = max(max_h, child_size.height);
                }
                (
                    Size {
                        width: x,
                        height: max_h,
                    },
                    all_rects,
                )
            }
            Widget::Column { children } => {
                let mut y = 0;
                let mut max_w = 0;
                let mut all_rects = vec![];
                for child in children {
                    let (child_size, child_rects) = Widget::layout(child, constraints, measurer);
                    for (r, s) in child_rects {
                        all_rects.push((
                            Rect {
                                x: r.x,
                                y: r.y + y,
                                width: r.width,
                                height: r.height,
                            },
                            s,
                        ));
                    }
                    y += child_size.height;
                    max_w = max(max_w, child_size.width);
                }
                (
                    Size {
                        width: max_w,
                        height: y,
                    },
                    all_rects,
                )
            }
        }
    }

    pub fn set_clock(widget: &mut Widget, s: &str) {
        match widget {
            Widget::Clock { text } => *text = s.into(),
            Widget::Row { children } => {
                for c in children {
                    Widget::set_clock(c, s);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake;
    impl Measure for Fake {
        fn text_width(&self, text: &str) -> u32 {
            text.len() as u32 * 8
        }
    }

    #[test]
    fn empty_row_any_constraints() {
        let measurer = Fake;
        let children = Widget::Row { children: vec![] };
        assert_eq!(
            Size::new(0, 0),
            (Widget::layout(
                &children,
                &Constraints {
                    max_width: 0,
                    max_height: 0
                },
                &measurer
            )
            .0)
        );
    }

    #[test]
    fn one_label() {
        let measurer = Fake;
        let widget = Widget::Row {
            children: vec![Widget::Label {
                text: String::from("ab"),
            }],
        };
        assert_eq!(
            (
                Size::new(16, 16),
                vec![(Rect::new(0, 0, 16, 16), "ab".into())]
            ),
            Widget::layout(
                &widget,
                &Constraints {
                    max_width: 1000,
                    max_height: 32
                },
                &measurer
            )
        );
    }

    #[test]
    fn two_labels() {
        let measurer = Fake;
        let widget = Widget::Row {
            children: vec![
                Widget::Label { text: "a".into() },
                Widget::Row { children: vec![] },
                Widget::Label { text: "ab".into() },
            ],
        };
        assert_eq!(
            (
                Size::new(24, 16),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into()),
                    (Rect::new(8, 0, 16, 16), "ab".into())
                ]
            ),
            Widget::layout(
                &widget,
                &Constraints {
                    max_width: 1000,
                    max_height: 32
                },
                &measurer
            )
        )
    }

    #[test]
    fn one_label_overflow() {
        let measurer = Fake;
        let widget = Widget::Row {
            children: vec![Widget::Label {
                text: "abcd".into(),
            }],
        };

        assert_eq!(
            (
                Size::new(16, 16),
                vec![(Rect::new(0, 0, 16, 16), "abcd".into())]
            ),
            Widget::layout(
                &widget,
                &Constraints {
                    max_width: 16,
                    max_height: 16
                },
                &measurer
            )
        )
    }

    #[test]
    fn leftover() {
        let measurer = Fake;
        let widget = Widget::Row {
            children: vec![
                Widget::Label { text: "a".into() },
                Widget::Spacer,
                Widget::Label { text: "b".into() },
            ],
        };
        assert_eq!(
            (
                Size::new(100, 16),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into()),
                    (Rect::new(92, 0, 8, 16), "b".into())
                ]
            ),
            Widget::layout(
                &widget,
                &Constraints {
                    max_width: 100,
                    max_height: 16
                },
                &measurer
            )
        );
    }

    #[test]
    fn clock() {
        let measurer = Fake;
        let widget = Widget::Row {
            children: vec![
                Widget::Label { text: "a".into() },
                Widget::Spacer,
                Widget::Clock { text: "b".into() },
            ],
        };
        assert_eq!(
            (
                Size::new(100, 16),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into()),
                    (Rect::new(92, 0, 8, 16), "b".into()),
                ]
            ),
            Widget::layout(
                &widget,
                &Constraints {
                    max_width: 100,
                    max_height: 16
                },
                &measurer
            )
        );
    }

    #[test]
    fn column() {
        let measurer = Fake;
        let widget = Widget::Column {
            children: vec![
                Widget::Label { text: "a".into() },
                Widget::Label { text: "ab".into() },
            ],
        };
        assert_eq!(
            (
                Size::new(16, 32),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into()),
                    (Rect::new(0, 16, 16, 16), "ab".into()),
                ]
            ),
            Widget::layout(
                &widget,
                &Constraints {
                    max_width: 100,
                    max_height: 32
                },
                &measurer
            )
        );
    }
}
