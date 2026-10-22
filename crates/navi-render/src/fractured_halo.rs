//! Finite, projected spatial geometry. No input, desktop observation or HWND ownership.
use super::*;
use navi_state::{holograms::EffectFrame, reactions::Intensity, visual::VisualStyle};
use std::f32::consts::TAU;

pub struct FracturedHalo;

pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame, style: VisualStyle) -> Result<()> {
    FracturedHalo::draw(r, frame, style)
}

#[derive(Clone, Copy, Debug)]
struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

impl V3 {
    fn rotate(self, x: f32, y: f32, z: f32) -> Self {
        let (sx, cx) = x.sin_cos();
        let (sy, cy) = y.sin_cos();
        let (sz, cz) = z.sin_cos();
        let a = Self {
            x: self.x,
            y: self.y * cx - self.z * sx,
            z: self.y * sx + self.z * cx,
        };
        let b = Self {
            x: a.x * cy + a.z * sy,
            y: a.y,
            z: -a.x * sy + a.z * cy,
        };
        Self {
            x: b.x * cz - b.y * sz,
            y: b.x * sz + b.y * cz,
            z: b.z,
        }
    }

    fn project(self) -> D2D_POINT_2F {
        let perspective = 480.0 / (480.0 + self.z);
        D2D_POINT_2F {
            x: 270.0 + self.x * perspective,
            y: 150.0 + self.y * perspective,
        }
    }
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn envelope(t: f32) -> f32 {
    smooth(t / 0.18) * smooth((1.0 - t) / 0.22)
}

fn noise(seed: u64, index: usize) -> f32 {
    let mut value = seed.wrapping_add((index as u64).wrapping_mul(0x9e3779b97f4a7c15));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((value ^ (value >> 31)) & 0xffff) as f32 / 65535.0
}

fn ring_point(ring: usize, a: f32, t: f32, seed: u64) -> V3 {
    let layer = ring as f32;
    let reconstruction = 1.0 - smooth((t - layer * 0.014) / 0.27);
    let dissolution = smooth((t - 0.77) / 0.23);
    let drift = reconstruction * 22.0 + dissolution * 24.0;
    let direction = if ring.is_multiple_of(2) { 1.0 } else { -1.0 };
    let temporal_slip = if (0.12..0.20).contains(&t) || (0.76..0.80).contains(&t) {
        (t * 89.0 + layer * 3.0).sin() * 0.05
    } else {
        0.0
    };
    let phase = t * (0.28 + layer * 0.09) * direction + temporal_slip;
    let radius = 64.0 + layer * 4.5;
    let point = V3 {
        x: radius * a.cos(),
        y: radius * a.sin(),
        z: 0.0,
    }
    .rotate(
        0.48 + layer * 0.46 + phase * 0.4,
        -0.72 + layer * 0.37 - phase * 0.7,
        layer * 0.68 + phase,
    );
    V3 {
        x: point.x + (noise(seed, ring * 3) - 0.5) * drift,
        y: point.y + (noise(seed, ring * 3 + 1) - 0.5) * drift,
        z: point.z + (layer - 3.0) * 7.0 + dissolution * 18.0,
    }
}

impl FracturedHalo {
    pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame, style: VisualStyle) -> Result<()> {
        let t = if frame.motion {
            frame.progress.clamp(0.0, 1.0)
        } else {
            0.48
        };
        let (rings, shards, strength) = match style.intensity {
            Intensity::Subtle => (3, 5, 0.63),
            Intensity::Experimental => (5, 10, 0.86),
            Intensity::Anomalous => (7, 15, 1.0),
        };
        let alpha = frame.opacity.clamp(0.0, 1.0) * envelope(t) * strength;
        let color = if frame.phase == Phase::Error {
            ERROR.0
        } else {
            0xb6d7c9
        };
        // Reuse a small brush palette throughout this frame rather than creating
        // one COM resource for every sampled segment. All resources are scoped.
        let bright = r.brush((color, alpha * 0.91))?;
        let middle = r.brush((color, alpha * 0.51))?;
        let dim = r.brush((color, alpha * 0.21))?;
        let fill = r.brush((0x507c72, alpha * 0.11))?;
        let factory = r.target.GetFactory()?;

        // Incomplete oblique bands with unrelated gaps. There are deliberately
        // no ticks, crosshairs, labels, central reticle or screen-space circles.
        for ring in (0..rings).rev() {
            let offset = noise(style.seed, ring + 71) * TAU;
            for segment in 0..64 {
                let sector = (segment + ring * 9) % 64;
                if sector > 47 || (18..25).contains(&sector) {
                    continue;
                }
                let local = segment as f32 / 64.0;
                let reveal = smooth((t - noise(style.seed, segment + ring * 64) * 0.17) / 0.13);
                if reveal < 0.1 || (t > 0.80 && local < (t - 0.80) * 3.0) {
                    continue;
                }
                let a = local * TAU + offset;
                let p = ring_point(ring, a, t, style.seed);
                let q = ring_point(ring, a + TAU / 64.0 * 0.92, t, style.seed);
                let brush = if p.z < -28.0 {
                    &bright
                } else if p.z < 24.0 {
                    &middle
                } else {
                    &dim
                };
                r.target.DrawLine(
                    p.project(),
                    q.project(),
                    brush,
                    0.75 + (ring % 3) as f32 * 0.3,
                    None,
                );
                if segment.is_multiple_of(13) {
                    // A split edge gives each band a narrow physical thickness.
                    let edge = V3 { z: p.z + 3.0, ..p };
                    r.target
                        .DrawLine(p.project(), edge.project(), &middle, 0.7, None);
                }
            }
        }

        // Detached, irregular facets inhabit different planes. Their vertices
        // reconstruct together and break apart at the end of the finite effect.
        for shard in 0..shards {
            let n = noise(style.seed, shard + 401);
            let a = n * TAU + t * (0.12 + shard as f32 * 0.013);
            let scatter = (1.0 - smooth(t / 0.3)) * 22.0 + smooth((t - 0.75) / 0.25) * 21.0;
            let radius = 75.0 + noise(style.seed, shard + 501) * 24.0 + scatter;
            let center = V3 {
                x: a.cos() * radius,
                y: a.sin() * radius * 0.68,
                z: (n - 0.5) * 112.0,
            };
            let mut vertices = [D2D_POINT_2F::default(); 3];
            for (vertex, result) in vertices.iter_mut().enumerate() {
                let angle = vertex as f32 * TAU / 3.0 + a;
                let span = 5.0 + noise(style.seed, shard * 3 + vertex + 601) * 10.0;
                let local = V3 {
                    x: angle.cos() * span,
                    y: angle.sin() * span,
                    z: 0.0,
                }
                .rotate(a * 0.8, a + t * 0.7, a * 0.3);
                *result = V3 {
                    x: center.x + local.x,
                    y: center.y + local.y,
                    z: center.z + local.z,
                }
                .project();
            }
            let geometry = factory.CreatePathGeometry()?;
            let sink = geometry.Open()?;
            sink.BeginFigure(vertices[0], D2D1_FIGURE_BEGIN_FILLED);
            sink.AddLines(&vertices[1..]);
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            r.target.FillGeometry(&geometry, &fill, None);
            for edge in 0..3 {
                r.target
                    .DrawLine(vertices[edge], vertices[(edge + 1) % 3], &middle, 0.8, None);
            }
            if shard.is_multiple_of(3) {
                let anchor = ring_point(shard % rings, a + 0.19, t, style.seed).project();
                r.target.DrawLine(vertices[0], anchor, &dim, 0.55, None);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifetime_has_finite_entry_stability_and_exit() {
        assert_eq!(envelope(0.0), 0.0);
        assert_eq!(envelope(1.0), 0.0);
        assert_eq!(envelope(0.5), 1.0);
        for step in 0..=100 {
            let value = envelope(step as f32 / 100.0);
            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
        }
    }

    #[test]
    fn projected_rings_stay_in_canonical_viewport_at_every_stage() {
        for seed in [0, 1, u64::MAX] {
            for step in 0..=100 {
                for ring in 0..7 {
                    for segment in 0..64 {
                        let p = ring_point(
                            ring,
                            segment as f32 * TAU / 64.0,
                            step as f32 / 100.0,
                            seed,
                        )
                        .project();
                        assert!(p.x.is_finite() && p.y.is_finite());
                        assert!((0.0..540.0).contains(&p.x), "x={}", p.x);
                        assert!((0.0..300.0).contains(&p.y), "y={}", p.y);
                    }
                }
            }
        }
    }

    #[test]
    fn seed_is_reproducible_and_rotations_are_spatial() {
        assert_eq!(noise(77, 4), noise(77, 4));
        assert_ne!(noise(77, 4), noise(78, 4));
        let a = ring_point(0, 0.8, 0.5, 1);
        let b = ring_point(1, 0.8, 0.5, 1);
        assert!((a.z - b.z).abs() > 1.0);
        let rotated = V3 {
            x: 31.0,
            y: -17.0,
            z: 13.0,
        }
        .rotate(0.7, 1.3, -0.2);
        assert!((rotated.x.powi(2) + rotated.y.powi(2) + rotated.z.powi(2) - 1419.0).abs() < 0.01);
    }
}
