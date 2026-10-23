use super::{
    model::{caret_line, lines, Model, Phase},
    view::{self, Action},
};
use crate::{render::Renderer, Options};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    time::Instant,
};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, DataExchange::*, LibraryLoader::*, Memory::*},
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};

const DISPATCH: u32 = WM_APP + 71;
thread_local! {
    static APP:RefCell<Option<App>>=const{RefCell::new(None)};
    static QUEUE:RefCell<VecDeque<(u32,WPARAM,LPARAM)>>=const{RefCell::new(VecDeque::new())};
    static GEOMETRY_DIRTY:Cell<bool>=const{Cell::new(false)};
    static PREVIOUS_FOREGROUND:Cell<isize>=const{Cell::new(0)};
}
struct Com;
impl Drop for Com {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
#[derive(Clone, Copy)]
enum Drag {
    Editor,
    Scroll,
}
struct App {
    model: Model,
    renderer: Renderer,
    origin: Instant,
    expanded: bool,
    opening: f32,
    transition: f32,
    transition_from: f32,
    focused: bool,
    scale: f32,
    hover: Action,
    keyboard: Action,
    drag: Option<Drag>,
    surrogate: Option<u16>,
    reduced: bool,
    tour: bool,
    tour_origin: f32,
    tour_step: usize,
    smoke: bool,
    failure: Option<Error>,
}

unsafe fn info(monitor: HMONITOR) -> Result<MONITORINFO> {
    let mut m = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut m).as_bool() {
        return Err(Error::from_win32());
    }
    Ok(m)
}
unsafe extern "system" fn enumerate(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    if let Ok(m) = info(monitor) {
        (&mut *(data.0 as *mut Vec<(HMONITOR, bool)>)).push((monitor, m.dwFlags & 1 != 0));
    }
    BOOL(1)
}
unsafe fn geometry(hwnd: HWND, large: bool) -> Result<(Renderer, f32)> {
    let work = info(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST))?.rcWork;
    let dpi = GetDpiForWindow(hwnd).max(96) as f32 / 96.0;
    let scale = dpi
        .min((work.right - work.left) as f32 / view::W)
        .min((work.bottom - work.top) as f32 / view::H);
    let (w, h) = if large {
        (view::W, view::H)
    } else {
        (view::CW, view::CH)
    };
    let width = (w * scale).round().max(1.0) as i32;
    let height = (h * scale).round().max(1.0) as i32;
    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        work.right - width,
        work.top + ((work.bottom - work.top) - height) / 2,
        width,
        height,
        SWP_NOACTIVATE,
    )?;
    Ok((Renderer::canvas(width, height, w, h)?, scale))
}
fn local(lp: LPARAM, scale: f32) -> (f32, f32) {
    (
        (lp.0 as u16 as i16) as f32 / scale,
        ((lp.0 >> 16) as u16 as i16) as f32 / scale,
    )
}
unsafe fn pressed(key: VIRTUAL_KEY) -> bool {
    GetKeyState(key.0 as i32) < 0
}

struct Clipboard;
impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}
unsafe fn paste(hwnd: HWND) -> Result<String> {
    OpenClipboard(hwnd)?;
    let _close = Clipboard;
    let handle = GetClipboardData(13)?;
    let global = HGLOBAL(handle.0 as *mut _);
    let count = (GlobalSize(global) / 2).min(16384);
    let ptr = GlobalLock(global) as *const u16;
    if ptr.is_null() {
        return Err(Error::from_win32());
    }
    let slice = std::slice::from_raw_parts(ptr, count);
    let end = slice.iter().position(|&v| v == 0).unwrap_or(count);
    let text = String::from_utf16_lossy(&slice[..end])
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let _ = GlobalUnlock(global);
    Ok(text)
}
unsafe fn copy(hwnd: HWND, text: &str) -> Result<()> {
    let data: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let memory = GlobalAlloc(GMEM_MOVEABLE, data.len() * 2)?;
    let ptr = GlobalLock(memory) as *mut u16;
    if ptr.is_null() {
        let _ = GlobalFree(memory);
        return Err(Error::from_win32());
    }
    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
    let _ = GlobalUnlock(memory);
    if let Err(e) = OpenClipboard(hwnd) {
        let _ = GlobalFree(memory);
        return Err(e);
    }
    let _close = Clipboard;
    if let Err(e) = EmptyClipboard() {
        let _ = GlobalFree(memory);
        return Err(e);
    }
    if let Err(e) = SetClipboardData(13, HANDLE(memory.0 as isize)) {
        let _ = GlobalFree(memory);
        return Err(e);
    }
    Ok(()) // Windows now owns the allocation.
}

impl App {
    fn now(&self) -> f32 {
        self.origin.elapsed().as_secs_f32()
    }
    fn large(&self) -> bool {
        self.expanded || self.opening > 0.0
    }
    fn take_control(&mut self) {
        self.tour = false;
    }
    unsafe fn show(&mut self, hwnd: HWND, open: bool, user: bool) -> Result<()> {
        if self.expanded == open {
            return Ok(());
        }
        self.transition_from = self.opening;
        self.transition = self.now();
        self.expanded = open;
        if open {
            (self.renderer, self.scale) = geometry(hwnd, true)?;
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                (WS_EX_LAYERED | WS_EX_TOOLWINDOW).0 as isize,
            );
            if user {
                let previous = GetForegroundWindow();
                if previous != hwnd {
                    PREVIOUS_FOREGROUND.with(|saved| saved.set(previous.0));
                }
                let _ = SetForegroundWindow(hwnd);
                let _ = SetFocus(hwnd);
            }
        } else {
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                (WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE).0 as isize,
            );
            if GetForegroundWindow() == hwnd {
                let previous = HWND(PREVIOUS_FOREGROUND.with(|saved| saved.get()));
                if previous.0 != 0 && IsWindow(previous).as_bool() {
                    let _ = SetForegroundWindow(previous);
                }
            }
        }
        if self.reduced {
            self.opening = if open { 1.0 } else { 0.0 };
            if !open {
                (self.renderer, self.scale) = geometry(hwnd, false)?;
            }
        }
        Ok(())
    }
    unsafe fn act(&mut self, hwnd: HWND, action: Action) -> Result<()> {
        let now = self.now();
        match action {
            Action::Open => self.show(hwnd, true, true)?,
            Action::Fold => self.show(hwnd, false, false)?,
            Action::Quit => {
                PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0))?;
            }
            Action::Tour => {
                if self.model.phase != Phase::Quiet || !self.model.editor.chars.is_empty() {
                    self.model.notice =
                        "Terminá el paso actual y dejá el borrador vacío para recorrer la demo."
                            .into();
                } else {
                    self.tour = true;
                    self.tour_origin = now;
                    self.tour_step = 0;
                }
            }
            Action::History => {
                self.model.history();
                self.model.scroll = view::max_scroll(&self.model);
            }
            Action::Copy => {
                let text = self
                    .model
                    .messages
                    .iter()
                    .map(|m| format!("{}\n{}", if m.user { "VOS" } else { "NAVI" }, m.text))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                if copy(hwnd, &text).is_ok() {
                    self.model.notice = "El hilo se copió al portapapeles.".into();
                } else {
                    self.model.notice = "No se pudo abrir el portapapeles. Probá de nuevo.".into();
                }
            }
            Action::Cancel => self.model.cancel(now),
            Action::Allow => self.model.decide(true, now),
            Action::Deny => self.model.decide(false, now),
            Action::Latest => {
                self.model.scroll = 0.0;
                self.model.unread = false;
            }
            Action::Jump(index) => {
                let l = view::layout(&self.model);
                if let Some(y) = l.offsets.get(index) {
                    self.model.scroll = (view::max_scroll(&self.model) - y).max(0.0);
                }
            }
            _ => {}
        }
        Ok(())
    }
    unsafe fn tour_tick(&mut self, hwnd: HWND, now: f32) -> Result<()> {
        if !self.tour {
            return Ok(());
        }
        let t = now - self.tour_origin;
        let times = [1.0, 3.0, 5.0, 9.0, 11.0, 15.0, 18.0, 20.0, 23.0, 25.0, 28.0];
        while self.tour_step < times.len() && t >= times[self.tour_step] {
            match self.tour_step {
                0 => self.show(hwnd, true, false)?,
                1 => self.model.editor.set("Hola. Quiero leer algo largo."),
                2 => self.model.submit(now),
                3 => self.model.editor.set("Guardar este hilo."),
                4 => self.model.submit(now),
                5 => self.model.decide(true, now),
                6 => self.model.history(),
                7 => self.model.scroll = view::max_scroll(&self.model) * 0.65,
                8 => {
                    self.model.editor.set("Podemos seguir desde acá mañana.");
                    self.show(hwnd, false, false)?;
                }
                9 => self.show(hwnd, true, false)?,
                _ => {
                    self.tour = false;
                    self.model.notice =
                        "Tu turno. El borrador quedó intacto al retirar y volver.".into();
                }
            }
            self.tour_step += 1;
        }
        Ok(())
    }
    unsafe fn key(&mut self, hwnd: HWND, key: u16) -> Result<()> {
        let ctrl = pressed(VK_CONTROL);
        let shift = pressed(VK_SHIFT);
        if key == VK_ESCAPE.0 {
            self.show(hwnd, false, false)?;
            return Ok(());
        }
        if key == VK_TAB.0 {
            let mut actions = vec![Action::Input, Action::History, Action::Copy, Action::Tour];
            if self.model.phase == Phase::Permission {
                actions.extend([Action::Allow, Action::Deny]);
            }
            if self.model.phase == Phase::Thinking {
                actions.push(Action::Cancel);
            }
            actions.extend([Action::Fold, Action::Quit]);
            let current = actions
                .iter()
                .position(|x| *x == self.keyboard)
                .unwrap_or(0);
            self.keyboard =
                actions[(current + if shift { actions.len() - 1 } else { 1 }) % actions.len()];
            self.hover = self.keyboard;
            return Ok(());
        }
        if key == VK_PRIOR.0 || key == VK_NEXT.0 {
            self.model.scroll = (self.model.scroll
                + if key == VK_PRIOR.0 { 260.0 } else { -260.0 })
            .clamp(0.0, view::max_scroll(&self.model));
            return Ok(());
        }
        if ctrl && key == VK_HOME.0 {
            self.model.scroll = view::max_scroll(&self.model);
            return Ok(());
        }
        if ctrl && key == VK_END.0 {
            self.model.scroll = 0.0;
            self.model.unread = false;
            return Ok(());
        }
        if ctrl {
            match key {
                0x41 => {
                    self.model.editor.anchor = 0;
                    self.model.editor.caret = self.model.editor.chars.len();
                }
                0x43 | 0x58 => {
                    let selected = self.model.editor.selected();
                    if !selected.is_empty() && copy(hwnd, &selected).is_ok() && key == 0x58 {
                        self.model.editor.erase(true);
                    }
                }
                0x56 => match paste(hwnd) {
                    Ok(text) => self.model.editor.insert(&text),
                    Err(_) => {
                        self.model.notice = "No hay texto disponible en el portapapeles.".into()
                    }
                },
                _ => {}
            }
            return Ok(());
        }
        let editor = &mut self.model.editor;
        match key {
            k if k == VK_LEFT.0 => editor.move_to(
                if !shift && !editor.selection().is_empty() {
                    editor.selection().start
                } else {
                    editor.caret.saturating_sub(1)
                },
                shift,
            ),
            k if k == VK_RIGHT.0 => editor.move_to(
                if !shift && !editor.selection().is_empty() {
                    editor.selection().end
                } else {
                    editor.caret + 1
                },
                shift,
            ),
            k if k == VK_DELETE.0 => editor.erase(false),
            k if k == VK_HOME.0 || k == VK_END.0 || k == VK_UP.0 || k == VK_DOWN.0 => {
                let rows = lines(&editor.chars, view::EDIT_COLS);
                let row = caret_line(&rows, editor.caret);
                let col = editor.caret - rows[row].start;
                let pos = if k == VK_HOME.0 {
                    rows[row].start
                } else if k == VK_END.0 {
                    rows[row].end
                } else {
                    let next = if k == VK_UP.0 {
                        row.saturating_sub(1)
                    } else {
                        (row + 1).min(rows.len() - 1)
                    };
                    (rows[next].start + col).min(rows[next].end)
                };
                editor.move_to(pos, shift);
            }
            _ => {}
        }
        Ok(())
    }
    unsafe fn input(&mut self, hwnd: HWND, value: u16) -> Result<()> {
        if value == 13 {
            if self.keyboard != Action::Input {
                self.act(hwnd, self.keyboard)?;
                self.keyboard = Action::Input;
                self.hover = Action::None;
            } else if pressed(VK_SHIFT) {
                self.model.editor.insert("\n");
            } else {
                self.model.submit(self.now());
            }
            return Ok(());
        }
        if value == 8 {
            self.model.editor.erase(true);
            return Ok(());
        }
        if value < 32 {
            return Ok(());
        }
        self.keyboard = Action::Input;
        self.hover = Action::None;
        if (0xD800..=0xDBFF).contains(&value) {
            self.surrogate = Some(value);
            return Ok(());
        }
        let code = if (0xDC00..=0xDFFF).contains(&value) {
            let Some(high) = self.surrogate.take() else {
                return Ok(());
            };
            0x10000 + (((high as u32 - 0xD800) << 10) | (value as u32 - 0xDC00))
        } else {
            self.surrogate = None;
            value as u32
        };
        if let Some(ch) = char::from_u32(code) {
            self.model.editor.insert(&ch.to_string());
        }
        Ok(())
    }
    unsafe fn event(&mut self, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Result<()> {
        if matches!(msg, WM_KEYDOWN | WM_CHAR | WM_LBUTTONDOWN | WM_MOUSEWHEEL) {
            self.take_control();
        }
        match msg {
            WM_SETFOCUS => self.focused = true,
            WM_KILLFOCUS => {
                self.focused = false;
                self.surrogate = None;
                if self.drag.take().is_some() {
                    let _ = ReleaseCapture();
                }
            }
            WM_KEYDOWN if self.expanded => self.key(hwnd, wp.0 as u16)?,
            WM_CHAR if self.expanded => self.input(hwnd, wp.0 as u16)?,
            WM_LBUTTONDOWN => {
                let (x, y) = local(lp, self.scale);
                let a = view::hit(&self.model, self.large(), x, y);
                self.keyboard = if matches!(
                    a,
                    Action::None | Action::Input | Action::Scroll | Action::Jump(_)
                ) {
                    Action::Input
                } else {
                    a
                };
                if a == Action::Input {
                    let pos = view::editor_position(&self.model, x, y);
                    self.model.editor.move_to(pos, pressed(VK_SHIFT));
                    self.drag = Some(Drag::Editor);
                    SetCapture(hwnd);
                } else if a == Action::Scroll {
                    self.drag = Some(Drag::Scroll);
                    SetCapture(hwnd);
                    self.scroll_to(y);
                } else {
                    self.act(hwnd, a)?;
                    self.keyboard = Action::Input;
                }
            }
            WM_LBUTTONUP => {
                if self.drag.take().is_some() {
                    let _ = ReleaseCapture();
                }
            }
            WM_MOUSEMOVE => {
                let (x, y) = local(lp, self.scale);
                self.hover = view::hit(&self.model, self.large(), x, y);
                if let Some(drag) = self.drag {
                    match drag {
                        Drag::Editor => {
                            let pos = view::editor_position(&self.model, x, y);
                            self.model.editor.move_to(pos, true);
                        }
                        Drag::Scroll => self.scroll_to(y),
                    }
                }
            }
            WM_MOUSEWHEEL if self.expanded => {
                let delta = ((wp.0 >> 16) as u16 as i16) as f32 / 120.0;
                self.model.scroll =
                    (self.model.scroll + delta * 85.0).clamp(0.0, view::max_scroll(&self.model));
                if self.model.scroll < 1.0 {
                    self.model.unread = false;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn scroll_to(&mut self, y: f32) {
        self.model.scroll = view::max_scroll(&self.model)
            * (1.0 - ((y - view::TOP) / (view::BOTTOM - view::TOP)).clamp(0.0, 1.0));
    }
    unsafe fn render(&mut self, hwnd: HWND) -> Result<()> {
        let now = self.now();
        self.model.tick(now);
        self.tour_tick(hwnd, now)?;
        let goal = if self.expanded { 1.0 } else { 0.0 };
        let previous = self.opening;
        let p = if self.reduced {
            1.0
        } else {
            crate::experiment::smooth(0.0, 0.65, now - self.transition)
        };
        self.opening = self.transition_from + (goal - self.transition_from) * p;
        if !self.expanded && previous > 0.0 && self.opening <= 0.0 {
            (self.renderer, self.scale) = geometry(hwnd, false)?;
        }
        if GEOMETRY_DIRTY.with(|dirty| dirty.replace(false)) {
            (self.renderer, self.scale) = geometry(hwnd, self.large())?;
        }
        self.model.scroll = self.model.scroll.clamp(0.0, view::max_scroll(&self.model));
        let frame = if self.large() {
            view::expanded(
                &self.model,
                self.opening,
                now,
                self.focused,
                self.hover,
                self.tour,
                self.reduced,
            )
        } else {
            view::compact(&self.model, self.hover)
        };
        self.renderer.draw(hwnd, &frame)?;
        let _ = KillTimer(hwnd, 1);
        let animating = (self.opening - goal).abs() > 0.001
            || (!self.reduced
                && self.expanded
                && (self.model.phase == Phase::Thinking || now - self.model.reveal < 0.85));
        let interval = if animating {
            Some(16)
        } else if self.tour {
            Some(80)
        } else if self.model.phase == Phase::Thinking {
            Some(((self.model.due - now) * 1000.0).ceil().max(1.0) as u32)
        } else {
            None
        };
        if let Some(ms) = interval {
            if SetTimer(hwnd, 1, ms, None) == 0 {
                return Err(Error::from_win32());
            }
        }
        if self.smoke && now >= 29.0 {
            PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0))?;
        } else if self.smoke && interval.is_none() {
            let _ = SetTimer(hwnd, 1, 300, None);
        }
        Ok(())
    }
}

unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => {
            let mut p = POINT {
                x: lp.0 as u16 as i16 as i32,
                y: (lp.0 >> 16) as u16 as i16 as i32,
            };
            let _ = ScreenToClient(hwnd, &mut p);
            let hit = APP.with(|cell| {
                cell.try_borrow()
                    .ok()
                    .and_then(|app| {
                        app.as_ref().map(|a| {
                            let x = p.x as f32 / a.scale;
                            if a.large() {
                                x > 180.0 || x > 1003.0 - 838.0 * a.opening + 40.0
                            } else {
                                x > 35.0
                            }
                        })
                    })
                    .unwrap_or(false)
            });
            return LRESULT(if hit {
                HTCLIENT as isize
            } else {
                HTTRANSPARENT as isize
            });
        }
        WM_MOUSEACTIVATE => {
            let previous = GetForegroundWindow();
            if previous != hwnd {
                PREVIOUS_FOREGROUND.with(|saved| saved.set(previous.0));
            }
            let active = APP.with(|cell| {
                cell.try_borrow()
                    .ok()
                    .and_then(|a| a.as_ref().map(|a| a.expanded))
                    .unwrap_or(false)
            });
            return LRESULT(if active { MA_ACTIVATE } else { MA_NOACTIVATE } as isize);
        }
        WM_SETCURSOR => {
            let text = APP.with(|cell| {
                cell.try_borrow()
                    .ok()
                    .and_then(|a| a.as_ref().map(|a| a.hover == Action::Input))
                    .unwrap_or(false)
            });
            if let Ok(cursor) = LoadCursorW(None, if text { IDC_IBEAM } else { IDC_ARROW }) {
                SetCursor(cursor);
            }
            return LRESULT(1);
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _ = BeginPaint(hwnd, &mut ps);
            let _ = EndPaint(hwnd, &ps);
            return LRESULT(0);
        }
        WM_ERASEBKGND => return LRESULT(1),
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            return LRESULT(0);
        }
        WM_DESTROY => {
            let _ = KillTimer(hwnd, 1);
            PostQuitMessage(0);
            return LRESULT(0);
        }
        WM_DPICHANGED | WM_DISPLAYCHANGE => {
            GEOMETRY_DIRTY.with(|d| d.set(true));
            let _ = PostMessageW(hwnd, DISPATCH, WPARAM(0), LPARAM(0));
            return LRESULT(0);
        }
        DISPATCH => {
            APP.with(|cell| {
                let mut holder = cell.borrow_mut();
                let Some(app) = holder.as_mut() else {
                    return;
                };
                let events = QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()));
                let result = (|| -> Result<()> {
                    for (m, w, l) in events {
                        app.event(hwnd, m, w, l)?;
                    }
                    app.render(hwnd)
                })();
                if let Err(e) = result {
                    app.failure = Some(e);
                    let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                }
            });
            return LRESULT(0);
        }
        WM_KEYDOWN | WM_CHAR | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_MOUSEMOVE | WM_MOUSEWHEEL
        | WM_SETFOCUS | WM_KILLFOCUS | WM_TIMER => {
            if msg == WM_TIMER {
                let _ = KillTimer(hwnd, 1);
            }
            QUEUE.with(|q| q.borrow_mut().push_back((msg, wp, lp)));
            let _ = PostMessageW(hwnd, DISPATCH, WPARAM(0), LPARAM(0));
            return LRESULT(0);
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

pub fn run(options: Options) -> Result<()> {
    unsafe {
        println!("NAVI / Margen. Click the incision at the right edge to write. Enter continues, Escape folds. All responses and actions are simulated.");
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let _com = Com;
        let mut monitors: Vec<(HMONITOR, bool)> = Vec::new();
        if !EnumDisplayMonitors(
            None,
            None,
            Some(enumerate),
            LPARAM(&mut monitors as *mut _ as isize),
        )
        .as_bool()
        {
            return Err(Error::from_win32());
        }
        monitors.sort_by_key(|m| !m.1);
        let monitor = monitors
            .get(options.monitor)
            .ok_or_else(|| Error::new(E_INVALIDARG, "Monitor unavailable"))?
            .0;
        let work = info(monitor)?.rcWork;
        let instance = GetModuleHandleW(None)?;
        let class = w!("NaviGenesisMargen");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: class,
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            return Err(Error::from_win32());
        }
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            w!("NAVI — Margen / prototipo"),
            WS_POPUP,
            work.right - 10,
            work.top + 30,
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
        let result = (|| -> Result<()> {
            let (renderer, scale) = geometry(hwnd, false)?;
            APP.with(|cell| {
                *cell.borrow_mut() = Some(App {
                    model: Model::new(),
                    renderer,
                    origin: Instant::now(),
                    expanded: false,
                    opening: 0.0,
                    transition: -1.0,
                    transition_from: 0.0,
                    focused: false,
                    scale,
                    hover: Action::None,
                    keyboard: Action::Input,
                    drag: None,
                    surrogate: None,
                    reduced: options.reduced,
                    tour: options.tour,
                    tour_origin: 0.0,
                    tour_step: 0,
                    smoke: options.smoke,
                    failure: None,
                })
            });
            PostMessageW(hwnd, DISPATCH, WPARAM(0), LPARAM(0))?;
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
            let mut msg = MSG::default();
            loop {
                let status = GetMessageW(&mut msg, None, 0, 0).0;
                if status == 0 {
                    break;
                }
                if status == -1 {
                    return Err(Error::from_win32());
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            APP.with(|cell| {
                let mut app = cell.borrow_mut();
                match app.as_mut().and_then(|a| a.failure.take()) {
                    Some(e) => Err(e),
                    None => Ok(()),
                }
            })
        })();
        if IsWindow(hwnd).as_bool() {
            let _ = DestroyWindow(hwnd);
        }
        APP.with(|cell| {
            cell.borrow_mut().take();
        });
        QUEUE.with(|q| q.borrow_mut().clear());
        let _ = UnregisterClassW(class, instance);
        result
    }
}
