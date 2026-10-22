use super::*;
use navi_state::{
    holograms::EffectFrame,
    reactions::Intensity,
    visual::{Family, VisualStyle},
};

pub(super) unsafe fn draw(
    r: &Renderer,
    frame: EffectFrame,
    family: Family,
    style: VisualStyle,
    points: [(f32, f32); 2],
) -> Result<()> {
    let weight = match style.intensity {
        Intensity::Subtle => 0.45,
        Intensity::Experimental => 0.75,
        Intensity::Anomalous => 0.95,
    };
    let alpha = frame.opacity * weight;
    let [(x, y), (ex, ey)] = points;
    if family == Family::ClickPulse {
        let radius = 5.0 + frame.progress * 24.0;
        for i in 0..6 {
            let a = i as f32 * std::f32::consts::TAU / 6.0 + 0.25;
            let b = a + 0.65;
            r.line(
                x + radius * a.cos(),
                y + radius * a.sin(),
                x + radius * b.cos(),
                y + radius * b.sin(),
                (MINT.0, alpha),
                1.1,
            )?;
        }
    } else {
        let dx = ex - x;
        let dy = ey - y;
        let len = dx.hypot(dy).max(1.0);
        let (nx, ny) = (-dy / len * 9.0, dx / len * 9.0);
        for i in 0..3 {
            let offset = i as f32 - 1.0;
            r.line(
                x + nx * offset,
                y + ny * offset,
                ex + nx * offset * 0.3,
                ey + ny * offset * 0.3,
                (MINT.0, alpha * if i == 1 { 0.85 } else { 0.25 }),
                0.8,
            )?;
        }
        for i in 0..5 {
            let t = i as f32 / 4.0;
            let (cx, cy) = (x + dx * t, y + dy * t);
            r.line(
                cx - nx * 0.4,
                cy - ny * 0.4,
                cx + nx * 0.4,
                cy + ny * 0.4,
                (MINT.0, alpha * 0.6),
                0.8,
            )?;
        }
        for (cx, cy) in points {
            r.line(cx - 4.0, cy, cx + 4.0, cy, (MINT.0, alpha), 1.0)?;
            r.line(cx, cy - 4.0, cx, cy + 4.0, (MINT.0, alpha), 1.0)?;
        }
    }
    Ok(())
}
