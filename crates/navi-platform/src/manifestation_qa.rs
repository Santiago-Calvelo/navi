//! Own-window compositor diagnostics only; never synthesizes external input.
use super::*;
use navi_state::{
    holograms::EffectFrame,
    reactions::Intensity,
    visual::{Family, VisualStyle},
};

pub(super) unsafe fn verify(
    hwnd: HWND,
    path: &std::path::Path,
) -> Result<Vec<(&'static str, bool)>> {
    let work = monitor_work(hwnd)?;
    let factor = ((work.right - work.left) as f32 / 540.0)
        .min((work.bottom - work.top) as f32 / 300.0)
        .clamp(0.0, 1.0);
    let width = (540.0 * factor).floor().max(1.0) as i32;
    let height = (300.0 * factor).floor().max(1.0) as i32;
    let x = (work.left + 24).clamp(work.left, (work.right - width).max(work.left));
    let y = (work.top + 24).clamp(work.top, (work.bottom - height).max(work.top));
    let bounds = Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    let overlay = holograms::Overlay::new(
        96,
        SurfaceSize {
            width: width as f32,
            height: height as f32,
        },
    )?;
    let overlay_hwnd = overlay.hwnd;
    let foreground = GetForegroundWindow();
    let frame = EffectFrame {
        layers: 4,
        mask: navi_state::holograms::STACK,
        phase: Phase::Processing,
        progress: 0.4,
        opacity: 1.0,
        motion: true,
    };
    let points = [
        (width as f32 * 0.35, height as f32 * 0.4),
        (width as f32 * 0.7, height as f32 * 0.65),
    ];
    let mut checks = Vec::new();
    let mut endpoints_clear = true;
    let mut focus_preserved = true;
    let mut passthrough = true;
    let mut painted_target_is_external = true;
    let mut geometry_inside_work_area = true;

    for (family, name, alpha_check) in [
        (
            Family::StackV2,
            "layer-stack-v2-native.bmp",
            "native_stack_v2_three_intensities_have_real_alpha",
        ),
        (
            Family::FracturedHalo,
            "fractured-halo-native.bmp",
            "native_fractured_halo_three_intensities_have_real_alpha",
        ),
        (
            Family::ClickPulse,
            "click-pulse-native.bmp",
            "native_click_pulse_three_intensities_have_real_alpha",
        ),
        (
            Family::DragConnector,
            "drag-connector-native.bmp",
            "native_drag_connector_three_intensities_have_real_alpha",
        ),
    ] {
        let mut transparent_and_visible = true;
        for intensity in [
            Intensity::Subtle,
            Intensity::Experimental,
            Intensity::Anomalous,
        ] {
            let style = VisualStyle {
                intensity,
                seed: 17,
            };
            overlay.manifestation(frame, bounds, family, style, points)?;
            let (zero, partial, opaque) = overlay.renderer.alpha_stats();
            transparent_and_visible &= zero > 0 && partial > 0 && opaque == 0;
            if intensity == Intensity::Experimental {
                let snapshot_path = path.with_file_name(name);
                overlay
                    .renderer
                    .snapshot(&snapshot_path)
                    .map_err(|error| Error::new(E_FAIL, error.to_string()))?;
                // This diagnostic BMP is the actual top-down premultiplied DIB.
                // Choose its most opaque painted pixel, not an empty margin or
                // an assumed geometric edge. This does not move the real cursor.
                let bitmap = std::fs::read(&snapshot_path)
                    .map_err(|error| Error::new(E_FAIL, error.to_string()))?;
                let painted = bitmap
                    .get(54..)
                    .and_then(|pixels| {
                        pixels
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .enumerate()
                            .max_by_key(|(_, pixel)| pixel[3])
                    })
                    .filter(|(_, pixel)| pixel[3] > 0);
                if let Some((index, _)) = painted {
                    let point = POINT {
                        x: bounds.left + index as i32 % width,
                        y: bounds.top + index as i32 / width,
                    };
                    let target = WindowFromPoint(point);
                    let root = GetAncestor(target, GA_ROOT);
                    let mut pid = 0;
                    GetWindowThreadProcessId(root, Some(&mut pid));
                    // CursorObserver uses this same point->root->PID identity
                    // boundary to ignore NAVI input. Decoration must not make
                    // a click on the underlying application look like our own.
                    painted_target_is_external &= target.0 != 0
                        && target != overlay.hwnd
                        && pid != 0
                        && pid != std::process::id();
                } else {
                    painted_target_is_external = false;
                }
            }
            let mut actual = RECT::default();
            GetWindowRect(overlay.hwnd, &mut actual)?;
            geometry_inside_work_area &= actual.left == bounds.left
                && actual.top == bounds.top
                && actual.right == bounds.right
                && actual.bottom == bounds.bottom
                && actual.left >= work.left
                && actual.top >= work.top
                && actual.right <= work.right
                && actual.bottom <= work.bottom;
            let flags = GetWindowLongPtrW(overlay.hwnd, GWL_EXSTYLE) as u32;
            let required = WS_EX_LAYERED.0 | WS_EX_NOACTIVATE.0 | WS_EX_TRANSPARENT.0;
            // Sample a pulse edge or a connector anchor, as well as the spatial
            // families' center. The flags apply to the complete surface.
            let point = if matches!(family, Family::ClickPulse | Family::DragConnector) {
                POINT {
                    x: bounds.left + points[0].0 as i32 + 14,
                    y: bounds.top + points[0].1 as i32 + 4,
                }
            } else {
                POINT {
                    x: bounds.left + width / 2,
                    y: bounds.top + height / 2,
                }
            };
            passthrough &= flags & required == required && WindowFromPoint(point) != overlay.hwnd;
            focus_preserved &= GetForegroundWindow() == foreground;
        }
        checks.push((alpha_check, transparent_and_visible));
        for progress in [0.0, 1.0] {
            overlay.manifestation(
                EffectFrame {
                    progress,
                    opacity: 0.0,
                    ..frame
                },
                bounds,
                family,
                VisualStyle::default(),
                points,
            )?;
            let (zero, partial, opaque) = overlay.renderer.alpha_stats();
            endpoints_clear &= zero > 0 && partial == 0 && opaque == 0;
        }
    }
    checks.push((
        "native_manifestation_lifetime_endpoints_clear",
        endpoints_clear,
    ));
    checks.push((
        "native_manifestation_bounds_stay_in_work_area",
        geometry_inside_work_area,
    ));
    checks.push(("native_manifestations_are_click_through", passthrough));
    checks.push((
        "native_painted_geometry_does_not_mask_input_target",
        painted_target_is_external,
    ));
    checks.push(("native_manifestations_preserve_foreground", focus_preserved));
    drop(overlay);
    checks.push((
        "native_manifestation_drop_destroys_hwnd",
        !IsWindow(overlay_hwnd).as_bool(),
    ));
    Ok(checks)
}
