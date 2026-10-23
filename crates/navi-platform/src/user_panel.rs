//! The user's writing window is separate from NAVI's response and decoration.
use super::*;
use windows::Win32::UI::{
    Controls::{EM_SETCUEBANNER, EM_SETLIMITTEXT},
    Shell::{DefSubclassProc, SetWindowSubclass},
};

pub(super) struct UserPanel {
    pub hwnd: HWND,
    pub edit: HWND,
    pub button: HWND,
    pub renderer: Renderer,
    pub bounds: Rect,
    font: HFONT,
    background: HBRUSH,
    dpi: u32,
    previous: HWND,
}

unsafe extern "system" fn input_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    main: usize,
) -> LRESULT {
    match msg {
        WM_KEYDOWN if wp.0 == VK_RETURN.0 as usize => {
            send(HWND(main as isize));
            return LRESULT(0);
        }
        WM_KEYDOWN if wp.0 == VK_ESCAPE.0 as usize => {
            hide(HWND(main as isize));
            return LRESULT(0);
        }
        WM_KEYDOWN if wp.0 == VK_TAB.0 as usize => {
            let target = with_app(|a| {
                a.user_panel
                    .as_ref()
                    .map(|p| if hwnd == p.edit { p.button } else { p.edit })
            })
            .flatten();
            if let Some(target) = target {
                SetFocus(target);
            }
            return LRESULT(0);
        }
        WM_CHAR if matches!(wp.0, 9 | 13 | 27) => return LRESULT(0),
        WM_SETFOCUS | WM_KILLFOCUS => {
            let _ = InvalidateRect(GetParent(hwnd), None, false);
        }
        _ => {}
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}

unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => return LRESULT(1),
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut paint);
            with_app(|a| {
                if let Some(p) = &a.user_panel {
                    let _ =
                        p.renderer
                            .paint_user_panel(dc, a.session.executing, GetFocus() == p.edit);
                }
            });
            let _ = EndPaint(hwnd, &paint);
            return LRESULT(0);
        }
        WM_CTLCOLOREDIT => {
            SetTextColor(HDC(wp.0 as isize), COLORREF(0x00e9eeec));
            SetBkColor(HDC(wp.0 as isize), COLORREF(0x00382924));
            return LRESULT(
                with_app(|a| a.user_panel.as_ref().map_or(0, |p| p.background.0)).unwrap_or(0),
            );
        }
        WM_COMMAND => {
            let main = with_app(|a| a.main_hwnd).unwrap_or_default();
            if wp.0 & 0xffff == 102 && (wp.0 >> 16) as u16 == BN_CLICKED as u16 {
                send(main);
            } else {
                let _ = InvalidateRect(hwnd, None, false);
            }
            return LRESULT(0);
        }
        WM_CLOSE => {
            let main = with_app(|a| a.main_hwnd).unwrap_or_default();
            hide(main);
            return LRESULT(0);
        }
        WM_NCHITTEST => {
            let hit = DefWindowProcW(hwnd, msg, wp, lp);
            if hit.0 == HTCLIENT as isize {
                let mut point = POINT {
                    x: lp.0 as i16 as i32,
                    y: (lp.0 >> 16) as i16 as i32,
                };
                let _ = ScreenToClient(hwnd, &mut point);
                let dpi = GetDpiForWindow(hwnd).max(96);
                if point.y < 80 * dpi as i32 / 96 {
                    return LRESULT(HTCAPTION as isize);
                }
            }
            return hit;
        }
        WM_DPICHANGED => {
            let suggested = *(lp.0 as *const RECT);
            let monitor = MonitorFromRect(&suggested, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(monitor, &mut info).as_bool() {
                with_app(|a| {
                    if let Some(p) = a.user_panel.as_mut() {
                        let work = Rect {
                            left: info.rcWork.left,
                            top: info.rcWork.top,
                            right: info.rcWork.right,
                            bottom: info.rcWork.bottom,
                        };
                        let _ = p.layout(work, (wp.0 as u16 as u32).max(96));
                        a.surfaces.avoid = Some(p.bounds);
                    }
                });
            }
            return LRESULT(0);
        }
        WM_EXITSIZEMOVE => {
            with_app(|a| {
                if let Some(p) = a.user_panel.as_mut() {
                    let mut r = RECT::default();
                    if GetWindowRect(hwnd, &mut r).is_ok() {
                        p.bounds = Rect {
                            left: r.left,
                            top: r.top,
                            right: r.right,
                            bottom: r.bottom,
                        };
                        a.surfaces.avoid = Some(p.bounds);
                    }
                }
            });
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

impl UserPanel {
    unsafe fn new(main: HWND, work: Rect, dpi: u32) -> Result<Self> {
        let instance = GetModuleHandleW(None)?;
        let class = w!("NaviUserPanel");
        let _ = RegisterClassW(&WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: class,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        });
        let renderer = Renderer::new(
            dpi,
            SurfaceSize {
                width: 540.0,
                height: 244.0,
            },
        )?;
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class,
            w!("Tu espacio — escribir a NAVI"),
            WS_POPUP | WS_CLIPCHILDREN,
            0,
            0,
            1,
            1,
            None,
            None,
            instance,
            None,
        );
        if hwnd.0 == 0 {
            return Err(Error::from_win32());
        }
        let mut p = Self {
            hwnd,
            edit: HWND(0),
            button: HWND(0),
            renderer,
            bounds: work,
            font: HFONT(0),
            background: HBRUSH(0),
            dpi,
            previous: HWND(0),
        };
        p.background = CreateSolidBrush(COLORREF(0x00382924));
        p.edit = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            None,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            0,
            0,
            1,
            1,
            hwnd,
            HMENU(101),
            instance,
            None,
        );
        p.button = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Enviar"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            0,
            0,
            1,
            1,
            hwnd,
            HMENU(102),
            instance,
            None,
        );
        if p.edit.0 == 0 || p.button.0 == 0 || p.background.0 == 0 {
            return Err(Error::from_win32());
        }
        if !SetWindowSubclass(p.edit, Some(input_proc), 1, main.0 as usize).as_bool()
            || !SetWindowSubclass(p.button, Some(input_proc), 1, main.0 as usize).as_bool()
        {
            return Err(Error::from_win32());
        }
        SendMessageW(
            p.edit,
            EM_SETLIMITTEXT,
            WPARAM(navi_agent::model::MAX_MESSAGE_CHARS),
            LPARAM(0),
        );
        let cue: Vec<u16> = "Ej.: abrí la calculadora"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        SendMessageW(
            p.edit,
            EM_SETCUEBANNER,
            WPARAM(1),
            LPARAM(cue.as_ptr() as isize),
        );
        p.layout(work, dpi)?;
        Ok(p)
    }

    unsafe fn layout(&mut self, work: Rect, dpi: u32) -> Result<()> {
        let scale = dpi as f32 / 96.0;
        let width = (540.0 * scale)
            .round()
            .min((work.right - work.left - 24).max(1) as f32) as i32;
        let height = (244.0 * scale)
            .round()
            .min((work.bottom - work.top - 24).max(1) as f32) as i32;
        let size = SurfaceSize {
            width: width as f32 / scale,
            height: height as f32 / scale,
        };
        self.renderer = Renderer::new(dpi, size)?;
        let font = CreateFontW(
            -(18.0 * scale).round() as i32,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );
        if font.0 == 0 {
            return Err(Error::from_win32());
        }
        SendMessageW(self.edit, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
        SendMessageW(self.button, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
        if self.font.0 != 0 {
            let _ = DeleteObject(self.font);
        }
        self.font = font;
        self.dpi = dpi;
        let x = work.left + (work.right - work.left - width) / 2;
        let y = (work.bottom - height - (36.0 * scale) as i32).max(work.top);
        self.bounds = Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };
        SetWindowPos(self.hwnd, HWND_TOPMOST, x, y, width, height, SWP_NOACTIVATE)?;
        SetWindowPos(
            self.edit,
            None,
            (29.0 * scale) as i32,
            (96.0 * scale) as i32,
            (width - (58.0 * scale) as i32).max(1),
            (32.0 * scale) as i32,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
        SetWindowPos(
            self.button,
            None,
            (width - (116.0 * scale) as i32).max(0),
            (150.0 * scale) as i32,
            (92.0 * scale) as i32,
            (33.0 * scale) as i32,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
        let _ = InvalidateRect(self.hwnd, None, false);
        Ok(())
    }

    pub(super) unsafe fn text(&self) -> String {
        let mut text = vec![0u16; navi_agent::model::MAX_MESSAGE_CHARS * 2 + 1];
        let length = GetWindowTextW(self.edit, &mut text).max(0) as usize;
        String::from_utf16_lossy(&text[..length])
    }
    pub(super) unsafe fn refresh(&self) {
        let _ = InvalidateRect(self.hwnd, None, false);
    }
}

impl Drop for UserPanel {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
            if self.font.0 != 0 {
                let _ = DeleteObject(self.font);
            }
            if self.background.0 != 0 {
                let _ = DeleteObject(self.background);
            }
        }
    }
}

pub(super) unsafe fn visible() -> bool {
    with_app(|a| {
        a.user_panel
            .as_ref()
            .is_some_and(|p| IsWindowVisible(p.hwnd).as_bool())
    })
    .unwrap_or(false)
}

pub(super) unsafe fn show(main: HWND, focus: bool) {
    let previous = GetForegroundWindow();
    let result = with_app(|a| -> Result<()> {
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(monitor, &mut info).ok()?;
        let mut dx = 96;
        let mut dy = 96;
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
        let work = Rect {
            left: info.rcWork.left,
            top: info.rcWork.top,
            right: info.rcWork.right,
            bottom: info.rcWork.bottom,
        };
        if a.user_panel.is_none() {
            a.user_panel = Some(UserPanel::new(main, work, dx.max(96))?);
        }
        let p = a.user_panel.as_mut().unwrap();
        p.layout(work, dx.max(96))?;
        if previous != p.hwnd {
            p.previous = previous;
        }
        a.surfaces.avoid = Some(p.bounds);
        focus_reaction::cancel(&mut a.reactions, &mut a.surfaces);
        SetWindowPos(
            p.hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )?;
        if focus {
            let _ = SetForegroundWindow(p.hwnd);
            SetFocus(p.edit);
        }
        p.refresh();
        Ok(())
    });
    if let Some(Err(error)) = result {
        let _ = std::fs::write(
            std::env::temp_dir().join("navi-input-error.log"),
            error.to_string(),
        );
    }
    commit(main, |_| {});
}

pub(super) unsafe fn hide(main: HWND) {
    with_app(|a| {
        if let Some(p) = &a.user_panel {
            let foreground = GetForegroundWindow();
            let _ = ShowWindow(p.hwnd, SW_HIDE);
            if foreground == p.hwnd && IsWindow(p.previous).as_bool() {
                let _ = SetForegroundWindow(p.previous);
            }
        }
        a.surfaces.avoid = None;
    });
    commit(main, |_| {});
}

pub(super) unsafe fn toggle(main: HWND) {
    if visible() {
        hide(main);
    } else {
        show(main, true);
    }
}

pub(super) unsafe fn send(main: HWND) {
    let text = with_app(|a| a.user_panel.as_ref().map(|p| p.text()))
        .flatten()
        .unwrap_or_default();
    if text.trim().is_empty() || with_app(|a| a.session.executing).unwrap_or(true) {
        return;
    }
    with_app(|a| a.session.input = text);
    with_app(|a| {
        if let Some(p) = &a.user_panel {
            let _ = SetWindowTextW(p.edit, w!(""));
            SetFocus(p.edit);
            p.refresh();
        }
    });
    text_backend::submit(main);
}
