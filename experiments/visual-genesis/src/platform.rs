use crate::{
    experiment::{self, CATALOG, HEIGHT, WIDTH},
    render::Renderer,
    Options,
};
use std::{cell::Cell, time::Instant};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, LibraryLoader::*},
        UI::{HiDpi::*, WindowsAndMessaging::*},
    },
};

struct Com;
impl Drop for Com {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
struct Window(HWND);
impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if IsWindow(self.0).as_bool() {
                let _ = DestroyWindow(self.0);
            }
        }
    }
}

struct App {
    options: Options,
    renderer: Renderer,
    started: Instant,
    current: Option<usize>,
    error: Option<Error>,
}

// SetWindowPos may synchronously reenter the procedure. Keep its notification
// flag outside App so no second mutable App reference can be created.
thread_local! { static REFLOW: Cell<bool> = const { Cell::new(false) }; }

unsafe extern "system" fn enumerate(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    let monitors = &mut *(data.0 as *mut Vec<(HMONITOR, bool)>);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(monitor, &mut info).as_bool() {
        monitors.push((monitor, info.dwFlags & 1 != 0));
    }
    BOOL(1)
}

unsafe fn work(monitor: HMONITOR) -> Result<RECT> {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        return Err(Error::from_win32());
    }
    Ok(info.rcWork)
}

unsafe fn layout(hwnd: HWND) -> Result<Renderer> {
    let r = work(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST))?;
    let dpi = GetDpiForWindow(hwnd).max(96) as f32 / 96.0;
    let scale = dpi
        .min((r.right - r.left) as f32 * 0.92 / WIDTH)
        .min((r.bottom - r.top) as f32 * 0.92 / HEIGHT);
    let width = (WIDTH * scale).round().max(1.0) as i32;
    let height = (HEIGHT * scale).round().max(1.0) as i32;
    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        r.left + ((r.right - r.left) - width) / 2,
        r.top + ((r.bottom - r.top) - height) / 2,
        width,
        height,
        SWP_NOACTIVATE,
    )?;
    Renderer::new(width, height)
}

impl App {
    unsafe fn tick(&mut self, hwnd: HWND) -> Result<bool> {
        let elapsed = self.started.elapsed().as_secs_f32();
        let cycle = self.options.seconds + 1.0;
        let slot = (elapsed / cycle) as usize;
        if slot >= self.options.selection.len() {
            return Ok(false);
        }
        let local = elapsed - slot as f32 * cycle;
        if local >= self.options.seconds {
            ShowWindow(hwnd, SW_HIDE);
            self.current = None;
            let ms = ((cycle - local) * 1000.0).ceil().max(1.0) as u32;
            if SetTimer(hwnd, 1, ms, None) == 0 {
                return Err(Error::from_win32());
            }
            return Ok(true);
        }
        let index = self.options.selection[slot];
        let fresh = self.current != Some(slot);
        if fresh {
            println!(
                "{} / {} / {:.0}s",
                CATALOG[index].id, CATALOG[index].name, self.options.seconds
            );
        }
        if REFLOW.with(|flag| flag.replace(false)) {
            self.renderer = layout(hwnd)?;
        }
        let frame = experiment::frame(index, local / self.options.seconds, self.options.reduced);
        // A failed render gets one resource recreation, not a retry loop.
        if self.renderer.draw(hwnd, &frame).is_err() {
            self.renderer = layout(hwnd)?;
            self.renderer.draw(hwnd, &frame)?;
        }
        if fresh {
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
        }
        self.current = Some(slot);
        let interval = if self.options.reduced {
            ((self.options.seconds - local) * 1000.0).ceil().max(1.0) as u32
        } else {
            16
        };
        if SetTimer(hwnd, 1, interval, None) == 0 {
            return Err(Error::from_win32());
        }
        Ok(true)
    }
}

unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => return LRESULT(HTTRANSPARENT as isize),
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_ERASEBKGND => return LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _ = BeginPaint(hwnd, &mut ps);
            let _ = EndPaint(hwnd, &ps);
            return LRESULT(0);
        }
        WM_DESTROY => {
            let _ = KillTimer(hwnd, 1);
            PostQuitMessage(0);
            return LRESULT(0);
        }
        WM_DPICHANGED | WM_DISPLAYCHANGE => {
            REFLOW.with(|flag| flag.set(true));
            let _ = SetTimer(hwnd, 1, 1, None);
            return LRESULT(0);
        }
        _ => {}
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
    if !ptr.is_null() && msg == WM_TIMER {
        let _ = KillTimer(hwnd, 1);
        match (*ptr).tick(hwnd) {
            Ok(true) => {}
            Ok(false) => {
                let _ = DestroyWindow(hwnd);
            }
            Err(e) => {
                (*ptr).error = Some(e);
                let _ = DestroyWindow(hwnd);
            }
        }
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

pub fn run(options: Options) -> Result<()> {
    unsafe {
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
        monitors.sort_by_key(|(_, primary)| !primary);
        let monitor = monitors
            .get(options.monitor)
            .ok_or_else(|| Error::new(E_INVALIDARG, "Monitor index unavailable; 0 is primary"))?
            .0;
        let r = work(monitor)?;
        let instance = GetModuleHandleW(None)?;
        let class = w!("NaviVisualGenesisStandalone");
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
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            class,
            w!("NAVI / Visual Genesis"),
            WS_POPUP,
            r.left + (r.right - r.left) / 2,
            r.top + (r.bottom - r.top) / 2,
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
        let window = Window(hwnd);
        let renderer = layout(hwnd)?;
        let mut app = Box::new(App {
            options,
            renderer,
            started: Instant::now(),
            current: None,
            error: None,
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, (&mut *app) as *mut App as isize);
        if let Err(e) = app.tick(hwnd) {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return Err(e);
        }
        let mut msg = MSG::default();
        loop {
            let result = GetMessageW(&mut msg, None, 0, 0).0;
            if result == 0 {
                break;
            }
            if result == -1 {
                app.error = Some(Error::from_win32());
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if IsWindow(hwnd).as_bool() {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        }
        let error = app.error.take();
        drop(window);
        drop(app);
        let _ = UnregisterClassW(class, instance);
        match error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}
