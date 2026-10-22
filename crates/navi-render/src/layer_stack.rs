use super::*;
use navi_state::holograms::EffectFrame;

pub struct LayerStack;

impl LayerStack {
    pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame) -> Result<()> {
        let t = if frame.motion { frame.progress } else { 0.5 };
        let entry = (t * 6.0).min(1.0);
        let exit = ((t - 0.8) * 5.0).max(0.0);
        for layer in (0..frame.layers).rev() {
            let depth = layer as f32;
            let parallax = if frame.motion {
                (t * std::f32::consts::PI).sin() * depth * 2.5
                    + (1.0 - entry) * depth * 5.0
                    + exit * depth * 4.0
            } else {
                0.0
            };
            let x = 48.0 + depth * 19.0 + parallax;
            let y = 38.0 + depth * 16.0 - parallax * 0.4;
            let a = frame.opacity;
            r.rect(
                [x, y, x + 328.0, y + 190.0],
                (0x0b1c24, a * (0.13 - depth * 0.014)),
                true,
            )?;
            r.rect(
                [x, y, x + 328.0, y + 190.0],
                (MINT.0, a * (0.30 - depth * 0.045)),
                false,
            )?;
            r.line(x, y, x + 42.0, y, (MINT.0, a * 0.68), 0.9)?;
            r.line(
                x + 328.0,
                y + 162.0,
                x + 328.0,
                y + 190.0,
                (MINT.0, a * 0.60),
                0.9,
            )?;
            r.text(
                &format!("layer / {:02}", layer + 1),
                [x + 12.0, y + 10.0, x + 128.0, y + 28.0],
                (MUTED.0, a * 0.6),
                &r.small,
            )?;
        }
        Ok(())
    }
}
