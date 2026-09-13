use std::cmp::max;

use crate::domain::layout::{Constraints, Rect, Size};

pub enum Widget {
    Label { text: String },
    Row { children: Vec<Widget> },
}

impl Widget {
    pub fn layout(widget: &Widget, constraints: &Constraints) -> (Size, Vec<(Rect, String)>) {
        match widget {
            Widget::Label { text } => {
                let size = Size::new((text.len() as u32 * 8).min(constraints.max_width), 16);
                let rects = vec![(Rect::new(0, 0, size.width, size.height), text.clone())];
                (size, rects)
            }
            Widget::Row { children } => {
                let mut x = 0;
                let mut max_h = 0;
                let mut all_rects = vec![];
                for child in children {
                    let (child_size, child_rects) = Widget::layout(child, constraints);
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_row_any_constraints() {
        let children = Widget::Row { children: vec![] };
        assert_eq!(
            Size::new(0, 0),
            (Widget::layout(
                &children,
                &Constraints {
                    max_width: 0,
                    max_height: 0
                }
            )
            .0)
        );
    }

    #[test]
    fn one_label() {
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
                }
            )
        );
    }

    #[test]
    fn two_labels() {
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
                }
            )
        )
    }

    #[test]
    fn one_label_overflow() {
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
                }
            )
        )
    }
}
