use crate::visual::Family;
// Local, transient policy. No Windows APIs, renderer or activity persistence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopKind {
    WindowFocus,
    MouseClick,
    MouseDragStart,
    MouseDragMove,
    MouseDragEnd,
}
#[derive(Clone, Copy, Debug)]
pub struct DesktopEvent {
    pub kind: DesktopKind,
    pub source: usize,
    pub bounds: Bounds,
    pub work_area: Bounds,
    pub point: Point,
    pub own: bool,
    pub suppressed: bool,
    pub dpi: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Intensity {
    #[default]
    Subtle,
    Experimental,
    Anomalous,
}
pub struct ReactionConfig {
    pub enabled: bool,
    pub events: u8,
    pub cooldown_ms: u64,
    pub simultaneous: usize,
    pub suppress_fullscreen: bool,
    pub intensity: Intensity,
    pub focus_family: Family,
    pub click_family: Family,
    pub drag_family: Family,
}
impl Default for ReactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            events: 7,
            cooldown_ms: 700,
            simultaneous: 1,
            suppress_fullscreen: true,
            intensity: Intensity::Experimental,
            focus_family: Family::StackV2,
            click_family: Family::ClickPulse,
            drag_family: Family::DragConnector,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Reaction {
    pub event: DesktopEvent,
    pub start: u64,
    pub duration_ms: u64,
    pub intensity: Intensity,
    pub family: Family,
}
#[derive(Clone, Copy, Debug)]
pub enum ReactionDecision {
    Start(Reaction),
    Cancel,
    Unchanged,
}
#[derive(Default)]
pub struct ReactionEngine {
    pub config: ReactionConfig,
    pub active: Vec<Reaction>,
    pub received: u64,
    pub emitted: u64,
    last_emitted: Option<u64>,
    last_focus: Option<usize>,
    last_click: Option<u64>,
    last_drag: Option<u64>,
    accepted_drag: Option<usize>,
    drag_updated: Option<u64>,
}
impl ReactionEngine {
    pub fn accept(&mut self, event: DesktopEvent, now: u64) -> bool {
        matches!(self.decide(event, now), ReactionDecision::Start(_))
    }

    pub fn decide(&mut self, event: DesktopEvent, now: u64) -> ReactionDecision {
        self.received += 1;
        self.prune(now);
        if event.kind == DesktopKind::MouseDragEnd {
            self.accepted_drag = None;
            if self.active.iter().any(|r| {
                matches!(
                    r.event.kind,
                    DesktopKind::MouseDragStart | DesktopKind::MouseDragMove
                )
            }) {
                self.clear();
                return ReactionDecision::Cancel;
            }
            return ReactionDecision::Unchanged;
        }
        if event.kind != DesktopKind::WindowFocus {
            return self.pointer(event, now);
        }
        if event.own {
            return ReactionDecision::Unchanged;
        }
        // Activating the dragged application's window must not replace the
        // connector with a focus effect halfway through the same gesture.
        if self.accepted_drag.is_some()
            && self.active.iter().any(|r| matches!(
                r.event.kind, DesktopKind::MouseDragStart | DesktopKind::MouseDragMove
            ))
        {
            self.last_focus = Some(event.source);
            return ReactionDecision::Unchanged;
        }
        if !self.config.enabled || self.config.simultaneous == 0 || self.config.events & 1 == 0 {
            self.clear();
            return ReactionDecision::Cancel;
        }
        if event.suppressed && self.config.suppress_fullscreen {
            self.active.clear();
            self.last_focus = Some(event.source);
            return ReactionDecision::Cancel;
        }
        if self.last_focus == Some(event.source) {
            return ReactionDecision::Unchanged;
        }
        self.last_focus = Some(event.source);
        // The foreground already changed: never leave an obsolete effect on
        // the previous window, even when the new event is rate-limited.
        self.active.clear();
        if self
            .last_emitted
            .is_some_and(|last| now.saturating_sub(last) < self.config.cooldown_ms)
        {
            return ReactionDecision::Cancel;
        }
        let reaction = Reaction {
            event,
            start: now,
            duration_ms: 850,
            intensity: self.config.intensity,
            family: Family::StackV2,
        };
        self.active.push(reaction);
        self.last_emitted = Some(now);
        self.emitted += 1;
        ReactionDecision::Start(reaction)
    }
    fn pointer(&mut self, event: DesktopEvent, now: u64) -> ReactionDecision {
        if event.own {
            return ReactionDecision::Unchanged;
        }
        let click = event.kind == DesktopKind::MouseClick;
        let bit = if click { 2 } else { 4 };
        if self.config.events & bit == 0 {
            if !click {
                self.accepted_drag = None;
                if self.active.iter().any(|r| {
                    matches!(
                        r.event.kind,
                        DesktopKind::MouseDragStart | DesktopKind::MouseDragMove
                    )
                }) {
                    self.clear();
                    return ReactionDecision::Cancel;
                }
            }
            return ReactionDecision::Unchanged;
        }
        if !self.config.enabled
            || self.config.simultaneous == 0
            || self.config.events & bit == 0
            || (event.suppressed && self.config.suppress_fullscreen)
        {
            if !click {
                self.accepted_drag = None;
                self.clear();
                return ReactionDecision::Cancel;
            }
            return ReactionDecision::Unchanged;
        }
        let updating = event.kind == DesktopKind::MouseDragMove;
        if updating && self.accepted_drag != Some(event.source) {
            return ReactionDecision::Unchanged;
        }
        let last = if click {
            self.last_click
        } else {
            self.last_drag
        };
        let cooldown = if click { 250 } else { 350 };
        if !updating && last.is_some_and(|t| now.saturating_sub(t) < cooldown) {
            return ReactionDecision::Unchanged;
        }
        // Single shared slot is an intentional hard resource cap. A microclick
        // cannot replace a more important window manifestation.
        if click && !self.active.is_empty() {
            return ReactionDecision::Unchanged;
        }
        let reaction = Reaction {
            event,
            start: now,
            duration_ms: if click { 260 } else { 1500 },
            intensity: self.config.intensity,
            family: if click {
                Family::ClickPulse
            } else {
                Family::DragConnector
            },
        };
        if !click && !updating {
            self.accepted_drag = Some(event.source);
        }
        self.active.clear();
        self.active.push(reaction);
        if click {
            self.last_click = Some(now);
        } else if !updating {
            self.last_drag = Some(now);
        }
        if !click {
            self.drag_updated = Some(now);
        }
        self.emitted += 1;
        ReactionDecision::Start(reaction)
    }

    pub fn prune(&mut self, now: u64) {
        if self
            .drag_updated
            .is_some_and(|t| now.saturating_sub(t) >= 60_000)
        {
            self.accepted_drag = None;
        }
        self.active
            .retain(|r| now.saturating_sub(r.start) < r.duration_ms);
    }
    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub fn forget_focus(&mut self) {
        self.clear();
        self.last_focus = None;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        self.clear();
        self.last_focus = None;
        self.last_emitted = None;
        self.last_click = None;
        self.last_drag = None;
        self.accepted_drag = None;
        self.drag_updated = None;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn event(source: usize) -> DesktopEvent {
        DesktopEvent {
            kind: DesktopKind::WindowFocus,
            source,
            bounds: Bounds::default(),
            work_area: Bounds::default(),
            point: Point::default(),
            own: false,
            suppressed: false,
            dpi: 96,
        }
    }
    #[test]
    fn focus_coalesces_and_respects_cooldown_and_privacy_filters() {
        let mut engine = ReactionEngine::default();
        assert!(engine.accept(event(1), 0));
        assert!(!engine.accept(event(1), 800));
        assert!(!engine.accept(
            DesktopEvent {
                own: true,
                ..event(2)
            },
            900
        ));
        assert!(engine.accept(event(3), 1000));
        assert!(!engine.accept(event(4), 1100));
        assert!(engine.active.is_empty());
        assert!(!engine.accept(
            DesktopEvent {
                suppressed: true,
                ..event(5)
            },
            2000
        ));
        assert!(engine.active.is_empty());
        engine.config.enabled = false;
        assert!(!engine.accept(event(6), 3000));
    }

    #[test]
    fn duplicate_does_not_restart_lifetime_or_extend_cooldown() {
        let mut engine = ReactionEngine::default();
        assert!(engine.accept(event(1), 0));
        assert!(!engine.accept(event(1), 600));
        assert_eq!(engine.active[0].start, 0);
        assert!(engine.accept(event(2), 700));
        assert_eq!(engine.active.len(), 1);
        engine.prune(1550);
        assert!(engine.active.is_empty());
    }

    #[test]
    fn own_events_do_not_consume_cooldown_and_disabling_rearms_cleanly() {
        let mut engine = ReactionEngine::default();
        assert!(!engine.accept(
            DesktopEvent {
                own: true,
                ..event(1)
            },
            0
        ));
        assert!(engine.accept(event(2), 1));
        engine.set_enabled(false);
        assert!(engine.active.is_empty());
        assert!(!engine.accept(event(3), 2));
        engine.set_enabled(true);
        assert!(engine.accept(event(2), 3));
    }

    #[test]
    fn disabling_focus_events_cancels_and_never_queues_a_late_reaction() {
        let mut engine = ReactionEngine::default();
        assert!(engine.accept(event(1), 0));
        assert!(matches!(
            engine.decide(event(2), 100),
            ReactionDecision::Cancel
        ));
        engine.prune(10000);
        assert!(engine.active.is_empty());
        engine.config.events = 0;
        assert!(!engine.accept(event(3), 11000));
    }
    fn pointer(kind: DesktopKind, source: usize) -> DesktopEvent {
        DesktopEvent {
            kind,
            ..event(source)
        }
    }
    #[test]
    fn click_budget_priority_and_finite_lifetime() {
        let mut e = ReactionEngine::default();
        assert!(e.accept(pointer(DesktopKind::MouseClick, 1), 0));
        assert_eq!(e.active[0].family, Family::ClickPulse);
        e.prune(260);
        assert!(e.active.is_empty());
        assert!(e.accept(pointer(DesktopKind::MouseClick, 1), 300));
        assert!(e.accept(event(2), 400));
        assert!(!e.accept(pointer(DesktopKind::MouseClick, 1), 600));
        assert_eq!(e.active[0].event.kind, DesktopKind::WindowFocus);
        assert_eq!(e.active.len(), 1);
    }
    #[test]
    fn dragging_resumes_after_inactivity_and_release_over_navi_always_ends() {
        let mut e = ReactionEngine::default();
        assert!(!e.accept(pointer(DesktopKind::MouseDragMove, 1), 0));
        assert!(e.accept(pointer(DesktopKind::MouseDragStart, 1), 1));
        e.prune(2000);
        assert!(e.active.is_empty());
        assert!(e.accept(pointer(DesktopKind::MouseDragMove, 1), 2001));
        assert!(matches!(
            e.decide(
                DesktopEvent {
                    own: true,
                    ..pointer(DesktopKind::MouseDragEnd, 1)
                },
                2002
            ),
            ReactionDecision::Cancel
        ));
        assert!(!e.accept(pointer(DesktopKind::MouseDragMove, 1), 2003));
        assert!(e.accept(pointer(DesktopKind::MouseDragStart, 1), 2004));
        e.decide(pointer(DesktopKind::MouseDragEnd, 1), 2005);
        assert!(!e.accept(pointer(DesktopKind::MouseDragStart, 1), 2006));
    }
    #[test]
    fn independent_masks_fullscreen_and_zero_budget_suppress_pointer() {
        let mut e = ReactionEngine::default();
        e.config.events = 1;
        assert!(!e.accept(pointer(DesktopKind::MouseClick, 1), 0));
        assert!(!e.accept(pointer(DesktopKind::MouseDragStart, 1), 1));
        assert!(e.accept(event(1), 2));
        e.set_enabled(true);
        e.config.events = 7;
        assert!(!e.accept(
            DesktopEvent {
                suppressed: true,
                ..pointer(DesktopKind::MouseClick, 1)
            },
            3
        ));
        assert!(!e.accept(
            DesktopEvent {
                own: true,
                ..pointer(DesktopKind::MouseDragStart, 1)
            },
            4
        ));
        e.config.simultaneous = 0;
        assert!(!e.accept(pointer(DesktopKind::MouseClick, 1), 5));
        assert!(!e.accept(event(2), 6));
        assert!(e.active.is_empty());
    }
    #[test]
    fn obsolete_family_configuration_cannot_change_event_roles() {
        let mut e = ReactionEngine::default();
        e.config.focus_family = Family::FracturedHalo;
        assert!(e.accept(event(1), 0));
        assert_eq!(e.active[0].family, Family::StackV2);
        assert_eq!(e.active[0].intensity, Intensity::Experimental);
        e.clear();
        e.config.click_family = Family::Sutura;
        assert!(e.accept(pointer(DesktopKind::MouseClick, 1), 1000));
        assert_eq!(e.active[0].family, Family::ClickPulse);
        e.config.drag_family = Family::StackV2;
        assert!(e.accept(pointer(DesktopKind::MouseDragStart, 1), 2000));
        assert_eq!(e.active[0].family, Family::DragConnector);
    }

    #[test]
    fn continuous_drag_does_not_expire_at_sixty_seconds() {
        let mut e = ReactionEngine::default();
        assert!(e.accept(pointer(DesktopKind::MouseDragStart, 1), 0));
        for now in [30_000, 60_000, 90_000, 120_000] {
            assert!(e.accept(pointer(DesktopKind::MouseDragMove, 1), now));
        }
        e.prune(180_000);
        assert!(!e.accept(pointer(DesktopKind::MouseDragMove, 1), 180_001));
    }
    #[test]
    fn disabled_drag_event_cannot_cancel_an_unrelated_focus_manifestation() {
        let mut e = ReactionEngine::default();
        e.config.events = 3;
        assert!(e.accept(event(1), 0));
        assert!(matches!(
            e.decide(pointer(DesktopKind::MouseDragStart, 1), 1),
            ReactionDecision::Unchanged
        ));
        assert_eq!(e.active[0].event.kind, DesktopKind::WindowFocus);
    }
}
