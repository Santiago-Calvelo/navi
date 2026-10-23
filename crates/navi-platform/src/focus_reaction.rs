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
    engine.config.focus_family = navi_state::visual::Family::StackV2;
    match engine.decide(event, now) {
        ReactionDecision::Start(reaction) => {
            // A new foreground supersedes decorative content, without changing
            // its demo settings, the session, the beacon or the console.
            effects.clear();
            if !surfaces.begin_focus_effect(reaction) {
                engine.clear();
            } else if let Some(active) = engine.active.last_mut() {
                active.duration_ms = surfaces.focus_duration().unwrap_or(reaction.duration_ms);
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
    fn obsolete_pointer_family_cannot_replace_click_pulse() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            SurfaceManager::default(),
            Holograms::default(),
        );
        engine.config.click_family = navi_state::visual::Family::Sutura;
        let click = DesktopEvent {
            kind: DesktopKind::MouseClick,
            point: Point { x: -600, y: 350 },
            bounds: Bounds::default(),
            ..focus(1)
        };
        assert!(apply(&mut engine, &mut surfaces, &mut effects, click, 0));
        let p = surfaces.focus_effect_plan(1).unwrap();
        assert_eq!(p.family, navi_state::visual::Family::ClickPulse);
    }

    fn organic_surfaces() -> SurfaceManager {
        let mut surfaces = SurfaceManager::default();
        surfaces.seed = Some(42);
        surfaces.monitors = vec![
            crate::organic::MonitorArea {
                id: 1,
                work: focus(1).work_area,
                dpi: 144,
                blocked: false,
            },
            crate::organic::MonitorArea {
                id: 2,
                work: Bounds {
                    left: 0,
                    top: -200,
                    right: 2560,
                    bottom: 1240,
                },
                dpi: 192,
                blocked: false,
            },
        ];
        surfaces
    }

    #[test]
    fn organic_plan_is_frozen_and_engine_surface_expire_together() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            organic_surfaces(),
            Holograms::default(),
        );
        apply(&mut engine, &mut surfaces, &mut effects, focus(1), 0);
        let first = surfaces.focus_effect_plan(0).unwrap();
        let duration = first.variation.unwrap().duration_ms;
        assert_eq!(duration, engine.active[0].duration_ms);
        surfaces.seed = Some(99);
        surfaces.placement = navi_state::visual::Placement::SafeRandom;
        surfaces.exclusions = [Some(focus(1).work_area); 3];
        let last = surfaces.focus_effect_plan(duration - 1).unwrap();
        assert_eq!(first.bounds, last.bounds);
        assert_eq!(first.variation, last.variation);
        assert_eq!(first.draw_scale, last.draw_scale);
        assert_eq!(first.dpi, last.dpi);
        assert_eq!(last.remaining_ms, 1);
        assert!(last.progress > first.progress);
        engine.prune(duration - 1);
        assert_eq!(engine.active.len(), 1);
        engine.prune(duration);
        surfaces.expire_focus_effect(duration);
        assert!(engine.active.is_empty());
        assert!(surfaces.focus_effect_plan(duration).is_none());
        assert!(surfaces.focus_duration().is_none());
        assert_eq!(surfaces.kind, SurfaceKind::Beacon);
    }

    #[test]
    fn suppression_does_not_consume_a_sample_or_escape_to_another_monitor() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            organic_surfaces(),
            Holograms::default(),
        );
        apply(
            &mut engine,
            &mut surfaces,
            &mut effects,
            DesktopEvent {
                suppressed: true,
                ..focus(1)
            },
            0,
        );
        assert!(surfaces.focus_effect_plan(0).is_none());
        assert!(engine.active.is_empty());
        apply(&mut engine, &mut surfaces, &mut effects, focus(2), 1000);
        let p = surfaces.focus_effect_plan(1000).unwrap();
        assert_eq!(p.style.seed, 42);
        surfaces.monitors.iter_mut().for_each(|m| m.blocked = true);
        apply(&mut engine, &mut surfaces, &mut effects, focus(3), 2000);
        assert!(surfaces.focus_effect_plan(2000).is_none());
        assert!(engine.active.is_empty());
    }

    #[test]
    fn reduced_motion_is_static_with_original_lifetime_and_no_rotation() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            organic_surfaces(),
            Holograms::default(),
        );
        surfaces.effect_motion = false;
        apply(&mut engine, &mut surfaces, &mut effects, focus(1), 0);
        let p = surfaces.focus_effect_plan(10).unwrap();
        let v = p.variation.unwrap();
        assert!(!p.motion);
        assert_eq!(v.rotation_radians, 0.0);
        assert_eq!(v.rhythm, navi_state::visual::Rhythm::Linear);
        assert_eq!(engine.active[0].duration_ms, 850);
        surfaces.clear_focus_effect();
        assert!(surfaces.focus_effect_plan(20).is_none());
    }

    #[test]
    fn cursor_anchors_remain_physical_even_with_a_remote_catalog() {
        let (mut engine, mut surfaces, mut effects) = (
            ReactionEngine::default(),
            organic_surfaces(),
            Holograms::default(),
        );
        // No eligible free monitor: pointer events must still use their real monitor.
        surfaces.monitors.iter_mut().for_each(|m| m.blocked = true);
        for (i, dpi) in [96, 144, 192].into_iter().enumerate() {
            let e = DesktopEvent {
                kind: DesktopKind::MouseClick,
                point: Point { x: -400, y: 500 },
                dpi,
                ..focus(i + 1)
            };
            apply(&mut engine, &mut surfaces, &mut effects, e, i as u64 * 1000);
            let p = surfaces.focus_effect_plan(i as u64 * 1000 + 1).unwrap();
            assert!(p.variation.is_none());
            assert_eq!(p.dpi, dpi);
            let scale = dpi as f32 / 96.0;
            assert!((p.bounds.left as f32 + p.points[1].0 * scale - e.point.x as f32).abs() < 0.01);
            assert!((p.bounds.top as f32 + p.points[1].1 * scale - e.point.y as f32).abs() < 0.01);
        }
        engine.config.drag_family = navi_state::visual::Family::StackV2;
        let drag = DesktopEvent {
            kind: DesktopKind::MouseDragStart,
            point: Point { x: -700, y: 500 },
            ..focus(9)
        };
        apply(&mut engine, &mut surfaces, &mut effects, drag, 4000);
        let p = surfaces.focus_effect_plan(4001).unwrap();
        assert!(p.variation.is_none());
        assert_eq!(p.dpi, drag.dpi);
        assert!(p.bounds.right <= 0);
    }
    #[test]
    fn temporal_family_lifetime_and_static_policy_share_the_resolved_deadline() {
        use navi_state::visual::Family;
        for (family, base) in [(Family::StackV2, 1200), (Family::Sutura, 1800)] {
            for motion in [true, false] {
                let mut surfaces = organic_surfaces();
                surfaces.effect_motion = motion;
                let reaction = navi_state::reactions::Reaction {
                    event: focus(1),
                    start: 100,
                    duration_ms: if family == Family::Sutura { 1800 } else { 850 },
                    intensity: navi_state::reactions::Intensity::Experimental,
                    family,
                };
                assert!(surfaces.begin_focus_effect(reaction));
                let first = surfaces.focus_effect_plan(100).unwrap();
                let v = first.variation.unwrap();
                if motion {
                    assert!((base * 9 / 10..=base * 11 / 10).contains(&v.duration_ms));
                } else {
                    assert_eq!(
                        v.duration_ms,
                        if family == Family::StackV2 { 850 } else { 1800 }
                    );
                    assert_eq!(v.rotation_radians, 0.0);
                }
                assert_eq!(first.motion, motion);
                assert_eq!(first.style.temporal, Some(v.temporal));
                let last = surfaces.focus_effect_plan(100 + v.duration_ms - 1).unwrap();
                assert_eq!(last.remaining_ms, 1);
                assert_eq!(first.style.temporal, last.style.temporal);
                assert_eq!(first.bounds, last.bounds);
                assert!(surfaces.focus_effect_plan(100 + v.duration_ms).is_none());
                surfaces.expire_focus_effect(100 + v.duration_ms);
                assert!(surfaces.focus_duration().is_none());
            }
        }
    }

    #[test]
    fn sutura_is_not_selected_by_default_focus_or_pointer_events() {
        use navi_state::visual::Family;
        let config = navi_state::reactions::ReactionConfig::default();
        assert_eq!(config.focus_family, Family::StackV2);
        assert_eq!(config.click_family, Family::ClickPulse);
        assert_eq!(config.drag_family, Family::DragConnector);
    }
}
