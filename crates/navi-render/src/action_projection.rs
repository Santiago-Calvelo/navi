//! Brief PS1/CRT-inspired projection: fracture, raster sweep, color separation.
use super::*;
use navi_state::{holograms::EffectFrame, visual::VisualStyle};

pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame, style: VisualStyle) -> Result<()> {
    let t = if frame.motion { frame.progress } else { 0.45 };
    crate::layer_stack_v2::draw(
        r,
        EffectFrame {
            opacity: frame.opacity * 0.78,
            ..frame
        },
        style,
    )?;
    crate::fractured_halo::draw(
        r,
        EffectFrame {
            progress: (t * 1.15).min(1.0),
            opacity: frame.opacity,
            ..frame
        },
        style,
    )?;
    let accent = if frame.phase == Phase::Error {
        ERROR.0
    } else {
        0xcbb2d9
    };
    // A moving partial raster band, never a full-frame white flash.
    let y = 32.0 + t * 226.0;
    let jitter = if t < 0.32 {
        (t * 37.0).sin() * 12.0
    } else {
        0.0
    };
    for i in 0..8 {
        let yy = y + i as f32 * 2.2;
        r.line(
            58.0 + jitter,
            yy,
            450.0 - i as f32 * 24.0,
            yy,
            (MINT.0, frame.opacity * 0.08),
            0.6,
        )?;
    }
    for i in 0..6 {
        let x = 74.0 + i as f32 * 64.0;
        let yy = 58.0 + ((i * 37) % 173) as f32;
        r.line(
            x + jitter,
            yy,
            x + 29.0 + jitter,
            yy,
            (accent, frame.opacity * 0.48),
            1.2,
        )?;
        r.line(
            x + jitter + 2.0,
            yy + 2.0,
            x + 31.0 + jitter,
            yy + 2.0,
            (0xd2c796, frame.opacity * 0.22),
            0.7,
        )?;
    }
    r.text(
        "n a v i _",
        [64.0, 251.0, 235.0, 278.0],
        (MINT.0, frame.opacity * 0.9),
        &r.label,
    )?;
    Ok(())
}
