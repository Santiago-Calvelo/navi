//! Finite result placement, frozen to the focused monitor at admission.
use super::*;
use navi_state::visual::variation_unit;

#[derive(Clone, Copy)]
pub(super) struct ActionSurface {
    pub work: Rect,
    pub dpi: u32,
    pub bounds: Rect,
    pub size: SurfaceSize,
    pub rotation: f32,
    pub scale: f32,
}

pub(super) unsafe fn focused() -> Option<(Rect, u32)> {
    let monitor = MonitorFromWindow(GetForegroundWindow(), MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        return None;
    }
    let (mut x, mut y) = (96, 96);
    let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
    Some((
        Rect {
            left: info.rcWork.left,
            top: info.rcWork.top,
            right: info.rcWork.right,
            bottom: info.rcWork.bottom,
        },
        x.max(96),
    ))
}

impl surface::SurfaceManager {
    pub(super) fn result_surface(&self, work: Rect, dpi: u32, seed: u64) -> ActionSurface {
        let density = dpi.max(1) as f32 / 96.0;
        let margin = (16.0 * density) as i32;
        let available = (work.right - work.left).min(work.bottom - work.top);
        let side = ((520.0 * density).min(available as f32 * 0.65) as i32).max(1);
        let x = work.left
            + margin
            + ((work.right - work.left - side - 2 * margin).max(0) as f32
                * variation_unit(seed, 71)) as i32;
        let y = work.top
            + margin
            + ((work.bottom - work.top - side - 2 * margin).max(0) as f32
                * variation_unit(seed, 73)) as i32;
        ActionSurface {
            work,
            dpi,
            bounds: Rect {
                left: x,
                top: y,
                right: x + side,
                bottom: y + side,
            },
            size: SurfaceSize {
                width: side as f32 / density,
                height: side as f32 / density,
            },
            rotation: variation_unit(seed, 79) * std::f32::consts::TAU,
            scale: side as f32 / density / 650.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn results_vary_reproducibly_inside_only_the_requested_monitor() {
        let manager = surface::SurfaceManager::default();
        for dpi in [96, 144, 192] {
            let work = Rect {
                left: -2560,
                top: -400,
                right: 0,
                bottom: 1040,
            };
            let a = manager.result_surface(work, dpi, 1);
            let b = manager.result_surface(work, dpi, 2);
            assert_ne!(a.bounds, b.bounds);
            assert_ne!(a.rotation, b.rotation);
            assert_eq!(a.bounds, manager.result_surface(work, dpi, 1).bounds);
            assert!(a.bounds.left >= work.left && a.bounds.right <= work.right);
            assert!(a.bounds.top >= work.top && a.bounds.bottom <= work.bottom);
        }
    }
}
