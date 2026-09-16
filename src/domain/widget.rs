use std::{cmp::max, iter::zip};

use crate::domain::layout::{Constraints, Measure, Rect, Size};

#[derive(Debug, PartialEq)]
pub enum Widget {
    Label { text: String, id: Option<String> },
    Spacer,
    Clock { text: String, id: Option<String> },
    Row { children: Vec<Widget> },
    Column { children: Vec<Widget> },
}

impl Widget {
    pub fn layout(
        widget: &Widget,
        constraints: &Constraints,
        measurer: &impl Measure,
    ) -> (Size, Vec<(Rect, String, Option<String>)>) {
        match widget {
            Widget::Label { text, id } | Widget::Clock { text, id } => {
                let size = Size::new(measurer.text_width(text).min(constraints.max_width), 16);
                let rects = vec![(
                    Rect::new(0, 0, size.width, size.height),
                    text.clone(),
                    id.clone(),
                )];
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

                    for (r, s, id) in child_rects {
                        all_rects.push((
                            Rect {
                                x: r.x + x,
                                y: r.y,
                                width: r.width,
                                height: r.height,
                            },
                            s,
                            id,
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
                    for (r, s, id) in child_rects {
                        all_rects.push((
                            Rect {
                                x: r.x,
                                y: r.y + y,
                                width: r.width,
                                height: r.height,
                            },
                            s,
                            id,
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
            Widget::Clock { text, id: _ } => *text = s.into(),
            Widget::Row { children } | Widget::Column { children } => {
                for c in children {
                    Widget::set_clock(c, s);
                }
            }
            _ => {}
        }
    }

    pub fn hit(items: &[(Rect, String, Option<String>)], x: u32, y: u32) -> Option<&str> {
        for (r, s, id) in items.iter() {
            if x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height {
                return id.as_deref().or(Some(s.as_str()));
            }
        }

        None
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
                id: None,
            }],
        };
        assert_eq!(
            (
                Size::new(16, 16),
                vec![(Rect::new(0, 0, 16, 16), "ab".into(), None)]
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
                Widget::Label {
                    text: "a".into(),
                    id: None,
                },
                Widget::Row { children: vec![] },
                Widget::Label {
                    text: "ab".into(),
                    id: None,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(24, 16),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into(), None),
                    (Rect::new(8, 0, 16, 16), "ab".into(), None)
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
                id: None,
            }],
        };

        assert_eq!(
            (
                Size::new(16, 16),
                vec![(Rect::new(0, 0, 16, 16), "abcd".into(), None)]
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
                Widget::Label {
                    text: "a".into(),
                    id: None,
                },
                Widget::Spacer,
                Widget::Label {
                    text: "b".into(),
                    id: None,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(100, 16),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into(), None),
                    (Rect::new(92, 0, 8, 16), "b".into(), None)
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
                Widget::Label {
                    text: "a".into(),
                    id: None,
                },
                Widget::Spacer,
                Widget::Clock {
                    text: "b".into(),
                    id: None,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(100, 16),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into(), None),
                    (Rect::new(92, 0, 8, 16), "b".into(), None),
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
                Widget::Label {
                    text: "a".into(),
                    id: None,
                },
                Widget::Label {
                    text: "ab".into(),
                    id: None,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(16, 32),
                vec![
                    (Rect::new(0, 0, 8, 16), "a".into(), None),
                    (Rect::new(0, 16, 16, 16), "ab".into(), None),
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

    #[test]
    fn hit() {
        let measurer = Fake;
        let widget = Widget::Row {
            children: vec![
                Widget::Label {
                    text: "a".into(),
                    id: None,
                },
                Widget::Spacer,
                Widget::Label {
                    text: "b".into(),
                    id: None,
                },
            ],
        };
        let (_, items) = Widget::layout(
            &widget,
            &Constraints {
                max_width: 100,
                max_height: 32,
            },
            &measurer,
        );

        assert_eq!(Widget::hit(&items, 0, 0), Some("a"));
        assert_eq!(Widget::hit(&items, 92, 0), Some("b"));
        assert_eq!(Widget::hit(&items, 50, 0), None);
        assert_eq!(Widget::hit(&items, 0, 20), None);
    }
}
