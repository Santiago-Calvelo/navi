//! A bounded pointer gesture, with no content inspection or activity history.
use crate::reactions::Point;

pub const MOVE_INTERVAL_MS: u64 = 33;
const MAX_GESTURE_MS: u64 = 60_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorKind {
    Click,
    DragStart,
    DragMove,
    DragEnd,
}

#[derive(Clone, Copy, Debug)]
pub struct CursorSample {
    pub kind: CursorKind,
    pub point: Point,
    pub origin: Point,
    pub source: usize,
    pub own: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct PointerInput {
    pub point: Point,
    pub down: bool,
    pub up: bool,
    pub held: bool,
    pub own: bool,
    pub source: usize,
}

#[derive(Clone, Copy, Debug)]
struct Press {
    origin: Point,
    source: usize,
    own: bool,
    started: u64,
    last_move: u64,
    last_point: Point,
    dragging: bool,
}

pub struct CursorGesture {
    threshold_x: i32,
    threshold_y: i32,
    press: Option<Press>,
}

impl CursorGesture {
    pub fn new(threshold_x: i32, threshold_y: i32) -> Self {
        Self {
            threshold_x: threshold_x.max(1),
            threshold_y: threshold_y.max(1),
            press: None,
        }
    }

    pub fn pressed(&self) -> bool {
        self.press.is_some()
    }

    pub fn reset(&mut self) {
        self.press = None;
    }

    pub fn feed(&mut self, input: PointerInput, now: u64) -> Option<CursorSample> {
        if input.down && self.press.is_none() {
            self.press = Some(Press {
                origin: input.point,
                source: input.source,
                own: input.own,
                started: now,
                last_move: now,
                last_point: input.point,
                dragging: false,
            });
        }
        let mut press = self.press?;
        let sample = move |kind| CursorSample {
            kind,
            point: input.point,
            origin: press.origin,
            source: press.source,
            own: press.own || input.own,
        };
        // Up wins over the asynchronous held bit: a rapid click can already be
        // released by the time its queued raw-input packets are processed.
        if input.up {
            self.press = None;
            if press.own {
                return None;
            }
            return Some(sample(if press.dragging {
                CursorKind::DragEnd
            } else if input.point.x.abs_diff(press.origin.x) >= self.threshold_x as u32
                || input.point.y.abs_diff(press.origin.y) >= self.threshold_y as u32
            {
                // A coalesced press/release can cross the threshold without a
                // move packet. It must never be mistaken for a click.
                CursorKind::DragEnd
            } else {
                CursorKind::Click
            }));
        }
        if (!input.held && !input.down) || now.saturating_sub(press.started) >= MAX_GESTURE_MS {
            self.press = None;
            return (!press.own && press.dragging).then(|| sample(CursorKind::DragEnd));
        }
        if press.own {
            return None;
        }
        if !press.dragging
            && (input.point.x.abs_diff(press.origin.x) >= self.threshold_x as u32
                || input.point.y.abs_diff(press.origin.y) >= self.threshold_y as u32)
        {
            press.dragging = true;
            press.last_move = now;
            press.last_point = input.point;
            self.press = Some(press);
            return Some(sample(CursorKind::DragStart));
        }
        if press.dragging
            && input.point != press.last_point
            && now.saturating_sub(press.last_move) >= MOVE_INTERVAL_MS
        {
            press.last_move = now;
            press.last_point = input.point;
            self.press = Some(press);
            return Some(sample(CursorKind::DragMove));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(x: i32, y: i32) -> PointerInput {
        PointerInput {
            point: Point { x, y },
            down: false,
            up: false,
            held: true,
            own: false,
            source: 12,
        }
    }

    #[test]
    fn click_is_only_emitted_on_release_and_unmatched_up_is_ignored() {
        let mut gesture = CursorGesture::new(4, 4);
        assert!(gesture
            .feed(
                PointerInput {
                    up: true,
                    ..input(0, 0)
                },
                0
            )
            .is_none());
        assert!(gesture
            .feed(
                PointerInput {
                    down: true,
                    ..input(0, 0)
                },
                1
            )
            .is_none());
        assert!(gesture.feed(input(3, 3), 2).is_none());
        assert_eq!(
            gesture
                .feed(
                    PointerInput {
                        up: true,
                        held: false,
                        ..input(3, 3)
                    },
                    3
                )
                .unwrap()
                .kind,
            CursorKind::Click
        );
        assert!(!gesture.pressed());
    }

    #[test]
    fn drag_threshold_move_budget_and_immediate_end() {
        let mut gesture = CursorGesture::new(4, 6);
        gesture.feed(
            PointerInput {
                down: true,
                ..input(-100, 0)
            },
            0,
        );
        let start = gesture.feed(input(-96, 0), 10).unwrap();
        assert_eq!(start.kind, CursorKind::DragStart);
        assert_eq!(start.origin.x, -100);
        assert!(gesture.feed(input(-90, 0), 42).is_none());
        assert_eq!(
            gesture.feed(input(-89, 0), 43).unwrap().kind,
            CursorKind::DragMove
        );
        assert_eq!(
            gesture
                .feed(
                    PointerInput {
                        up: true,
                        ..input(-89, 0)
                    },
                    44
                )
                .unwrap()
                .kind,
            CursorKind::DragEnd
        );
        assert!(!gesture.pressed());
    }

    #[test]
    fn own_press_never_generates_a_click_or_drag() {
        let mut gesture = CursorGesture::new(4, 4);
        gesture.feed(
            PointerInput {
                down: true,
                own: true,
                ..input(0, 0)
            },
            0,
        );
        assert!(gesture.feed(input(200, 200), 100).is_none());
        assert!(gesture
            .feed(
                PointerInput {
                    up: true,
                    ..input(200, 200)
                },
                200
            )
            .is_none());
        assert!(!gesture.pressed());
    }

    #[test]
    fn own_endpoint_is_marked_but_still_ends_an_external_drag() {
        let mut gesture = CursorGesture::new(4, 4);
        gesture.feed(
            PointerInput {
                down: true,
                ..input(0, 0)
            },
            0,
        );
        gesture.feed(input(10, 0), 1);
        let end = gesture
            .feed(
                PointerInput {
                    up: true,
                    own: true,
                    ..input(20, 0)
                },
                2,
            )
            .unwrap();
        assert_eq!(end.kind, CursorKind::DragEnd);
        assert!(end.own);
    }

    #[test]
    fn lost_release_and_watchdog_clear_transient_state() {
        let mut gesture = CursorGesture::new(4, 4);
        gesture.feed(
            PointerInput {
                down: true,
                ..input(0, 0)
            },
            0,
        );
        gesture.feed(input(10, 0), 1);
        assert_eq!(
            gesture
                .feed(
                    PointerInput {
                        held: false,
                        ..input(11, 0)
                    },
                    2
                )
                .unwrap()
                .kind,
            CursorKind::DragEnd
        );
        gesture.feed(
            PointerInput {
                down: true,
                ..input(0, 0)
            },
            3,
        );
        assert!(gesture.feed(input(0, 0), 60_003).is_none());
        assert!(!gesture.pressed());
        gesture.reset();
    }

    #[test]
    fn coalesced_release_that_moved_is_not_a_click() {
        let mut gesture = CursorGesture::new(4, 4);
        gesture.feed(
            PointerInput {
                down: true,
                ..input(0, 0)
            },
            0,
        );
        assert_eq!(
            gesture
                .feed(
                    PointerInput {
                        up: true,
                        ..input(4, 0)
                    },
                    1
                )
                .unwrap()
                .kind,
            CursorKind::DragEnd
        );
    }
}
