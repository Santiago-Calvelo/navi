//! Startup adaptation of the preserved Visual Genesis letter study.
use super::*;
use navi_state::visual::variation_unit;
use windows::Foundation::Numerics::Matrix3x2;

fn smooth(a: f32, b: f32, t: f32) -> f32 {
    let x = ((t - a) / (b - a)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

struct Line {
    layout: IDWriteTextLayout,
    width: f32,
    letters: Vec<(char, f32)>,
}

pub(super) struct StartupText {
    phrase: String,
    lines: Vec<Line>,
    fit: f32,
}

impl StartupText {
    unsafe fn new(renderer: &Renderer, phrase: &str) -> Result<Self> {
        let mut lines = Vec::new();
        let mut widest = 1.0_f32;
        for line in phrase.lines() {
            let utf16: Vec<u16> = line.encode_utf16().collect();
            let layout = renderer.write.CreateTextLayout(&utf16, &renderer.air, 4096.0, 100.0)?;
            let mut metrics = DWRITE_TEXT_METRICS::default();
            layout.GetMetrics(&mut metrics)?;
            let width = metrics.widthIncludingTrailingWhitespace;
            widest = widest.max(width);
            let mut letters = Vec::new();
            let mut position = 0;
            for ch in line.chars() {
                let (mut x, mut y) = (0.0, 0.0);
                let mut hit = DWRITE_HIT_TEST_METRICS::default();
                layout.HitTestTextPosition(position, false, &mut x, &mut y, &mut hit)?;
                letters.push((ch, x));
                position += ch.len_utf16() as u32;
            }
            lines.push(Line { layout, width, letters });
        }
        Ok(Self { phrase: phrase.into(), lines, fit: (720.0 / widest).min(1.0) })
    }
}

impl Renderer {
    /// # Safety
    /// Requires this renderer's owning UI thread and a live layered HWND.
    pub unsafe fn draw_startup(
        &self,
        hwnd: HWND,
        phrase: &str,
        progress: f32,
        motion: bool,
        seed: u64,
    ) -> Result<()> {
        let t = if motion {
            progress.clamp(0.0, 1.0)
        } else {
            0.55
        };
        let opacity = smooth(0.0, 0.08, t) * (1.0 - smooth(0.90, 1.0, t));
        let alignment = smooth(0.08, 0.32, t);
        let dissolve = smooth(0.76, 1.0, t);
        let mut cached = self.startup_text.borrow_mut();
        if cached.as_ref().is_none_or(|text| text.phrase != phrase) {
            *cached = Some(StartupText::new(self, phrase)?);
        }
        let text = cached.as_ref().unwrap();
        let rows = text.lines.len();
        let canvas_scale = (self.width as f32 / 800.0).min(self.height as f32 / 460.0);
        let pad_x = (self.width as f32 - 800.0 * canvas_scale) * 0.5;
        let pad_y = (self.height as f32 - 460.0 * canvas_scale) * 0.5;
        let mut original = Matrix3x2::default();
        self.target.GetTransform(&mut original);
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F::default()));
        let drawn = (|| -> Result<()> {
            let mut ordinal = 0;
            for (row, line) in text.lines.iter().enumerate() {
                let x = (800.0 - line.width * text.fit) * 0.5;
                let y = 230.0 - rows as f32 * 36.0 + row as f32 * 72.0;
                if alignment == 1.0 && dissolve == 0.0 {
                    self.target.SetTransform(&Matrix3x2 {
                        M11: canvas_scale * text.fit, M12: 0.0,
                        M21: 0.0, M22: canvas_scale * text.fit,
                        M31: pad_x + canvas_scale * x,
                        M32: pad_y + canvas_scale * y,
                    });
                    self.target.DrawTextLayout(
                        D2D_POINT_2F { x: 0.0, y: 0.0 }, &line.layout,
                        &self.brush((0xe6e2d4, opacity))?, D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                    continue;
                }
                for &(ch, advance) in &line.letters {
                    ordinal += 1;
                    if ch == ' ' {
                        continue;
                    }
                    let n = variation_unit(seed, ordinal);
                    let depth = variation_unit(seed, ordinal + 200);
                    let local = smooth(n * 0.12, 1.0, alignment);
                    let source = (60.0 + n * 650.0, 70.0 + depth * 280.0);
                    let px = source.0
                        + (x + advance * text.fit - source.0) * local
                        + dissolve * (n - 0.5) * 140.0;
                    let py = source.1 + (y - source.1) * local - dissolve * (30.0 + depth * 70.0);
                    let scale =
                        (0.25 + n * 0.2 + (0.75 - n * 0.2) * local) * (1.0 - dissolve * 0.4) * text.fit;
                    let letter = ch.to_string();
                    for layer in (0..=3).rev() {
                        let d = layer as f32;
                        self.target.SetTransform(&Matrix3x2 {
                            M11: canvas_scale * scale,
                            M12: 0.0,
                            M21: 0.0,
                            M22: canvas_scale * scale,
                            M31: pad_x
                                + canvas_scale
                                    * (px + (1.0 - local) * d * 16.0 + dissolve * d * 5.0),
                            M32: pad_y + canvas_scale * (py - (1.0 - local) * d * 9.0),
                        });
                        let color = if layer == 0 {
                            (0xe6e2d4, opacity * (0.4 + local * 0.6))
                        } else {
                            (0x9dd6cf, opacity * ((1.0 - local) + dissolve) * 0.12 / d)
                        };
                        self.text(&letter, [0.0, 0.0, 70.0, 75.0], color, &self.air)?;
                    }
                }
            }
            Ok(())
        })();
        self.target.SetTransform(&original);
        let end = self.target.EndDraw(None, None);
        drawn?;
        end?;
        self.present_overlay(hwnd)
    }
}
