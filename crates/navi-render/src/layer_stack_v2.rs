//! Finite, fractured projection planes in a canonical 540 x 300 DIP viewport.
//! No HWNDs, clocks, event observation or retained animation resources.
use super::*;
use navi_state::{holograms::EffectFrame, reactions::Intensity, visual::VisualStyle};

pub struct LayerStackV2;

pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame, style: VisualStyle) -> Result<()> {
    LayerStackV2::draw(r, frame, style)
}

fn smooth(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn noise(seed: u64, index: u64) -> f32 {
    let mut value = seed.wrapping_add(index.wrapping_mul(0x9e3779b97f4a7c15));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((value ^ (value >> 31)) & 0xffff) as f32 / 65535.0
}

fn geometry_point(
    (x, y, width, height, shear_y, skew): (f32, f32, f32, f32, f32, f32),
    (u, v): (f32, f32),
    displacement: f32,
    (center_x, center_y): (f32, f32),
    angle: f32,
) -> D2D_POINT_2F {
    let px = (x + u * width + v * skew + displacement) * 0.9 + 32.0;
    let py = y + v * height - u * shear_y;
    let (sin, cos) = angle.sin_cos();
    D2D_POINT_2F {
        x: center_x + (px - center_x) * cos - (py - center_y) * sin,
        y: center_y + (px - center_x) * sin + (py - center_y) * cos,
    }
}

impl LayerStackV2 {
    pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame, style: VisualStyle) -> Result<()> {
        let t = if frame.motion {
            frame.progress.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let (strength, spread, extra) = match style.intensity {
            Intensity::Subtle => (0.64, 0.6, 0),
            Intensity::Experimental => (0.88, 1.0, 0),
            Intensity::Anomalous => (1.0, 1.35, 1),
        };
        let count = (frame.layers.clamp(2, 4) + extra).min(5);
        let factory = r.target.GetFactory()?;
        for layer in (0..count).rev() {
            let d = layer as f32;
            let temporal = style.temporal;
            let (construct, dissolve, organic) = if let Some(motion) = temporal {
                (
                    motion.construct(layer as usize, t),
                    motion.dissolve(layer as usize, t),
                    if frame.motion {
                        motion.layer(layer as usize, t)
                    } else {
                        Default::default()
                    },
                )
            } else {
                let local = t - (count - 1 - layer) as f32 * 0.025;
                (
                    smooth(local / 0.27),
                    smooth((t - 0.73 - d * 0.012) / 0.22),
                    Default::default(),
                )
            };
            let envelope = frame.opacity.clamp(0.0, 1.0) * construct * (1.0 - dissolve);
            if envelope < 0.001 {
                continue;
            }
            let n = noise(style.seed, layer as u64);
            let travel = (1.0 - construct) * 36.0 + dissolve * 43.0;
            let drift = (t * std::f32::consts::PI).sin() * (d - 1.0) * 6.0 * spread;
            // Leave enough room for skew and travelling fragments even at the
            // five-plane Anomalous setting; do not clip depth on a narrow canvas.
            let x = 54.0 + d * 22.0 * spread + drift + travel * (n - 0.5) + organic.offset_x;
            let y = 48.0 + d * 18.0 - drift * 0.35 - travel * 0.32 + organic.offset_y;
            let width = 326.0 - d * 16.0;
            let height = 173.0 - d * 6.0;
            let skew = (n - 0.5) * 19.0 + (d - 1.0) * 9.0 * spread + organic.skew;
            let alpha = envelope * strength * (1.0 - d * 0.085);
            let cx = 32.0 + (x + width * 0.5) * 0.9;
            let cy = y + height * 0.5;
            let point = |u: f32, v: f32, displacement: f32| {
                geometry_point(
                    (x, y, width, height, 5.0 + d * 2.5, skew),
                    (u, v),
                    displacement,
                    (cx, cy),
                    organic.rotation_radians,
                )
            };

            // Each plane consists of separate tesserae; missing strips reveal the
            // desktop. They converge in depth rather than merely translating cards.
            for shard in 0..4 {
                let v0 = shard as f32 * 0.25 + 0.012;
                let v1 = (shard + 1) as f32 * 0.25 - 0.014;
                let jitter = noise(style.seed ^ layer as u64, shard as u64 + 71) - 0.5;
                let separation = jitter * (travel + (1.0 - construct) * 20.0) * spread;
                let cut = 0.56 + jitter * 0.12;
                for (u0, u1) in [(0.0, cut - 0.012), (cut + 0.016, 1.0)] {
                    let points = [
                        point(u0, v0, separation),
                        point(u1, v0, separation),
                        point(u1 - 0.015, v1, separation),
                        point(u0 + 0.018, v1, separation),
                    ];
                    let path = factory.CreatePathGeometry()?;
                    let sink = path.Open()?;
                    sink.BeginFigure(points[0], D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLines(&points[1..]);
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    sink.Close()?;
                    let brush = r.brush((0x183b42, alpha * (0.38 - d * 0.025)))?;
                    r.target.FillGeometry(&path, &brush, None);
                    // Partial seams and lit edges keep the form broken, without
                    // a conventional window border or an all-enclosing rectangle.
                    let edge = r.brush((MINT.0, alpha * (0.77 - shard as f32 * 0.08)))?;
                    r.target.DrawLine(points[0], points[1], &edge, 1.05, None);
                    if shard == 0 || shard == 3 {
                        r.target.DrawLine(points[2], points[3], &edge, 0.75, None);
                    }
                    r.target.DrawLine(points[0], points[3], &edge, 0.65, None);
                }
            }

            // Sparse structures imply an interrupted data projection rather than
            // adding a second dashboard. There are no persistent labels or HUD rings.
            for segment in 0..5 {
                let u = 0.08 + segment as f32 * 0.18;
                let p = point(u, 0.36, travel * (n - 0.5));
                let q = point(u + 0.045 + n * 0.055, 0.36, travel * (n - 0.5));
                r.target
                    .DrawLine(p, q, &r.brush((0xd1eee3, alpha * 0.65))?, 1.4, None);
            }
            if layer > 0 {
                let p = point(0.91, 0.8, 0.0);
                let tail = D2D_POINT_2F {
                    x: p.x - 25.0 * spread,
                    y: p.y - 13.0,
                };
                r.line(p.x, p.y, tail.x, tail.y, (MINT.0, alpha * 0.24), 0.6)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_noise_is_repeatable_and_bounded() {
        for seed in [0, 1, u64::MAX] {
            for index in 0..32 {
                let a = noise(seed, index);
                assert!((0.0..=1.0).contains(&a));
                assert_eq!(a, noise(seed, index));
            }
        }
        assert_ne!(noise(1, 9), noise(2, 9));
    }

    #[test]
    fn construction_envelope_has_finite_endpoints() {
        assert_eq!(smooth(-1.0), 0.0);
        assert_eq!(smooth(2.0), 1.0);
        assert_eq!(smooth(0.5), 0.5);
    }

    #[test]
    fn organic_motion_is_repeatable_bounded_and_varies_by_layer_and_time() {
        use navi_state::visual::TemporalMotion;
        let a = TemporalMotion::sample(123, Intensity::Anomalous, true);
        let b = TemporalMotion::sample(123, Intensity::Anomalous, true);
        assert_eq!(a, b);
        let first = a.layer(0, 0.4);
        assert_eq!(first, a.layer(0, 0.4));
        assert_ne!(first, a.layer(1, 0.4));
        assert_ne!(first, a.layer(0, 0.6));
        let still = TemporalMotion::sample(123, Intensity::Anomalous, false);
        assert_eq!(still.layer(2, 0.4), Default::default());
        assert_ne!(a, TemporalMotion::sample(124, Intensity::Anomalous, true));
        for i in 0..5 {
            assert_eq!(a.construct(i, 0.0), 0.0);
            assert_eq!(a.construct(i, 1.0), 1.0);
            assert_eq!(a.dissolve(i, 0.0), 0.0);
            assert_eq!(a.dissolve(i, 1.0), 1.0);
            assert_eq!(a.layer(i, 0.0), Default::default());
            assert_eq!(a.layer(i, 1.0), Default::default());
        }
        assert_eq!(a.envelope(0.0), 0.0);
        assert_eq!(a.envelope(1.0), 0.0);
        assert_eq!(still.envelope(0.5), a.envelope(0.5));
        for i in 0..5 {
            for step in 0..=10_000 {
                let p = step as f32 / 10_000.0;
                assert!((0.0..=1.0).contains(&a.envelope(p)));
                assert!((0.0..=1.0).contains(&a.construct(i, p)));
                assert!((0.0..=1.0).contains(&a.dissolve(i, p)));
                if step < 10_000 {
                    let next = (step + 1) as f32 / 10_000.0;
                    assert!(a.construct(i, next) >= a.construct(i, p));
                    assert!(a.dissolve(i, next) >= a.dissolve(i, p));
                    let first = a.layer(i, p);
                    let second = a.layer(i, next);
                    // |wave derivative| <= TAU*1.7 + PI; amplitude <= 1.
                    let bound = (std::f32::consts::TAU * 1.7 + std::f32::consts::PI) * (next - p);
                    assert!((first.offset_x - second.offset_x).abs() <= 8.0 * bound + 0.00001);
                    assert!(
                        (first.rotation_radians - second.rotation_radians).abs()
                            <= 2.5_f32.to_radians() * bound + 0.000001
                    );
                }
            }
        }
    }

    #[test]
    fn emitted_layer_geometry_stays_inside_canvas_without_clamping() {
        use navi_state::visual::TemporalMotion;
        for intensity in [
            Intensity::Subtle,
            Intensity::Experimental,
            Intensity::Anomalous,
        ] {
            for seed in 0..64 {
                let motion = TemporalMotion::sample(seed, intensity, true);
                let (spread, extra) = match intensity {
                    Intensity::Subtle => (0.6, 0),
                    Intensity::Experimental => (1.0, 0),
                    Intensity::Anomalous => (1.35, 1),
                };
                for base_layers in 2..=4 {
                    let count = (base_layers + extra).min(5);
                    for step in 0..=100 {
                        let t = step as f32 / 100.0;
                        for layer in 0..count {
                            let d = layer as f32;
                            let organic = motion.layer(layer, t);
                            let construct = motion.construct(layer, t);
                            let dissolve = motion.dissolve(layer, t);
                            let travel = (1.0 - construct) * 36.0 + dissolve * 43.0;
                            let drift = (t * std::f32::consts::PI).sin() * (d - 1.0) * 6.0 * spread;
                            let x = 54.0
                                + d * 22.0 * spread
                                + drift
                                + travel * (noise(seed, layer as u64) - 0.5)
                                + organic.offset_x;
                            let y =
                                48.0 + d * 18.0 - drift * 0.35 - travel * 0.32 + organic.offset_y;
                            let width = 326.0 - d * 16.0;
                            let height = 173.0 - d * 6.0;
                            let n = noise(seed, layer as u64);
                            let skew = (n - 0.5) * 19.0 + (d - 1.0) * 9.0 * spread + organic.skew;
                            let cx = 32.0 + (x + width * 0.5) * 0.9;
                            let cy = y + height * 0.5;
                            let point = |u, v, displacement| {
                                geometry_point(
                                    (x, y, width, height, 5.0 + d * 2.5, skew),
                                    (u, v),
                                    displacement,
                                    (cx, cy),
                                    organic.rotation_radians,
                                )
                            };
                            let mut emitted = Vec::new();
                            for shard in 0..4 {
                                let v0 = shard as f32 * 0.25 + 0.012;
                                let v1 = (shard + 1) as f32 * 0.25 - 0.014;
                                let jitter = noise(seed ^ layer as u64, shard as u64 + 71) - 0.5;
                                let separation =
                                    jitter * (travel + (1.0 - construct) * 20.0) * spread;
                                let cut = 0.56 + jitter * 0.12;
                                for (u0, u1) in [(0.0, cut - 0.012), (cut + 0.016, 1.0)] {
                                    emitted.extend([
                                        point(u0, v0, separation),
                                        point(u1, v0, separation),
                                        point(u1 - 0.015, v1, separation),
                                        point(u0 + 0.018, v1, separation),
                                    ]);
                                }
                            }
                            for segment in 0..5 {
                                let u = 0.08 + segment as f32 * 0.18;
                                emitted.push(point(u, 0.36, travel * (n - 0.5)));
                                emitted.push(point(
                                    u + 0.045 + n * 0.055,
                                    0.36,
                                    travel * (n - 0.5),
                                ));
                            }
                            if layer > 0 {
                                let p = point(0.91, 0.8, 0.0);
                                emitted.push(p);
                                emitted.push(D2D_POINT_2F {
                                    x: p.x - 25.0 * spread,
                                    y: p.y - 13.0,
                                });
                            }
                            for p in emitted {
                                assert!(p.x.is_finite() && p.y.is_finite());
                                assert!(
                                    (0.0..=540.0).contains(&p.x),
                                    "x={} seed={seed} i={intensity:?} t={t}",
                                    p.x
                                );
                                assert!(
                                    (0.0..=300.0).contains(&p.y),
                                    "y={} seed={seed} i={intensity:?} t={t}",
                                    p.y
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
