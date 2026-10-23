//! Keyboard invocation reuses the compact HWND and its existing text backend.
use super::*;

pub(super) unsafe fn invoke(hwnd: HWND, focus: bool) {
    let previous = GetForegroundWindow();
    commit(hwnd, |app| {
        focus_reaction::cancel(&mut app.reactions, &mut app.surfaces);
        if previous != hwnd {
            app.previous_focus = previous;
        }
        let monitor = MonitorFromWindow(app.previous_focus, MONITOR_DEFAULTTONEAREST);
        if let Ok(work) = monitor_work(app.previous_focus) {
            app.work = work;
            let mut dx = 96;
            let mut dy = 96;
            let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
            app.dpi = dx.max(96);
        }
        app.session.apply(Event::KeyboardInvoke(app.now()));
        app.surfaces.kind = SurfaceKind::Compact;
        app.surfaces.protect(&app.session);
        app.surfaces.present(app.session.phase, app.now());
        app.interference = 0;
        app.interference_until = None;
    });
    if focus {
        let _ = SetForegroundWindow(hwnd);
        SetFocus(hwnd);
    }
    commit(hwnd, |_| {});
}

/// Call from the existing surface sync, after rendering, while App is borrowed.
pub(super) unsafe fn sync(hwnd: HWND, session: &Session, kind: SurfaceKind, now: u64) {
    let _ = KillTimer(hwnd, 5);
    if kind == SurfaceKind::Compact
        && GetForegroundWindow() == hwnd
        && session.input.is_empty()
        && session.phase == Phase::Invoked
    {
        if let Some(entry) = session.keyboard_entry {
            let entry = navi_state::KeyboardEntry { now, ..entry };
            SetTimer(hwnd, 5, entry.next_tick().max(1), None);
        }
    }
}

pub(super) unsafe fn tick(hwnd: HWND) {
    commit(hwnd, |app| {
        app.session.apply(Event::KeyboardTick(app.now()));
    });
}

pub(super) unsafe fn cancel(hwnd: HWND) {
    let _ = KillTimer(hwnd, 5);
    with_app(|app| app.session.apply(Event::KeyboardCancel));
}

/// WM_CHAR decoder matches the console's UTF-16 handling; Enter never submits a demo.
pub(super) unsafe fn character(hwnd: HWND, wp: WPARAM) {
    match wp.0 {
        13 => text_backend::submit(hwnd),
        8 => update(hwnd, Event::Backspace),
        v if v >= 32 => {
            let unit = v as u16;
            let c = with_app(|app| {
                if (0xD800..=0xDBFF).contains(&unit) {
                    app.surrogate = Some(unit);
                    return None;
                }
                if let Some(high) = app.surrogate.take() {
                    char::decode_utf16([high, unit]).next().and_then(|r| r.ok())
                } else {
                    char::from_u32(unit as u32)
                }
            })
            .flatten();
            if let Some(c) = c {
                update(hwnd, Event::Character(c));
            }
        }
        _ => {}
    }
}
