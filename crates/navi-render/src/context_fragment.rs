use super::*;
use navi_state::holograms::EffectFrame;

pub struct ContextFragment;

impl ContextFragment {
    pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame) -> Result<()> {
        let data = navi_demo::ContextSample::for_phase(frame.phase);
        let a = frame.opacity;
        let color = if frame.phase == Phase::Error {
            ERROR.0
        } else {
            MINT.0
        };
        let slide = if frame.motion {
            (1.0 - (frame.progress * 6.0).min(1.0)) * 10.0
        } else {
            0.0
        };
        let x = 286.0 + slide;
        r.line(212.0, 126.0, 246.0, 64.0, (color, a * 0.38), 0.7)?;
        r.line(246.0, 64.0, x + 18.0, 64.0, (color, a * 0.38), 0.7)?;
        r.target.DrawEllipse(
            &D2D1_ELLIPSE {
                point: D2D_POINT_2F { x: 212.0, y: 126.0 },
                radiusX: 3.0,
                radiusY: 3.0,
            },
            &r.brush((color, a * 0.75))?,
            0.8,
            None,
        );
        r.text(
            "contexto / simulado",
            [x, 38.0, x + 245.0, 59.0],
            (color, a * 0.9),
            &r.small,
        )?;
        for (y, label, value) in [
            (78.0, "aplicación", data.application),
            (133.0, "archivo", data.file),
            (188.0, "acción", data.action),
        ] {
            r.text(
                label,
                [x, y, x + 235.0, y + 18.0],
                (MUTED.0, a * 0.72),
                &r.small,
            )?;
            r.text(
                value,
                [x, y + 19.0, x + 242.0, y + 43.0],
                (TEXT.0, a * 0.94),
                &r.label,
            )?;
            r.line(
                x - 10.0,
                y + 3.0,
                x - 10.0,
                y + 35.0,
                (color, a * 0.34),
                0.6,
            )?;
        }
        Ok(())
    }
}
