//! Synthetic policy events and our own HWNDs only; no external input injection.
use super::*;
use navi_state::reactions::{Bounds, DesktopEvent, DesktopKind, Point, ReactionEngine};
use std::time::Duration;

pub(super) unsafe fn verify(hwnd: HWND) -> Result<Vec<(&'static str, bool)>> {
    if with_app(|_| ()).is_none() {
        return Err(Error::new(E_FAIL, "Cursor QA requires a live NAVI App"));
    }
    let mut checks = Vec::with_capacity(6);
    // Snapshots normally have no passive observer. Do not replace an existing
    // process-wide Raw Input registration if this helper is reused elsewhere.
    let registered = if with_app(|a| a.cursor.is_some()).unwrap_or(false) {
        true
    } else {
        cursor_observer::CursorObserver::new(hwnd).is_ok()
    };
    checks.push(("cursor_raw_input_registration_smoke", registered));
    reset(hwnd);
    let foreground = GetForegroundWindow();
    let click = with_app(|a| {
        let point = Point {
            x: a.work.left + (a.work.right - a.work.left) / 2,
            y: a.work.top + (a.work.bottom - a.work.top) / 2,
        };
        DesktopEvent {
            kind: DesktopKind::MouseClick,
            source: 1,
            bounds: Bounds {
                left: point.x,
                top: point.y,
                right: point.x,
                bottom: point.y,
            },
            work_area: Bounds {
                left: a.work.left,
                top: a.work.top,
                right: a.work.right,
                bottom: a.work.bottom,
            },
            point,
            own: false,
            suppressed: false,
            dpi: a.dpi,
        }
    })
    .ok_or_else(|| Error::new(E_FAIL, "App disappeared during cursor QA"))?;
    dispatch(hwnd, click);
    verification::pump(Duration::from_millis(90));
    checks.push((
        "cursor_click_native_alpha_noactivation_passthrough",
        with_app(|a| {
            a.overlay.as_ref().is_some_and(|o| {
                let alpha = o.renderer.alpha_stats();
                let flags = GetWindowLongPtrW(o.hwnd, GWL_EXSTYLE) as u32;
                let required = (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE).0;
                alpha.0 > 0
                    && alpha.1 > 0
                    && flags & required == required
                    && GetForegroundWindow() == foreground
            })
        })
        .unwrap_or(false),
    ));
    verification::pump(Duration::from_millis(300));
    checks.push(("cursor_click_expires_to_quiet_beacon", quiet(hwnd)));

    let start = DesktopEvent {
        kind: DesktopKind::MouseDragStart,
        dpi: 96,
        work_area: Bounds {
            left: GetSystemMetrics(SM_XVIRTUALSCREEN),
            top: GetSystemMetrics(SM_YVIRTUALSCREEN),
            right: GetSystemMetrics(SM_XVIRTUALSCREEN) + GetSystemMetrics(SM_CXVIRTUALSCREEN),
            bottom: GetSystemMetrics(SM_YVIRTUALSCREEN) + GetSystemMetrics(SM_CYVIRTUALSCREEN),
        },
        bounds: Bounds {
            left: click.point.x - 50,
            top: click.point.y - 20,
            right: click.point.x,
            bottom: click.point.y,
        },
        ..click
    };
    dispatch(hwnd, start);
    let overlay = with_app(|a| {
        a.overlay.as_ref().map(|o| {
            let alpha = o.renderer.alpha_stats();
            (o.hwnd, o.renderer.width, o.renderer.height, alpha.1 > 0)
        })
    })
    .flatten();
    checks.push((
        "cursor_drag_initial_static_frame_visible",
        overlay.is_some_and(|(_, _, _, visible)| visible) && GetForegroundWindow() == foreground,
    ));
    let moved = DesktopEvent {
        kind: DesktopKind::MouseDragMove,
        point: Point {
            x: start.work_area.right - 20,
            y: start.work_area.bottom - 20,
        },
        ..start
    };
    dispatch(hwnd, moved);
    checks.push((
        "cursor_drag_growing_canvas_retains_origin",
        with_app(|a| {
            a.overlay.is_some() && a.surfaces.focus_effect_plan(a.now()).is_some_and(|p| {
                p.dpi == 96
                    && p.bounds.left as f32 + p.points[0].0 == start.bounds.left as f32
                    && p.bounds.top as f32 + p.points[0].1 == start.bounds.top as f32
                    && p.bounds.left as f32 + p.points[1].0 == moved.point.x as f32
                    && p.bounds.top as f32 + p.points[1].1 == moved.point.y as f32
            })
        })
        .unwrap_or(false),
    ));
    dispatch(
        hwnd,
        DesktopEvent {
            kind: DesktopKind::MouseDragEnd,
            own: true,
            ..moved
        },
    );
    checks.push((
        "cursor_drag_release_over_navi_cleans_immediately",
        quiet(hwnd),
    ));
    reset(hwnd);
    Ok(checks)
}

unsafe fn dispatch(hwnd: HWND, event: DesktopEvent) {
    commit(hwnd, |a| {
        let now = a.now();
        focus_reaction::apply(
            &mut a.reactions,
            &mut a.surfaces,
            &mut a.effects,
            event,
            now,
        );
    });
}

unsafe fn quiet(hwnd: HWND) -> bool {
    IsWindowVisible(hwnd).as_bool()
        && with_app(|a| {
            a.overlay.is_none()
                && a.reactions.active.is_empty()
                && a.surfaces.kind == SurfaceKind::Beacon
                && a.session.phase == Phase::Idle
                && a.timers == [false; 4]
        })
        .unwrap_or(false)
}

unsafe fn reset(hwnd: HWND) {
    commit(hwnd, |a| {
        a.reactions = ReactionEngine::default();
        a.session.apply(Event::Reset);
        a.surfaces.idle();
        a.surfaces.protect(&a.session);
        a.surfaces.clear_focus_effect();
        a.effects.clear();
        a.effects.enabled = 0;
        a.interference = 0;
        a.interference_until = None;
    });
}
