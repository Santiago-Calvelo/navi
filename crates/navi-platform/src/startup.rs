//! One passive startup manifestation; selection and persistence happen once.
use super::*;
use navi_state::startup::{PhraseRotation, DURATION_MS};
use windows::Win32::UI::Shell::*;

pub(super) struct Startup {
    phrase: &'static str,
    start: u64,
    seed: u64,
    monitor: u64,
    dpi: u32,
    bounds: Rect,
    size: SurfaceSize,
}

pub(super) unsafe fn begin(app: &mut App) {
    let monitor = MonitorFromWindow(GetForegroundWindow(), MONITOR_DEFAULTTONEAREST).0 as u64;
    monitor_catalog::prepare(app);
    if SHQueryUserNotificationState().ok().is_some_and(|s| {
        matches!(
            s,
            QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_BUSY | QUNS_PRESENTATION_MODE | QUNS_NOT_PRESENT
        )
    }) {
        return;
    }
    let Some(area) = app
        .surfaces
        .monitors
        .iter()
        .find(|m| m.id == monitor && !m.blocked)
    else {
        return;
    };
    let density = area.dpi as f32 / 96.0;
    let width = area.work.right - area.work.left;
    let height = area.work.bottom - area.work.top;
    let scale = density
        .min((width - 32) as f32 / 800.0)
        .min((height - 32) as f32 / 460.0);
    if scale <= 0.0 {
        return;
    }
    let w = (800.0 * scale).floor() as i32;
    let h = (460.0 * scale).floor() as i32;
    let left = area.work.left + (width - w) / 2;
    let top = area.work.top + (height - h) / 2;
    let path = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("NAVI").join("startup-phrases.txt"));
    let saved = path.as_ref().and_then(|p| {
        (std::fs::metadata(p).ok()?.len() <= 2048)
            .then(|| std::fs::read_to_string(p).ok())
            .flatten()
    });
    let mut rotation = saved
        .as_deref()
        .and_then(PhraseRotation::decode)
        .unwrap_or_default();
    let phrase = rotation.next(app.surfaces.session_seed);
    // This file contains phrase indices only, never user text or desktop activity.
    let persisted = path
        .as_ref()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "LOCALAPPDATA no disponible")
        })
        .and_then(|p| {
            std::fs::create_dir_all(p.parent().unwrap())?;
            std::fs::write(p, rotation.encode())
        });
    if let Err(error) = persisted {
        let _ = std::fs::write(
            std::env::temp_dir().join("navi-startup-error.log"),
            error.to_string(),
        );
    }
    app.startup = Some(Startup {
        phrase,
        start: app.now(),
        seed: app.surfaces.session_seed,
        monitor,
        dpi: area.dpi,
        bounds: Rect {
            left,
            top,
            right: left + w,
            bottom: top + h,
        },
        size: SurfaceSize {
            width: w as f32 / density,
            height: h as f32 / density,
        },
    });
}

/// True while startup owns the shared overlay. No Windows observation per frame.
pub(super) unsafe fn sync(hwnd: HWND, app: &mut App) -> bool {
    let now = app.now();
    let Some(startup) = &app.startup else {
        return false;
    };
    let elapsed = now.saturating_sub(startup.start);
    if elapsed >= DURATION_MS
        || app.surfaces.kind != SurfaceKind::Beacon
        || !app.surfaces.visible
        || !app.reactions.config.enabled
        || app.effects.frame(now).is_some()
        || !app
            .surfaces
            .monitors
            .iter()
            .any(|m| m.id == startup.monitor && !m.blocked)
    {
        app.startup = None;
        app.overlay = None;
        return false;
    }
    let result = (|| -> Result<()> {
        if app.overlay.is_none() {
            app.overlay = Some(holograms::Overlay::new(startup.dpi, startup.size)?);
        }
        let overlay = app.overlay.as_ref().unwrap();
        SetWindowPos(
            overlay.hwnd,
            HWND_TOPMOST,
            startup.bounds.left,
            startup.bounds.top,
            startup.bounds.right - startup.bounds.left,
            startup.bounds.bottom - startup.bounds.top,
            SWP_NOACTIVATE,
        )?;
        overlay.renderer.draw_startup(
            overlay.hwnd,
            startup.phrase,
            elapsed as f32 / DURATION_MS as f32,
            app.effects.motion,
            startup.seed,
        )?;
        if !IsWindowVisible(overlay.hwnd).as_bool() {
            SetWindowPos(
                overlay.hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
        }
        let remaining = (DURATION_MS - elapsed) as u32;
        if SetTimer(
            hwnd,
            4,
            if app.effects.motion {
                40.min(remaining)
            } else {
                remaining
            },
            None,
        ) == 0
        {
            return Err(Error::from_win32());
        }
        app.timers[3] = true;
        Ok(())
    })();
    if let Err(error) = result {
        app.startup = None;
        app.overlay = None;
        let _ = std::fs::write(
            std::env::temp_dir().join("navi-startup-error.log"),
            error.to_string(),
        );
        return false;
    }
    true
}
