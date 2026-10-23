//! Ruled silk membrane for a finite Sutura manifestation (540 x 300 DIP).
use super::*;
use navi_state::{
    holograms::EffectFrame,
    visual::{TemporalMotion, VisualStyle},
};
use std::f32::consts::PI;

const WIDTH: f32 = 540.0;
const HEIGHT: f32 = 300.0;
const FILAMENTS: usize = 24;
const LONGITUDINAL_STEPS: usize = 56;
const CROSS_STEPS: usize = 18;
const SAFE_MARGIN: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

fn smooth(v: f32) -> f32 {
    let v = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
    v * v * (3.0 - 2.0 * v)
}

/// Folded surface followed by every filament, with the experiment's perspective projection.
fn surface(u: f32, v: f32, opening: f32, turn: f32, motion_x: f32, motion_y: f32) -> Point {
    let bend = u * PI * 1.45 + turn;
    // Preserve the original three-dimensional membrane, uniformly fitted to
    // production space rather than flattening its vertical folds.
    let width = opening * (24.0 + 108.0 * (u * PI).sin().max(0.0).powf(0.65));
    let x = (u - 0.5) * 620.0 + v * width * bend.cos();
    let y = 38.0 * (u * PI * 2.0 + turn * 0.4).sin() + v * width * bend.sin();
    let z = v * width * (bend + 0.8).cos() * 1.5 + 80.0 * (u * PI).sin();
    let perspective = 700.0 / (700.0 + z);
    Point {
        x: 270.0 + x * perspective * 0.65 + motion_x,
        y: 150.0 + y * perspective * 0.65 + motion_y,
    }
}

fn parameters(p: f32, phase: f32) -> (f32, f32, f32, f32) {
    let opening = smooth((p - 0.06) / 0.30) * (1.0 - smooth((p - 0.74) / 0.22));
    let turn = -0.8 + smooth((p - 0.30) / 0.40) * PI * 1.2 + (phase - 0.5) * 0.16;
    let length = smooth(p / 0.20);
    let seam = smooth((p - 0.03) / 0.22) * (1.0 - smooth((p - 0.34) / 0.14));
    (opening, turn, length, seam)
}

fn phase_unit(temporal: TemporalMotion, layer: usize) -> f32 {
    temporal.layers[layer % temporal.layers.len()].phase / (2.0 * PI)
}

fn point_on_surface(u: f32, v: f32, p: f32, temporal: TemporalMotion, layer: usize) -> Point {
    let (opening, turn, _, _) = parameters(p, phase_unit(temporal, layer));
    let motion = temporal.layer(layer % 5, p);
    surface(u, v, opening, turn, motion.offset_x, motion.offset_y)
}

fn filament_points(index: usize, p: f32, temporal: TemporalMotion) -> Vec<Point> {
    let (_, _, length, _) = parameters(p, phase_unit(temporal, index));
    let v = index as f32 / (FILAMENTS - 1) as f32 * 2.0 - 1.0;
    (0..=LONGITUDINAL_STEPS)
        .map(|j| {
            point_on_surface(
                j as f32 / LONGITUDINAL_STEPS as f32 * length,
                v,
                p,
                temporal,
                index,
            )
        })
        .collect()
}

fn band_points(strip: usize, p: f32, temporal: TemporalMotion) -> Vec<Point> {
    let (_, _, length, _) = parameters(p, phase_unit(temporal, strip));
    let v = strip as f32 / (FILAMENTS - 1) as f32 * 2.0 - 1.0;
    let mut points: Vec<_> = (0..=LONGITUDINAL_STEPS)
        .map(|j| {
            point_on_surface(
                j as f32 / LONGITUDINAL_STEPS as f32 * length,
                v,
                p,
                temporal,
                strip,
            )
        })
        .collect();
    points.extend((0..=LONGITUDINAL_STEPS).rev().map(|j| {
        point_on_surface(
            j as f32 / LONGITUDINAL_STEPS as f32 * length,
            v + 0.035,
            p,
            temporal,
            strip,
        )
    }));
    points
}

fn cross_points(index: usize, p: f32, temporal: TemporalMotion) -> Vec<Point> {
    let u = index as f32 / (CROSS_STEPS - 1) as f32;
    (0..=12)
        .map(|k| point_on_surface(u, k as f32 / 6.0 - 1.0, p, temporal, index))
        .collect()
}

fn stitch_points(index: usize, p: f32, temporal: TemporalMotion) -> (Point, Point, f32) {
    let phase = temporal.layers[index % temporal.layers.len()].phase;
    let (_, _, length, seam) = parameters(p, phase / (2.0 * PI));
    let u = index as f32 / 18.0;
    let center = point_on_surface(u, 0.0, p, temporal, index);
    let height = 5.0 + ((phase + index as f32 * 0.73).sin() * 0.5 + 0.5) * 11.0;
    (
        Point {
            x: center.x - 3.0,
            y: center.y - height * seam,
        },
        Point {
            x: center.x + 3.0,
            y: center.y + height * seam,
        },
        if u <= length { seam } else { 0.0 },
    )
}

fn in_canvas(point: Point) -> bool {
    point.x >= SAFE_MARGIN
        && point.x <= WIDTH - SAFE_MARGIN
        && point.y >= SAFE_MARGIN
        && point.y <= HEIGHT - SAFE_MARGIN
}

fn effective_temporal(motion: bool, temporal: Option<TemporalMotion>) -> TemporalMotion {
    if motion {
        temporal.unwrap_or_default()
    } else {
        TemporalMotion::default()
    }
}

unsafe fn path(
    r: &Renderer,
    points: &[Point],
    fill: Option<(u32, f32)>,
    stroke: Option<((u32, f32), f32)>,
) -> Result<()> {
    if points.len() < 2 {
        return Ok(());
    }
    let geometry = r.target.GetFactory()?.CreatePathGeometry()?;
    let sink = geometry.Open()?;
    let first = points[0];
    sink.BeginFigure(
        D2D_POINT_2F {
            x: first.x,
            y: first.y,
        },
        if fill.is_some() {
            D2D1_FIGURE_BEGIN_FILLED
        } else {
            D2D1_FIGURE_BEGIN_HOLLOW
        },
    );
    let lines: Vec<_> = points[1..]
        .iter()
        .map(|p| D2D_POINT_2F { x: p.x, y: p.y })
        .collect();
    sink.AddLines(&lines);
    sink.EndFigure(if fill.is_some() {
        D2D1_FIGURE_END_CLOSED
    } else {
        D2D1_FIGURE_END_OPEN
    });
    sink.Close()?;
    if let Some(color) = fill {
        r.target.FillGeometry(&geometry, &r.brush(color)?, None);
    }
    if let Some((color, width)) = stroke {
        r.target
            .DrawGeometry(&geometry, &r.brush(color)?, width, None);
    }
    Ok(())
}

pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame, style: VisualStyle) -> Result<()> {
    // The host already applies the shared temporal envelope to frame.opacity.
    // A static frame deliberately suppresses all temporal shape changes.
    let temporal = effective_temporal(frame.motion, style.temporal);
    let p = if frame.motion {
        frame.progress.clamp(0.0, 1.0)
    } else {
        0.5
    };
    let (opening, turn, length, _) = parameters(p, phase_unit(temporal, 0));
    let opacity = frame.opacity.clamp(0.0, 1.0);

    // Narrow intermittent ink bands retain the paper/red accent language.
    for strip in (0..FILAMENTS).step_by(2) {
        let points = band_points(strip, p, temporal);
        if points.iter().all(|point| in_canvas(*point)) {
            path(r, &points, Some((0x251e1c, opacity * 0.20)), None)?;
        }
    }
    for i in 0..FILAMENTS {
        let points = filament_points(i, p, temporal);
        let brightness = (i as f32 * 0.71 + turn).cos() * 0.5 + 0.5;
        let ink = if i % 8 == 0 {
            (0x9b4339, opacity * 0.58)
        } else {
            (0xe8dfd1, opacity * (0.16 + brightness * 0.52))
        };
        path(
            r,
            &points,
            None,
            Some((
                ink,
                if i == 0 || i == FILAMENTS - 1 {
                    1.0
                } else {
                    0.58
                },
            )),
        )?;
    }
    for j in 0..CROSS_STEPS {
        let u = j as f32 / (CROSS_STEPS - 1) as f32;
        if u > length {
            continue;
        }
        let points = cross_points(j, p, temporal);
        path(
            r,
            &points,
            None,
            Some(((0xb7c4bd, opacity * 0.12 * opening), 0.45)),
        )?;
    }
    for i in 0..18 {
        let (a, b, stitch_opacity) = stitch_points(i, p, temporal);
        r.line(
            a.x,
            a.y,
            b.x,
            b.y,
            (0x9b4339, opacity * stitch_opacity),
            0.9,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use navi_state::reactions::Intensity;

    fn all_geometry(p: f32, temporal: TemporalMotion) -> Vec<Point> {
        let mut points = Vec::new();
        for i in 0..FILAMENTS {
            points.extend(filament_points(i, p, temporal));
        }
        for i in (0..FILAMENTS).step_by(2) {
            points.extend(band_points(i, p, temporal));
        }
        for i in 0..CROSS_STEPS {
            points.extend(cross_points(i, p, temporal));
        }
        for i in 0..18 {
            let (a, b, _) = stitch_points(i, p, temporal);
            points.extend([a, b]);
        }
        points
    }

    #[test]
    fn dense_geometry_and_all_detail_stay_inside_canvas_under_motion() {
        for seed in [0, 1, 42, 0xfeed_beef, u64::MAX] {
            let motion = TemporalMotion::sample(seed, Intensity::Anomalous, true);
            for step in 0..=100 {
                let p = step as f32 / 100.0;
                let geometry = all_geometry(p, motion);
                assert!(geometry.iter().all(|point| {
                    point.x.is_finite() && point.y.is_finite() && in_canvas(*point)
                }));
            }
        }
    }

    #[test]
    fn fine_progress_steps_are_continuous_and_seed_changes_geometry() {
        let temporal = TemporalMotion::sample(7, Intensity::Anomalous, true);
        for i in 0..10_000 {
            let p = i as f32 / 10_000.0;
            let a = filament_points(7, p, temporal);
            let b = filament_points(7, p + 0.0001, temporal);
            assert!(a
                .iter()
                .zip(b)
                .all(|(a, b)| { (a.x - b.x).abs() < 0.5 && (a.y - b.y).abs() < 0.5 }));
        }
        let a = all_geometry(0.5, TemporalMotion::sample(7, Intensity::Anomalous, true));
        let b = all_geometry(0.5, TemporalMotion::sample(8, Intensity::Anomalous, true));
        assert_ne!(a, b);
    }

    #[test]
    fn static_mode_is_identical_and_shared_envelopes_have_endpoints() {
        let static_a = all_geometry(0.5, effective_temporal(false, None));
        let static_b = all_geometry(
            0.5,
            effective_temporal(
                false,
                Some(TemporalMotion::sample(42, Intensity::Anomalous, true)),
            ),
        );
        assert_eq!(static_a, static_b);
        assert_eq!(parameters(0.0, 0.5).0, 0.0);
        assert_eq!(parameters(1.0, 0.5).0, 0.0);
        assert_eq!(parameters(0.0, 0.5).2, 0.0);
        assert_eq!(parameters(1.0, 0.5).2, 1.0);
        assert_eq!(parameters(0.0, 0.5).3, 0.0);
        assert_eq!(parameters(1.0, 0.5).3, 0.0);
        for i in 0..18 {
            for p in [0.0, 1.0] {
                let (_, _, opacity) =
                    stitch_points(i, p, TemporalMotion::sample(42, Intensity::Anomalous, true));
                assert_eq!(opacity, 0.0);
            }
        }
    }
}
