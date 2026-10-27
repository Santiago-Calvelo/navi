//! Geometry snapshots only. Refresh topology on Windows messages, never on a timer.
use super::*;
use crate::organic::MonitorArea;
use navi_state::reactions::Bounds;

fn bounds(r: RECT) -> Bounds {
    Bounds {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}

unsafe extern "system" fn enumerate(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(monitor, &mut info).as_bool() {
        let (mut x, mut y) = (96, 96);
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
        (&mut *(data.0 as *mut Vec<MonitorArea>)).push(MonitorArea {
            id: monitor.0 as u64,
            work: bounds(info.rcWork),
            dpi: x.max(96),
            blocked: false,
        });
    }
    BOOL(1)
}

pub(super) unsafe fn collect() -> Vec<MonitorArea> {
    let mut monitors = Vec::<MonitorArea>::with_capacity(4);
    let _ = EnumDisplayMonitors(
        None,
        None,
        Some(enumerate),
        LPARAM(&mut monitors as *mut _ as isize),
    );
    monitors.sort_by_key(|m| (m.work.left, m.work.top, m.dpi));
    monitors
}

unsafe fn covered(monitor: &MonitorArea) -> bool {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(HMONITOR(monitor.id as isize), &mut info).as_bool() {
        return true;
    }
    let r = info.rcMonitor;
    let point = POINT {
        x: (i64::from(r.left) + (i64::from(r.right) - i64::from(r.left)) / 2) as i32,
        y: (i64::from(r.top) + (i64::from(r.bottom) - i64::from(r.top)) / 2) as i32,
    };
    let window = GetAncestor(WindowFromPoint(point), GA_ROOT);
    let mut pid = 0;
    GetWindowThreadProcessId(window, Some(&mut pid));
    if window.0 == 0
        || pid == std::process::id()
        || IsZoomed(window).as_bool()
        || window == GetShellWindow()
        || window == GetDesktopWindow()
        || !IsWindowVisible(window).as_bool()
        || IsIconic(window).as_bool()
    {
        return false;
    }
    let mut class = [0u16; 32];
    let count = GetClassNameW(window, &mut class).max(0) as usize;
    let class = String::from_utf16_lossy(&class[..count]);
    if class == "WorkerW" || class == "Progman" {
        return false;
    }
    let mut actual = RECT::default();
    GetWindowRect(window, &mut actual).is_ok()
        && actual.left <= r.left
        && actual.top <= r.top
        && actual.right >= r.right
        && actual.bottom >= r.bottom
}

pub(super) unsafe fn prepare(app: &mut App) {
    // Only called on focus admission/explicit previews, not from sync/WM_TIMER.
    for monitor in app.surfaces.monitors.iter_mut() {
        monitor.blocked = covered(monitor);
    }
    app.surfaces.effect_motion = app.effects.motion;
    let plan = app.surfaces.plan(app.work, app.dpi, &app.session);
    let own = plan.visible.then_some(Bounds {
        left: plan.bounds.left,
        top: plan.bounds.top,
        right: plan.bounds.right,
        bottom: plan.bounds.bottom,
    });
    let panel = app.surfaces.avoid.map(|r| Bounds {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    });
    app.surfaces.exclusions = [own, panel, None];
}

pub(super) unsafe fn refresh(app: &mut App) {
    if app.startup.take().is_some() {
        app.overlay = None;
    }
    if app.passive_observers {
        app.surfaces.monitors = collect();
    }
    app.surfaces.clear_focus_effect();
    app.reactions.clear();
}
