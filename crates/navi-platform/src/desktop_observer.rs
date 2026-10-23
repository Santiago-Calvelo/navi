//! Out-of-context WinEvents. Only handles, bounds and process IDs; no titles or contents.
use super::*;
use navi_state::reactions::{Bounds, DesktopEvent, DesktopKind, Point};
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::Win32::{Graphics::Dwm::*, UI::Accessibility::*};

pub const MESSAGE: u32 = WM_APP + 40;
pub const SETTLED_MESSAGE: u32 = WM_APP + 42;
pub const FOCUS_EVENT: u32 = EVENT_SYSTEM_FOREGROUND;
static SINK: AtomicIsize = AtomicIsize::new(0);

pub struct DesktopObserver {
    hooks: Vec<HWINEVENTHOOK>,
}

fn is_window_focus_event(event: u32, hwnd: HWND, object: i32, child: i32) -> bool {
    matches!(event, FOCUS_EVENT | EVENT_SYSTEM_MINIMIZEEND)
        && hwnd.0 != 0
        && object == 0
        && child == 0
}

pub fn is_current_focus_message(event: u32, source: HWND, current: HWND) -> bool {
    event == FOCUS_EVENT && source.0 != 0 && source == current
}

unsafe extern "system" fn event_callback(
    _: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    object: i32,
    child: i32,
    _: u32,
    _: u32,
) {
    if !is_window_focus_event(event, hwnd, object, child) {
        return;
    }
    let sink = HWND(SINK.load(Ordering::Relaxed));
    if sink.0 != 0 {
        let _ = PostMessageW(sink, MESSAGE, WPARAM(FOCUS_EVENT as usize), LPARAM(hwnd.0));
    }
}

impl DesktopObserver {
    pub unsafe fn new(sink: HWND) -> Result<Self> {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if hook.0 == 0 {
            return Err(Error::from_win32());
        }
        let restored = SetWinEventHook(
            EVENT_SYSTEM_MINIMIZEEND,
            EVENT_SYSTEM_MINIMIZEEND,
            None,
            Some(event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if restored.0 == 0 {
            let _ = UnhookWinEvent(hook);
            return Err(Error::from_win32());
        }
        SINK.store(sink.0, Ordering::Relaxed);
        Ok(Self {
            hooks: vec![hook, restored],
        })
    }
}
impl Drop for DesktopObserver {
    fn drop(&mut self) {
        unsafe {
            SINK.store(0, Ordering::Relaxed);
            for hook in &self.hooks {
                let _ = UnhookWinEvent(*hook);
            }
        }
    }
}

pub unsafe fn resolve(hwnd: HWND) -> Option<DesktopEvent> {
    // Queued notifications can become obsolete before the UI thread resolves
    // them. Only the current foreground may create a manifestation.
    if GetForegroundWindow() != hwnd
        || !IsWindowVisible(hwnd).as_bool()
        || IsIconic(hwnd).as_bool()
        || GetAncestor(hwnd, GA_ROOT) != hwnd
        || GetWindow(hwnd, GW_OWNER).0 != 0
    {
        return None;
    }
    if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0 {
        return None;
    }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 || pid == std::process::id() {
        return None;
    }
    let mut cloaked = 0u32;
    if DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as _, 4).is_ok()
        && cloaked != 0
    {
        return None;
    }
    let mut rect = RECT::default();
    GetWindowRect(hwnd, &mut rect).ok()?;
    if rect.right - rect.left < 120 || rect.bottom - rect.top < 80 {
        return None;
    }
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut info).as_bool() {
        return None;
    }
    let full = !IsZoomed(hwnd).as_bool()
        && rect.left <= info.rcMonitor.left
        && rect.top <= info.rcMonitor.top
        && rect.right >= info.rcMonitor.right
        && rect.bottom >= info.rcMonitor.bottom;
    let notification = windows::Win32::UI::Shell::SHQueryUserNotificationState().ok();
    let blocked = notification.is_some_and(|s| {
        matches!(
            s,
            windows::Win32::UI::Shell::QUNS_RUNNING_D3D_FULL_SCREEN
                | windows::Win32::UI::Shell::QUNS_BUSY
                | windows::Win32::UI::Shell::QUNS_PRESENTATION_MODE
                | windows::Win32::UI::Shell::QUNS_NOT_PRESENT
        )
    });
    let mut cursor = POINT {
        x: rect.right - 200,
        y: rect.top + 40,
    };
    let _ = GetCursorPos(&mut cursor);
    Some(DesktopEvent {
        kind: DesktopKind::WindowFocus,
        source: hwnd.0 as usize,
        bounds: Bounds {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        },
        work_area: Bounds {
            left: info.rcWork.left,
            top: info.rcWork.top,
            right: info.rcWork.right,
            bottom: info.rcWork.bottom,
        },
        point: Point {
            x: cursor.x,
            y: cursor.y,
        },
        own: false,
        suppressed: full || blocked,
        dpi: GetDpiForWindow(hwnd).max(96),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_accepts_only_top_level_focus_notifications() {
        assert!(is_window_focus_event(FOCUS_EVENT, HWND(1), 0, 0));
        assert!(!is_window_focus_event(FOCUS_EVENT, HWND(0), 0, 0));
        assert!(!is_window_focus_event(FOCUS_EVENT, HWND(1), -4, 0));
        assert!(!is_window_focus_event(FOCUS_EVENT, HWND(1), 0, 1));
        assert!(!is_window_focus_event(EVENT_OBJECT_SHOW, HWND(1), 0, 0));
    }
    #[test]
    fn stale_queued_window_and_other_message_types_are_ignored() {
        assert!(is_current_focus_message(FOCUS_EVENT, HWND(1), HWND(1)));
        assert!(!is_current_focus_message(FOCUS_EVENT, HWND(1), HWND(2)));
        assert!(!is_current_focus_message(
            EVENT_OBJECT_SHOW,
            HWND(1),
            HWND(1)
        ));
    }
}
