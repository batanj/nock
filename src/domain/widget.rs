use std::{cmp::max, iter::zip};

use crate::domain::layout::{Constraints, Measure, Rect, Size};

#[derive(Debug, PartialEq)]
pub enum Widget {
    Label {
        text: String,
        id: Option<String>,
        on_click: Option<String>,
        command: Option<String>,
        pad: u32,
    },
    Spacer,
    Clock {
        text: String,
        id: Option<String>,
        on_click: Option<String>,
        command: Option<String>,
        pad: u32,
    },
    Row {
        children: Vec<Widget>,
    },
    Column {
        children: Vec<Widget>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitTarget {
    pub rect: Rect,
    pub text: String,
    pub id: Option<String>,
    pub on_click: Option<String>,
    pub command: Option<String>,
    pub pad: u32,
}

impl HitTarget {
    pub fn new(
        rect: Rect,
        text: String,
        id: Option<String>,
        on_click: Option<String>,
        command: Option<String>,
        pad: u32,
    ) -> Self {
        Self {
            rect,
            text,
            id,
            on_click,
            command,
            pad,
        }
    }
}

impl Widget {
    pub fn layout(
        widget: &Widget,
        constraints: &Constraints,
        measurer: &impl Measure,
    ) -> (Size, Vec<HitTarget>) {
        match widget {
            Widget::Label {
                text,
                id,
                on_click,
                command,
                pad,
            }
            | Widget::Clock {
                text,
                id,
                on_click,
                command,
                pad,
            } => {
                let tw = measurer.text_width(text);
                let width = tw
                    .saturating_add(pad.saturating_mul(2))
                    .min(constraints.max_width);

                let size = Size::new(width, 16);
                let rects = vec![HitTarget::new(
                    Rect::new(0, 0, size.width, size.height),
                    text.clone(),
                    id.clone(),
                    on_click.clone(),
                    command.clone(),
                    *pad,
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

                    for ht in child_rects {
                        let HitTarget {
                            rect,
                            text,
                            id,
                            on_click,
                            command,
                            pad,
                        } = ht;
                        all_rects.push(HitTarget::new(
                            Rect {
                                x: rect.x + x,
                                y: rect.y,
                                width: rect.width,
                                height: rect.height,
                            },
                            text,
                            id,
                            on_click,
                            command,
                            pad,
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
                    for ht in child_rects {
                        let HitTarget {
                            rect,
                            text,
                            id,
                            on_click,
                            command,
                            pad,
                        } = ht;
                        all_rects.push(HitTarget::new(
                            Rect {
                                x: rect.x,
                                y: rect.y + y,
                                width: rect.width,
                                height: rect.height,
                            },
                            text,
                            id,
                            on_click,
                            command,
                            pad,
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
            Widget::Clock {
                text,
                id: _,
                on_click: _,
                command: _,
                pad: _,
            } => *text = s.into(),
            Widget::Row { children } | Widget::Column { children } => {
                for c in children {
                    Widget::set_clock(c, s);
                }
            }
            _ => {}
        }
    }

    pub fn hit(items: &[HitTarget], x: u32, y: u32) -> Option<&HitTarget> {
        for ht in items.iter() {
            let r = &ht.rect;
            if x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height {
                return Some(ht);
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
                on_click: None,
                command: None,
                pad: 0,
            }],
        };
        assert_eq!(
            (
                Size::new(16, 16),
                vec![HitTarget::new(
                    Rect::new(0, 0, 16, 16),
                    "ab".into(),
                    None,
                    None,
                    None,
                    0,
                )]
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
                    on_click: None,
                    command: None,
                    pad: 0,
                },
                Widget::Row { children: vec![] },
                Widget::Label {
                    text: "ab".into(),
                    id: None,
                    on_click: None,
                    command: None,
                    pad: 0,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(24, 16),
                vec![
                    HitTarget::new(Rect::new(0, 0, 8, 16), "a".into(), None, None, None, 0),
                    HitTarget::new(Rect::new(8, 0, 16, 16), "ab".into(), None, None, None, 0)
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
                on_click: None,
                command: None,
                pad: 0,
            }],
        };

        assert_eq!(
            (
                Size::new(16, 16),
                vec![HitTarget::new(
                    Rect::new(0, 0, 16, 16),
                    "abcd".into(),
                    None,
                    None,
                    None,
                    0,
                )]
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
                    on_click: None,
                    command: None,
                    pad: 0,
                },
                Widget::Spacer,
                Widget::Label {
                    text: "b".into(),
                    id: None,
                    on_click: None,
                    command: None,
                    pad: 0,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(100, 16),
                vec![
                    HitTarget::new(Rect::new(0, 0, 8, 16), "a".into(), None, None, None, 0),
                    HitTarget::new(Rect::new(92, 0, 8, 16), "b".into(), None, None, None, 0)
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
                    on_click: None,
                    command: None,
                    pad: 0,
                },
                Widget::Spacer,
                Widget::Clock {
                    text: "b".into(),
                    id: None,
                    on_click: None,
                    command: None,
                    pad: 0,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(100, 16),
                vec![
                    HitTarget::new(Rect::new(0, 0, 8, 16), "a".into(), None, None, None, 0),
                    HitTarget::new(Rect::new(92, 0, 8, 16), "b".into(), None, None, None, 0),
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
                    on_click: None,
                    command: None,
                    pad: 0,
                },
                Widget::Label {
                    text: "ab".into(),
                    id: None,
                    on_click: None,
                    command: None,
                    pad: 0,
                },
            ],
        };
        assert_eq!(
            (
                Size::new(16, 32),
                vec![
                    HitTarget::new(Rect::new(0, 0, 8, 16), "a".into(), None, None, None, 0),
                    HitTarget::new(Rect::new(0, 16, 16, 16), "ab".into(), None, None, None, 0),
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
                    id: Some("brand".into()),
                    on_click: Some("toggle_theme".into()),
                    command: None,
                    pad: 0,
                },
                Widget::Spacer,
                Widget::Label {
                    text: "b".into(),
                    id: None,
                    on_click: None,
                    command: None,
                    pad: 8,
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

        assert_eq!(
            Widget::hit(&items, 0, 0),
            Some(&HitTarget::new(
                Rect::new(0, 0, 8, 16),
                "a".to_string(),
                Some("brand".to_string()),
                Some("toggle_theme".to_string()),
                None,
                0,
            ))
        );
        assert_eq!(
            Widget::hit(&items, 92, 0),
            Some(&HitTarget::new(
                Rect::new(92, 0, 8, 16),
                "b".to_string(),
                None,
                None,
                None,
                8
            ))
        );
        assert_eq!(Widget::hit(&items, 50, 0), None);
        assert_eq!(Widget::hit(&items, 0, 20), None);
    }
}
