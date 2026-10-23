//! Passive Raw Input. Never suppresses Windows mouse messages or installs hooks.
pub use navi_state::cursor::CursorSample;
use navi_state::{
    cursor::{CursorGesture, PointerInput},
    reactions::Point,
};
use windows::{
    core::Result,
    Win32::{
        Foundation::{HWND, LPARAM, POINT},
        UI::{
            HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
            Input::{KeyboardAndMouse::*, *},
            WindowsAndMessaging::*,
        },
    },
};

pub struct CursorObserver {
    gesture: CursorGesture,
    button: Option<(u16, u16, i32)>,
}

impl CursorObserver {
    /// # Safety
    /// `sink` must be a live window on the calling message thread.
    pub unsafe fn new(sink: HWND) -> Result<Self> {
        RegisterRawInputDevices(
            &[RAWINPUTDEVICE {
                usUsagePage: 1,
                usUsage: 2,
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: sink,
            }],
            std::mem::size_of::<RAWINPUTDEVICE>() as u32,
        )?;
        Ok(Self {
            gesture: CursorGesture::new(GetSystemMetrics(SM_CXDRAG), GetSystemMetrics(SM_CYDRAG)),
            button: None,
        })
    }

    /// # Safety
    /// Call only with a current `WM_INPUT` lParam on the owning UI thread.
    /// The caller still passes that message to `DefWindowProcW` for cleanup.
    pub unsafe fn observe(&mut self, lp: LPARAM, now: u64) -> Option<CursorSample> {
        let mut raw = RAWINPUT::default();
        let mut bytes = std::mem::size_of::<RAWINPUT>() as u32;
        let got = GetRawInputData(
            HRAWINPUT(lp.0),
            RID_INPUT,
            Some(&mut raw as *mut _ as _),
            &mut bytes,
            std::mem::size_of::<RAWINPUTHEADER>() as u32,
        );
        if got == u32::MAX
            || got
                < (std::mem::size_of::<RAWINPUTHEADER>() + std::mem::size_of::<RAWMOUSE>()) as u32
            || raw.header.dwType != RIM_TYPEMOUSE.0
        {
            return None;
        }
        let flags = raw.data.mouse.Anonymous.Anonymous.usButtonFlags;
        if self.button.is_none() {
            self.button = [
                (
                    RI_MOUSE_LEFT_BUTTON_DOWN,
                    RI_MOUSE_LEFT_BUTTON_UP,
                    VK_LBUTTON,
                ),
                (
                    RI_MOUSE_RIGHT_BUTTON_DOWN,
                    RI_MOUSE_RIGHT_BUTTON_UP,
                    VK_RBUTTON,
                ),
                (
                    RI_MOUSE_MIDDLE_BUTTON_DOWN,
                    RI_MOUSE_MIDDLE_BUTTON_UP,
                    VK_MBUTTON,
                ),
            ]
            .into_iter()
            .find(|(down, _, _)| flags as u32 & down != 0)
            .map(|(down, up, key)| (down as u16, up as u16, key.0 as i32));
        }
        let (down_flag, up_flag, key) = self.button?;
        let mut point = POINT::default();
        if GetCursorPos(&mut point).is_err() {
            self.gesture.reset();
            self.button = None;
            return None;
        }
        let target = GetAncestor(WindowFromPoint(point), GA_ROOT);
        if !self.gesture.pressed() {
            let dpi = GetDpiForWindow(target).max(96);
            self.gesture = CursorGesture::new(
                GetSystemMetricsForDpi(SM_CXDRAG, dpi),
                GetSystemMetricsForDpi(SM_CYDRAG, dpi),
            );
        }
        let mut pid = 0;
        GetWindowThreadProcessId(target, Some(&mut pid));
        // The layered connector spans the desktop. WindowFromPoint can return
        // that passive HWND even though Windows passes mouse input through it.
        let mut class = [0u16; 32];
        let count = GetClassNameW(target, &mut class).max(0) as usize;
        let decoration = pid == std::process::id()
            && String::from_utf16_lossy(&class[..count]) == "NaviHolograms";
        let result = self.gesture.feed(
            PointerInput {
                point: Point {
                    x: point.x,
                    y: point.y,
                },
                down: flags & down_flag != 0,
                up: flags & up_flag != 0,
                held: GetAsyncKeyState(key) < 0,
                own: pid == std::process::id() && !decoration,
                source: target.0 as usize,
            },
            now,
        );
        if !self.gesture.pressed() {
            self.button = None;
        }
        result
    }
}

impl Drop for CursorObserver {
    fn drop(&mut self) {
        unsafe {
            let _ = RegisterRawInputDevices(
                &[RAWINPUTDEVICE {
                    usUsagePage: 1,
                    usUsage: 2,
                    dwFlags: RIDEV_REMOVE,
                    hwndTarget: HWND(0),
                }],
                std::mem::size_of::<RAWINPUTDEVICE>() as u32,
            );
        }
    }
}
