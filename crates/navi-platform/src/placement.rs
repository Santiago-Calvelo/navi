//! Pure physical-screen placement. DPI only bounds the logical visual size;
//! monitor origin and available physical area remain authoritative.
use navi_state::{
    reactions::{Bounds, DesktopEvent},
    visual::Placement,
};

fn random(seed: u64, stream: u64) -> f64 {
    let mut x = seed.wrapping_add(stream.wrapping_mul(0x9e3779b97f4a7c15));
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((x ^ (x >> 31)) >> 11) as f64 / ((1_u64 << 53) - 1) as f64
}

pub fn place(event: DesktopEvent, placement: Placement, seed: u64) -> Option<(Bounds, u32)> {
    let work = event.work_area;
    if event.dpi == 0 || work.right <= work.left || work.bottom <= work.top {
        return None;
    }
    let available = if placement == Placement::WindowAnchor {
        Bounds {
            left: event.bounds.left.max(work.left),
            top: event.bounds.top.max(work.top),
            right: event.bounds.right.min(work.right),
            bottom: event.bounds.bottom.min(work.bottom),
        }
    } else {
        work
    };
    if available.right <= available.left || available.bottom <= available.top {
        return None;
    }
    let scale = f64::from(event.dpi) / 96.0;
    let monitor_width = f64::from(work.right) - f64::from(work.left);
    let monitor_height = f64::from(work.bottom) - f64::from(work.top);
    let aw = available.right - available.left;
    let ah = available.bottom - available.top;
    // A similar fraction of each monitor, capped in DIPs on very large displays.
    // The fallback for a tiny window remains uniformly scaled and fully visible.
    let desired = (monitor_width * 0.25)
        .min(monitor_height * 0.44 * 1.8)
        .clamp(260.0 * scale, 560.0 * scale);
    let fraction = if placement == Placement::WindowAnchor {
        0.72
    } else {
        0.95
    };
    let width = desired
        .min(f64::from(aw) * fraction)
        .min(f64::from(ah) * fraction * 1.8)
        .round()
        .clamp(1.0, f64::from(aw)) as i32;
    let height = (f64::from(width) / 1.8).round().clamp(1.0, f64::from(ah)) as i32;
    let margin = (12.0 * scale).round() as i32;
    let (x, y) = match placement {
        Placement::WindowAnchor => (available.right - margin - width, available.top + margin),
        Placement::NearCursor => {
            let right = event.point.x.saturating_add(margin);
            let x = if right.saturating_add(width) <= available.right {
                right
            } else {
                event.point.x.saturating_sub(margin).saturating_sub(width)
            };
            let below = event.point.y.saturating_add(margin);
            let y = if below.saturating_add(height) <= available.bottom {
                below
            } else {
                event.point.y.saturating_sub(margin).saturating_sub(height)
            };
            (x, y)
        }
        Placement::SafeRandom => (
            available.left + (random(seed, 1) * f64::from(aw - width)).round() as i32,
            available.top + (random(seed, 2) * f64::from(ah - height)).round() as i32,
        ),
    };
    let x = x.clamp(available.left, available.right - width);
    let y = y.clamp(available.top, available.bottom - height);
    Some((
        Bounds {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        },
        event.dpi,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use navi_state::reactions::{DesktopKind, Point};

    fn event() -> DesktopEvent {
        DesktopEvent {
            kind: DesktopKind::WindowFocus,
            source: 9,
            bounds: Bounds {
                left: -1800,
                top: 40,
                right: -100,
                bottom: 980,
            },
            work_area: Bounds {
                left: -1920,
                top: 0,
                right: 0,
                bottom: 1040,
            },
            point: Point { x: -50, y: 1020 },
            dpi: 144,
            own: false,
            suppressed: false,
        }
    }

    fn contained(inner: Bounds, outer: Bounds) {
        assert!(inner.left >= outer.left && inner.top >= outer.top);
        assert!(inner.right <= outer.right && inner.bottom <= outer.bottom);
        assert!(inner.right > inner.left && inner.bottom > inner.top);
    }

    #[test]
    fn negative_monitor_origin_and_cursor_edges_are_clamped() {
        let e = event();
        for strategy in [
            Placement::NearCursor,
            Placement::WindowAnchor,
            Placement::SafeRandom,
        ] {
            let (bounds, dpi) = place(e, strategy, 19).unwrap();
            assert_eq!(dpi, 144);
            contained(bounds, e.work_area);
            if strategy == Placement::WindowAnchor {
                contained(bounds, e.bounds);
            }
        }
    }

    #[test]
    fn random_position_is_repeatable_and_seeded() {
        let e = event();
        assert_eq!(
            place(e, Placement::SafeRandom, 7),
            place(e, Placement::SafeRandom, 7)
        );
        assert_ne!(
            place(e, Placement::SafeRandom, 7),
            place(e, Placement::SafeRandom, 8)
        );
    }

    #[test]
    fn equivalent_monitors_at_mixed_dpi_keep_equivalent_logical_size() {
        let mut a = event();
        a.dpi = 96;
        a.work_area = Bounds {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
        };
        a.bounds = a.work_area;
        let mut b = a;
        b.dpi = 192;
        b.work_area = Bounds {
            left: -3840,
            top: 0,
            right: 0,
            bottom: 2080,
        };
        b.bounds = b.work_area;
        let (a, _) = place(a, Placement::WindowAnchor, 0).unwrap();
        let (b, _) = place(b, Placement::WindowAnchor, 0).unwrap();
        assert_eq!((a.right - a.left) * 2, b.right - b.left);
        // Independent physical raster rounding may differ by a single pixel.
        assert!(((a.bottom - a.top) * 2 - (b.bottom - b.top)).abs() <= 1);
    }

    #[test]
    fn small_work_area_and_tiny_window_never_escape_visible_geometry() {
        let mut e = event();
        e.bounds = Bounds {
            left: -5,
            top: 5,
            right: -4,
            bottom: 6,
        };
        let (bounds, _) = place(e, Placement::WindowAnchor, u64::MAX).unwrap();
        contained(bounds, e.bounds);
        e.work_area = e.bounds;
        for strategy in [Placement::NearCursor, Placement::SafeRandom] {
            contained(place(e, strategy, 0).unwrap().0, e.work_area);
        }
    }

    #[test]
    fn invalid_monitor_and_offscreen_anchor_do_not_allocate_a_surface() {
        let mut e = event();
        e.dpi = 0;
        assert!(place(e, Placement::NearCursor, 0).is_none());
        e.dpi = 96;
        e.bounds.left = 100;
        e.bounds.right = 900;
        assert!(place(e, Placement::WindowAnchor, 0).is_none());
        e.work_area.right = e.work_area.left;
        assert!(place(e, Placement::SafeRandom, 0).is_none());
    }
}
