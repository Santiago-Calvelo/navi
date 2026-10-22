//! Raw-input adapter to the existing event policy and surface coordinator.
use super::*;
use navi_state::{
    cursor::{CursorKind, CursorSample},
    reactions::{Bounds, DesktopEvent, DesktopKind},
};
use windows::Win32::UI::Shell::*;

pub(super) unsafe fn apply(app: &mut App, sample: CursorSample, now: u64) -> bool {
    let kind = match sample.kind {
        CursorKind::Click => DesktopKind::MouseClick,
        CursorKind::DragStart => DesktopKind::MouseDragStart,
        CursorKind::DragMove => DesktopKind::MouseDragMove,
        CursorKind::DragEnd => DesktopKind::MouseDragEnd,
    };
    // Always terminate a drag, including a release over NAVI or a lost button.
    if kind == DesktopKind::MouseDragEnd {
        let mut event = dummy(kind);
        event.own = sample.own;
        return focus_reaction::apply(
            &mut app.reactions,
            &mut app.surfaces,
            &mut app.effects,
            event,
            now,
        );
    }
    if sample.own {
        return false;
    }
    let point = POINT {
        x: sample.point.x,
        y: sample.point.y,
    };
    let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        return false;
    }
    let foreground = GetForegroundWindow();
    let mut rect = RECT::default();
    let full = GetWindowRect(foreground, &mut rect).is_ok()
        && rect.left <= info.rcMonitor.left
        && rect.top <= info.rcMonitor.top
        && rect.right >= info.rcMonitor.right
        && rect.bottom >= info.rcMonitor.bottom;
    let blocked = SHQueryUserNotificationState().ok().is_some_and(|s| {
        matches!(
            s,
            QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_BUSY | QUNS_PRESENTATION_MODE | QUNS_NOT_PRESENT
        )
    });
    let mut dx = 96;
    let mut dy = 96;
    let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
    let event = DesktopEvent {
        kind,
        source: sample.source,
        point: sample.point,
        own: false,
        suppressed: full || blocked,
        dpi: dx.max(96),
        bounds: Bounds {
            left: sample.origin.x,
            top: sample.origin.y,
            right: sample.point.x,
            bottom: sample.point.y,
        },
        work_area: Bounds {
            left: info.rcWork.left,
            top: info.rcWork.top,
            right: info.rcWork.right,
            bottom: info.rcWork.bottom,
        },
    };
    focus_reaction::apply(
        &mut app.reactions,
        &mut app.surfaces,
        &mut app.effects,
        event,
        now,
    )
}

fn dummy(kind: DesktopKind) -> DesktopEvent {
    DesktopEvent {
        kind,
        source: 0,
        bounds: Bounds::default(),
        work_area: Bounds::default(),
        point: Default::default(),
        own: false,
        suppressed: false,
        dpi: 96,
    }
}
