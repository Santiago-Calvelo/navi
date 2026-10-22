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
            let local = t - (count - 1 - layer) as f32 * 0.025;
            let construct = smooth(local / 0.27);
            let dissolve = smooth((t - 0.73 - d * 0.012) / 0.22);
            let envelope = frame.opacity.clamp(0.0, 1.0) * construct * (1.0 - dissolve);
            if envelope < 0.001 {
                continue;
            }
            let n = noise(style.seed, layer as u64);
            let travel = (1.0 - construct) * 36.0 + dissolve * 43.0;
            let drift = (t * std::f32::consts::PI).sin() * (d - 1.0) * 6.0 * spread;
            // Leave enough room for skew and travelling fragments even at the
            // five-plane Anomalous setting; do not clip depth on a narrow canvas.
            let x = 54.0 + d * 22.0 * spread + drift + travel * (n - 0.5);
            let y = 48.0 + d * 18.0 - drift * 0.35 - travel * 0.32;
            let width = 326.0 - d * 16.0;
            let height = 173.0 - d * 6.0;
            let skew = (n - 0.5) * 19.0 + (d - 1.0) * 9.0 * spread;
            let alpha = envelope * strength * (1.0 - d * 0.085);
            let point = |u: f32, v: f32, displacement: f32| D2D_POINT_2F {
                // Reserve horizontal bleed for temporary shard separation.
                x: (x + u * width + v * skew + displacement) * 0.9 + 32.0,
                y: y + v * height - u * (5.0 + d * 2.5),
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
                r.line(
                    p.x,
                    p.y,
                    p.x - 25.0 * spread,
                    p.y - 13.0,
                    (MINT.0, alpha * 0.24),
                    0.6,
                )?;
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
}
