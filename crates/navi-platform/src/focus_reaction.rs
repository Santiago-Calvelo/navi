//! Tested coordination between event policy and transient surface placement.
//! No HWND, renderer, focus activation or session mutation.
use crate::surface::SurfaceManager;
use navi_state::{
    holograms::Holograms,
    reactions::{DesktopEvent, ReactionDecision, ReactionEngine},
};

pub(super) fn apply(
    engine: &mut ReactionEngine,
    surfaces: &mut SurfaceManager,
    effects: &mut Holograms,
    event: DesktopEvent,
    now: u64,
) -> bool {
    match engine.decide(event, now) {
        ReactionDecision::Start(reaction) => {
            // A new foreground supersedes decorative content, without changing
            // its demo settings, the session, the beacon or the console.
            effects.clear();
            if !surfaces.begin_focus_effect(reaction) {
                engine.clear();
            }
            true
        }
        ReactionDecision::Cancel => surfaces.clear_focus_effect(),
        ReactionDecision::Unchanged => false,
    }
}

pub(super) fn cancel(engine: &mut ReactionEngine, surfaces: &mut SurfaceManager) -> bool {
    engine.forget_focus();
    surfaces.clear_focus_effect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{Rect, SurfaceAnchor};
    use navi_state::{
        reactions::{Bounds, DesktopKind, Point},
        Phase, Session,
    };
    use navi_ui::SurfaceKind;

    fn focus(source: usize) -> DesktopEvent {
        DesktopEvent {
            kind: DesktopKind::WindowFocus,
            source,
            bounds: Bounds {
                left: -1600,
                top: 100,
                right: -300,
                bottom: 900,
            },
            work_area: Bounds {
                left: -1920,
                top: 0,
                right: 0,
                bottom: 1040,
            },
            point: Point::default(),
            own: false,
            suppressed: false,
            dpi: 144,
        }
    }

    #[test]
    fn focus_to_surface_uses_target_monitor_and_preserves_beacon_and_demo_anchor() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        let anchor = SurfaceAnchor::ContextRect(Rect {
            left: 20,
            top: 30,
            right: 40,
            bottom: 50,
        });
        surfaces.effect_anchor = anchor;
        assert!(apply(&mut engine, &mut surfaces, &mut effects, focus(1), 0));
        let plan = surfaces.focus_effect_plan(100).unwrap();
        assert_eq!(plan.dpi, 144);
        assert!(plan.size.width >= 270.0 && plan.size.width <= 540.0);
        assert!(plan.bounds.left >= -1600 && plan.bounds.right <= -300);
        assert!(plan.bounds.top >= 100 && plan.bounds.bottom <= 900);
        assert_eq!(surfaces.kind, SurfaceKind::Beacon);
        assert!(surfaces.visible);
        assert_eq!(surfaces.effect_anchor, anchor);
        surfaces.expire_focus_effect(850);
        engine.prune(850);
        assert!(surfaces.focus_effect_plan(850).is_none());
        assert!(engine.active.is_empty());
        assert_eq!(surfaces.kind, SurfaceKind::Beacon);
    }

    #[test]
    fn duplicate_is_unchanged_new_focus_during_cooldown_cancels_old_visual() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        assert!(apply(&mut engine, &mut surfaces, &mut effects, focus(1), 0));
        assert!(!apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            focus(1),
            100
        ));
        assert!(apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            focus(2),
            200
        ));
        assert!(surfaces.focus_effect_plan(200).is_none());
        assert!(apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            focus(3),
            700
        ));
        assert!(surfaces.focus_effect_plan(700).is_some());
        assert!(apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            DesktopEvent {
                suppressed: true,
                ..focus(4)
            },
            800
        ));
        assert!(surfaces.focus_effect_plan(800).is_none());
    }

    #[test]
    fn own_events_and_disabled_reactions_do_not_create_surfaces_or_erase_drafts() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        let session = Session {
            phase: Phase::Invoked,
            input: "borrador pendiente".into(),
            ..Default::default()
        };
        surfaces.present(session.phase, 0);
        surfaces.protect(&session);
        assert!(!apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            DesktopEvent {
                own: true,
                ..focus(1)
            },
            0
        ));
        assert!(surfaces.focus_effect_plan(0).is_none());
        assert!(apply(&mut engine, &mut surfaces, &mut effects, focus(2), 1));
        surfaces.expire_focus_effect(851);
        assert_eq!(surfaces.kind, SurfaceKind::Compact);
        assert_eq!(session.input, "borrador pendiente");
        engine.set_enabled(false);
        assert!(!apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            focus(3),
            1000
        ));
        assert!(surfaces.focus_effect_plan(1000).is_none());
    }

    #[test]
    fn small_window_clamps_effect_and_invalid_geometry_cannot_leave_stale_visual() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        let small = DesktopEvent {
            bounds: Bounds {
                left: 10,
                top: 10,
                right: 160,
                bottom: 110,
            },
            work_area: Bounds {
                left: 0,
                top: 0,
                right: 300,
                bottom: 200,
            },
            dpi: 192,
            ..focus(1)
        };
        assert!(apply(&mut engine, &mut surfaces, &mut effects, small, 0));
        let plan = surfaces.focus_effect_plan(10).unwrap();
        assert!(plan.bounds.left >= 10 && plan.bounds.right <= 160);
        assert!(plan.bounds.top >= 10 && plan.bounds.bottom <= 110);
        let invalid = DesktopEvent {
            work_area: Bounds::default(),
            ..focus(2)
        };
        apply(&mut engine, &mut surfaces, &mut effects, invalid, 1000);
        assert!(surfaces.focus_effect_plan(1000).is_none());
    }

    #[test]
    fn explicitly_focusing_navi_cancels_and_allows_return_to_previous_app() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        apply(&mut engine, &mut surfaces, &mut effects, focus(1), 0);
        assert!(cancel(&mut engine, &mut surfaces));
        assert!(apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            focus(1),
            1000
        ));
    }
    #[test]
    fn cursor_pipeline_uses_stable_viewport_and_release_clears_without_touching_beacon() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        let start = DesktopEvent {
            kind: DesktopKind::MouseDragStart,
            point: Point { x: -600, y: 350 },
            bounds: Bounds {
                left: -1000,
                top: 300,
                right: -600,
                bottom: 350,
            },
            ..focus(1)
        };
        assert!(apply(&mut engine, &mut surfaces, &mut effects, start, 0));
        let p = surfaces.focus_effect_plan(1).unwrap();
        assert!(p.bounds.left >= start.work_area.left && p.bounds.right <= start.work_area.right);
        let moved = DesktopEvent {
            kind: DesktopKind::MouseDragMove,
            point: Point { x: -400, y: 600 },
            ..start
        };
        assert!(apply(&mut engine, &mut surfaces, &mut effects, moved, 50));
        assert_eq!(surfaces.focus_effect_plan(51).unwrap().size, p.size);
        assert!(apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            DesktopEvent {
                kind: DesktopKind::MouseDragEnd,
                own: true,
                ..moved
            },
            52
        ));
        assert!(surfaces.focus_effect_plan(53).is_none());
        assert_eq!(surfaces.kind, SurfaceKind::Beacon);
        assert!(surfaces.visible);
    }
    #[test]
    fn alternate_pointer_family_uses_aspect_correct_placement() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        engine.config.click_family = navi_state::visual::Family::StackV1;
        let click = DesktopEvent {
            kind: DesktopKind::MouseClick,
            point: Point { x: -600, y: 350 },
            bounds: Bounds::default(),
            ..focus(1)
        };
        assert!(apply(&mut engine, &mut surfaces, &mut effects, click, 0));
        let p = surfaces.focus_effect_plan(1).unwrap();
        assert!((p.size.width / p.size.height - 1.8).abs() < 0.02);
    }
}
