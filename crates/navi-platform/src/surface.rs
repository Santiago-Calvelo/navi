//! Surface policy only: no renderer, HWND, COM or Windows calls.
use navi_state::reactions::{DesktopKind, Reaction};
use navi_state::visual::{Family, Placement, VisualStyle};
use navi_state::{Phase, Session};
use navi_ui::{surface_size, SurfaceKind, SurfaceSize};

pub const RESPONSE_MS: u64 = 4_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Contextual anchors can later be resolved from a window or desktop element by
/// the platform adapter. The manager only consumes their current screen bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceAnchor {
    #[default]
    WorkArea,
    ContextRect(Rect),
}

#[derive(Clone, Copy, Debug)]
pub struct SurfacePlan {
    pub bounds: Rect,
    pub size: SurfaceSize,
    pub kind: SurfaceKind,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct FocusEffectPlan {
    pub bounds: Rect,
    pub dpi: u32,
    pub size: SurfaceSize,
    pub progress: f32,
    pub remaining_ms: u32,
    pub family: Family,
    pub style: VisualStyle,
    pub points: [(f32, f32); 2],
    pub drag: bool,
}

pub struct SurfaceManager {
    pub kind: SurfaceKind,
    pub visible: bool,
    pub anchor: SurfaceAnchor,
    pub console_position: Option<(i32, i32)>,
    pub effect_anchor: SurfaceAnchor,
    focus_effect: Option<Reaction>,
    pub placement: Placement,
    pub seed: Option<u64>,
    deadline: Option<u64>,
    phase: Phase,
    protected: bool,
}

impl Default for SurfaceManager {
    fn default() -> Self {
        Self {
            kind: SurfaceKind::Beacon,
            visible: true,
            anchor: SurfaceAnchor::WorkArea,
            console_position: None,
            effect_anchor: SurfaceAnchor::WorkArea,
            focus_effect: None,
            placement: Placement::WindowAnchor,
            seed: None,
            deadline: None,
            phase: Phase::Idle,
            protected: false,
        }
    }
}

impl SurfaceManager {
    pub fn begin_focus_effect(&mut self, reaction: Reaction) -> bool {
        let event = reaction.event;
        let valid = event.dpi > 0
            && event.work_area.right > event.work_area.left
            && event.work_area.bottom > event.work_area.top
            && (event.kind != DesktopKind::WindowFocus
                || (event.bounds.right.min(event.work_area.right)
                    > event.bounds.left.max(event.work_area.left)
                    && event.bounds.bottom.min(event.work_area.bottom)
                        > event.bounds.top.max(event.work_area.top)));
        self.focus_effect = valid.then_some(reaction);
        valid
    }

    pub fn clear_focus_effect(&mut self) -> bool {
        self.focus_effect.take().is_some()
    }

    pub fn expire_focus_effect(&mut self, now: u64) {
        if self
            .focus_effect
            .is_some_and(|r| now.saturating_sub(r.start) >= r.duration_ms)
        {
            self.focus_effect = None;
        }
    }

    pub fn focus_effect_plan(&self, now: u64) -> Option<FocusEffectPlan> {
        let reaction = self.focus_effect?;
        let elapsed = now.saturating_sub(reaction.start);
        if elapsed >= reaction.duration_ms {
            return None;
        }
        let event = reaction.event;
        let scale = event.dpi as f32 / 96.0;
        let seed = self.seed.unwrap_or(
            reaction
                .start
                .wrapping_mul(6364136223846793005)
                .wrapping_add(event.source as u64),
        );
        let pointer = matches!(reaction.family, Family::ClickPulse | Family::DragConnector);
        let (bounds, _) = if pointer {
            let drag = matches!(
                event.kind,
                DesktopKind::MouseDragStart | DesktopKind::MouseDragMove
            );
            let margin = (35.0 * scale) as i32;
            let origin = if drag {
                (event.bounds.left, event.bounds.top)
            } else {
                (event.point.x, event.point.y)
            };
            // A fixed bounded drag viewport reuses the D2D target and DIB for
            // every move. Long connectors are clipped instead of allocating a
            // monitor-sized bitmap or rebuilding fonts 30 times per second.
            let (left, top, right, bottom) = if drag {
                let width = (640.0 * scale) as i32;
                let height = (360.0 * scale) as i32;
                let width = width.min(event.work_area.right - event.work_area.left);
                let height = height.min(event.work_area.bottom - event.work_area.top);
                let left = (event.point.x - width / 2)
                    .clamp(event.work_area.left, event.work_area.right - width);
                let top = (event.point.y - height / 2)
                    .clamp(event.work_area.top, event.work_area.bottom - height);
                (left, top, left + width, top + height)
            } else {
                let width = (70.0 * scale).round() as i32;
                let width = width.min(event.work_area.right - event.work_area.left);
                let height = width.min(event.work_area.bottom - event.work_area.top);
                let left =
                    (origin.0 - margin).clamp(event.work_area.left, event.work_area.right - width);
                let top =
                    (origin.1 - margin).clamp(event.work_area.top, event.work_area.bottom - height);
                (left, top, left + width, top + height)
            };
            if right <= left || bottom <= top {
                return None;
            }
            (
                navi_state::reactions::Bounds {
                    left,
                    top,
                    right,
                    bottom,
                },
                event.dpi,
            )
        } else {
            let mut placed_event = event;
            let placement = if event.kind != DesktopKind::WindowFocus {
                placed_event.bounds = event.work_area;
                Placement::NearCursor
            } else {
                self.placement
            };
            crate::placement::place(placed_event, placement, seed)?
        };
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        let origin = if pointer {
            (event.bounds.left, event.bounds.top)
        } else {
            (event.point.x, event.point.y)
        };
        let points = [
            (
                (origin.0 - bounds.left) as f32 / scale,
                (origin.1 - bounds.top) as f32 / scale,
            ),
            (
                (event.point.x - bounds.left) as f32 / scale,
                (event.point.y - bounds.top) as f32 / scale,
            ),
        ];
        Some(FocusEffectPlan {
            bounds: Rect {
                left: bounds.left,
                top: bounds.top,
                right: bounds.right,
                bottom: bounds.bottom,
            },
            family: reaction.family,
            drag: matches!(
                event.kind,
                DesktopKind::MouseDragStart | DesktopKind::MouseDragMove
            ),
            style: VisualStyle {
                intensity: reaction.intensity,
                seed,
            },
            points,
            dpi: event.dpi,
            size: SurfaceSize {
                width: width as f32 / scale,
                height: height as f32 / scale,
            },
            progress: elapsed as f32 / reaction.duration_ms as f32,
            remaining_ms: reaction.duration_ms.saturating_sub(elapsed).max(1) as u32,
        })
    }

    pub fn effect_bounds(&self, work: Rect, dpi: u32, size: SurfaceSize) -> Rect {
        let scale = dpi as f32 / 96.0;
        let width = (size.width * scale).round() as i32;
        let height = (size.height * scale).round() as i32;
        let margin = (20.0 * scale).round() as i32;
        let (x, y) = match self.effect_anchor {
            SurfaceAnchor::WorkArea => (
                work.right - margin - width,
                work.bottom - margin - height - (190.0 * scale) as i32,
            ),
            SurfaceAnchor::ContextRect(r) => (r.left, r.bottom + margin),
        };
        let x = x.clamp(work.left, (work.right - width).max(work.left));
        let y = y.clamp(work.top, (work.bottom - height).max(work.top));
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }
    pub fn present(&mut self, phase: Phase, now: u64) {
        if phase == Phase::Idle {
            self.idle();
            return;
        }
        let changed = self.phase != phase || !self.visible;
        self.phase = phase;
        if self.kind != SurfaceKind::Console {
            self.kind = if phase == Phase::Idle {
                SurfaceKind::Beacon
            } else {
                SurfaceKind::Compact
            };
            self.visible = true;
        } else {
            self.visible = true;
        }
        if changed {
            self.deadline = if phase == Phase::Responding
                && self.kind != SurfaceKind::Console
                && !self.protected
            {
                Some(now + RESPONSE_MS)
            } else {
                None
            };
        }
    }

    pub fn hide(&mut self) {
        self.idle();
    }

    pub fn idle(&mut self) {
        self.kind = SurfaceKind::Beacon;
        self.phase = Phase::Idle;
        self.visible = true;
        self.deadline = None;
    }

    pub fn protect(&mut self, session: &Session) {
        self.protected = !session.input.trim().is_empty() || session.animated();
        if self.protected {
            self.deadline = None;
        }
    }

    pub fn toggle_console(&mut self, now: u64) {
        self.kind = if self.kind == SurfaceKind::Console {
            SurfaceKind::Compact
        } else {
            SurfaceKind::Console
        };
        self.visible = true;
        self.deadline = if self.kind == SurfaceKind::Compact
            && self.phase == Phase::Responding
            && !self.protected
        {
            Some(now + RESPONSE_MS)
        } else {
            None
        };
    }

    pub fn remaining(&self, now: u64) -> Option<u32> {
        self.deadline
            .map(|end| end.saturating_sub(now).max(1) as u32)
    }

    pub fn expire(&mut self, now: u64) -> bool {
        if !self.protected && self.deadline.is_some_and(|end| now >= end) {
            self.idle();
            true
        } else {
            false
        }
    }

    pub fn plan(&self, work: Rect, dpi: u32, session: &Session) -> SurfacePlan {
        let size = surface_size(self.kind, session);
        let scale = dpi as f32 / 96.0;
        let margin = (20.0 * scale).round() as i32;
        let width = ((size.width * scale).round() as i32).min(work.right - work.left);
        let height = ((size.height * scale).round() as i32).min(work.bottom - work.top);
        let (x, y) = match self.anchor {
            SurfaceAnchor::WorkArea if self.kind == SurfaceKind::Console => (
                self.console_position
                    .map_or(work.left + (work.right - work.left - width) / 2, |p| p.0),
                self.console_position
                    .map_or(work.top + (work.bottom - work.top - height) / 2, |p| p.1),
            ),
            SurfaceAnchor::WorkArea => (work.right - margin - width, work.bottom - margin - height),
            SurfaceAnchor::ContextRect(rect) => (rect.right - width, rect.bottom + margin),
        };
        let x = x.clamp(work.left, (work.right - width).max(work.left));
        let y = y.clamp(work.top, (work.bottom - height).max(work.top));
        SurfacePlan {
            bounds: Rect {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            },
            size,
            kind: self.kind,
            visible: self.visible,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn work() -> Rect {
        Rect {
            left: -1920,
            top: 0,
            right: 0,
            bottom: 1040,
        }
    }
    #[test]
    fn response_returns_to_beacon_and_cancels_on_new_invocation() {
        let mut manager = SurfaceManager::default();
        manager.present(Phase::Responding, 10);
        assert!(!manager.expire(4009));
        assert!(manager.expire(4010));
        assert_eq!(manager.kind, SurfaceKind::Beacon);
        assert!(manager.visible);
        manager.present(Phase::Responding, 5000);
        manager.present(Phase::Invoked, 5100);
        assert!(!manager.expire(10000));
        assert!(manager.visible);
        manager.present(Phase::Responding, 11000);
        assert!(manager.expire(15000));
        assert!(manager.visible);
    }
    #[test]
    fn unsent_text_and_active_interaction_never_expire() {
        let mut manager = SurfaceManager::default();
        let mut s = Session {
            phase: Phase::Responding,
            input: "no enviar todavía".into(),
            ..Default::default()
        };
        manager.protect(&s);
        manager.present(s.phase, 0);
        assert!(!manager.expire(10000));
        assert_eq!(manager.kind, SurfaceKind::Compact);
        s.input.clear();
        s.phase = Phase::Processing;
        manager.protect(&s);
        manager.present(s.phase, 10001);
        assert!(!manager.expire(20000));
        manager.hide();
        assert_eq!(manager.kind, SurfaceKind::Beacon);
        assert!(manager.visible);
    }
    #[test]
    fn layout_follows_negative_monitor_work_area_and_grows_upward() {
        let mut manager = SurfaceManager::default();
        let mut s = Session {
            phase: Phase::Invoked,
            ..Default::default()
        };
        manager.present(s.phase, 0);
        let short = manager.plan(work(), 144, &s);
        s.input = "a".repeat(120);
        let long = manager.plan(work(), 144, &s);
        assert_eq!(short.bounds.right, -30);
        assert_eq!(short.bounds.bottom, 1010);
        assert_eq!(long.bounds.bottom, short.bounds.bottom);
        assert!(long.bounds.top < short.bounds.top);
        assert_eq!(long.bounds.right - long.bounds.left, 480);
    }
    #[test]
    fn contextual_anchors_clamp_and_console_does_not_auto_dismiss() {
        let mut manager = SurfaceManager {
            anchor: SurfaceAnchor::ContextRect(Rect {
                left: 0,
                top: 0,
                right: 40,
                bottom: 2000,
            }),
            ..Default::default()
        };
        manager.present(Phase::Responding, 0);
        manager.toggle_console(0);
        assert!(!manager.expire(10000));
        let plan = manager.plan(work(), 96, &Session::default());
        assert!(plan.bounds.right <= 0);
        assert!(plan.bounds.bottom <= 1040);
    }
}
