use super::*;
use navi_state::holograms::{EffectFrame, CONTEXT, SIGNAL, STACK};

pub struct SignalField;

impl SignalField {
    unsafe fn draw(renderer: &Renderer, frame: EffectFrame) -> Result<()> {
        let color = if frame.phase == Phase::Error {
            ERROR.0
        } else {
            MINT.0
        };
        let alpha = frame.opacity;
        let t = if frame.motion { frame.progress } else { 0.35 };
        let cx = 124.0;
        let cy = 120.0;
        for (r, a) in [(34.0, 0.65), (55.0, 0.30), (82.0, 0.16)] {
            renderer.target.DrawEllipse(
                &D2D1_ELLIPSE {
                    point: D2D_POINT_2F { x: cx, y: cy },
                    radiusX: r,
                    radiusY: r,
                },
                &renderer.brush((color, alpha * a))?,
                0.7,
                None,
            );
        }
        renderer.line(cx - 98.0, cy, cx + 100.0, cy, (color, alpha * 0.14), 0.5)?;
        renderer.line(cx, cy - 100.0, cx, cy + 99.0, (color, alpha * 0.14), 0.5)?;
        let sweep = t * std::f32::consts::TAU;
        let r = if frame.phase == Phase::Processing {
            55.0
        } else {
            40.0 + t * 40.0
        };
        for i in 0..56 {
            let a = i as f32 * 0.065 + sweep;
            let b = a + 0.04;
            renderer.line(
                cx + r * a.cos(),
                cy + r * a.sin(),
                cx + r * b.cos(),
                cy + r * b.sin(),
                (color, alpha * (i as f32 / 56.0) * 0.65),
                0.9,
            )?;
        }
        for i in 0..18 {
            let a = i as f32 * std::f32::consts::TAU / 18.0 + sweep * 0.1;
            let r = 66.0 + (i % 3) as f32 * 9.0;
            renderer.target.FillEllipse(
                &D2D1_ELLIPSE {
                    point: D2D_POINT_2F {
                        x: cx + r * a.cos(),
                        y: cy + r * a.sin(),
                    },
                    radiusX: 0.9,
                    radiusY: 0.9,
                },
                &renderer.brush((color, alpha * 0.55))?,
            );
        }
        let amplitude = match frame.phase {
            Phase::Listening => 9.0,
            Phase::Processing => 4.0,
            Phase::Error => 12.0,
            _ => 2.0,
        };
        for i in 0..30 {
            let x = cx - 44.0 + i as f32 * 3.0;
            let height = ((i as f32 * 1.9 + t * 14.0).sin().abs() * amplitude)
                * (-((i as f32 - 15.0) / 10.0).powi(2)).exp()
                + 0.5;
            renderer.line(x, cy - height, x, cy + height, (color, alpha * 0.85), 0.8)?;
        }
        Ok(())
    }
}

impl Renderer {
    /// Draw existing geometry at a smaller logical scale for an edge-anchored
    /// focus reaction; always restore the transform for subsequent demo draws.
    /// # Safety
    /// Same live layered HWND and owning-thread requirements as `draw_holograms`.
    pub unsafe fn draw_holograms_scaled(
        &self,
        hwnd: HWND,
        frame: EffectFrame,
        scale: f32,
    ) -> Result<()> {
        use windows::Foundation::Numerics::Matrix3x2;
        self.target.SetTransform(&Matrix3x2 {
            M11: scale,
            M22: scale,
            ..Default::default()
        });
        let result = self.draw_holograms(hwnd, frame);
        self.target.SetTransform(&Matrix3x2 {
            M11: 1.0,
            M22: 1.0,
            ..Default::default()
        });
        result
    }
    /// Diagnostic-only compositor capture. Never used by the interactive frontend.
    /// # Safety
    /// Call on the UI thread, with a valid screen-space rectangle.
    pub unsafe fn capture_composed_for_qa(bounds: RECT, path: &std::path::Path) -> Result<()> {
        let r = Self::new(
            96,
            SurfaceSize {
                width: (bounds.right - bounds.left) as f32,
                height: (bounds.bottom - bounds.top) as f32,
            },
        )?;
        let screen = GetDC(None);
        if screen.0 == 0 {
            return Err(Error::from_win32());
        }
        let copied = BitBlt(
            r.dc,
            0,
            0,
            r.width,
            r.height,
            screen,
            bounds.left,
            bounds.top,
            SRCCOPY | CAPTUREBLT,
        );
        ReleaseDC(None, screen);
        copied?;
        let _ = GdiFlush();
        let bytes =
            std::slice::from_raw_parts_mut(r.pixels as *mut u8, (r.width * r.height * 4) as usize);
        for pixel in bytes.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        r.snapshot(path)
            .map_err(|e| Error::new(E_FAIL, e.to_string()))
    }
    /// # Safety
    /// Requires a live layered HWND owned by the renderer's UI thread.
    pub unsafe fn draw_holograms(&self, hwnd: HWND, frame: EffectFrame) -> Result<()> {
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F::default()));
        let drawn = (|| -> Result<()> {
            if frame.mask & STACK != 0 {
                crate::layer_stack::LayerStack::draw(self, frame)?;
            }
            if frame.mask & SIGNAL != 0 {
                SignalField::draw(self, frame)?;
            }
            if frame.mask & CONTEXT != 0 {
                crate::context_fragment::ContextFragment::draw(self, frame)?;
            }
            Ok(())
        })();
        let end = self.target.EndDraw(None, None);
        drawn?;
        end?;
        self.present_overlay(hwnd)
    }

    unsafe fn present_overlay(&self, hwnd: HWND) -> Result<()> {
        UpdateLayeredWindow(
            hwnd,
            None,
            None,
            Some(&SIZE {
                cx: self.width,
                cy: self.height,
            }),
            self.dc,
            Some(&POINT::default()),
            COLORREF(0),
            Some(&BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            }),
            ULW_ALPHA,
        )
    }

    /// Finite manifestations use the existing premultiplied-alpha compositor.
    /// # Safety
    /// Requires an owning UI thread and live layered HWND.
    pub unsafe fn draw_manifestation(
        &self,
        hwnd: HWND,
        frame: EffectFrame,
        family: navi_state::visual::Family,
        style: navi_state::visual::VisualStyle,
        scale: f32,
        points: [(f32, f32); 2],
    ) -> Result<()> {
        use navi_state::visual::Family;
        use windows::Foundation::Numerics::Matrix3x2;
        self.target.SetTransform(&Matrix3x2 {
            M11: scale,
            M22: scale,
            ..Default::default()
        });
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F::default()));
        let drawn = match family {
            Family::StackV1 => crate::layer_stack::LayerStack::draw(self, frame),
            Family::StackV2 => crate::layer_stack_v2::draw(self, frame, style),
            Family::FracturedHalo => crate::fractured_halo::draw(self, frame, style),
            Family::ClickPulse | Family::DragConnector => {
                crate::cursor_effects::draw(self, frame, family, style, points)
            }
        };
        let ended = self.target.EndDraw(None, None);
        self.target.SetTransform(&Matrix3x2 {
            M11: 1.0,
            M22: 1.0,
            ..Default::default()
        });
        drawn?;
        ended?;
        self.present_overlay(hwnd)
    }

    pub fn alpha_stats(&self) -> (usize, usize, usize) {
        let pixels = unsafe {
            std::slice::from_raw_parts(
                self.pixels as *const u8,
                (self.width * self.height * 4) as usize,
            )
        };
        let mut counts = (0, 0, 0);
        for p in pixels.as_chunks::<4>().0 {
            match p[3] {
                0 => counts.0 += 1,
                255 => counts.2 += 1,
                _ => counts.1 += 1,
            }
        }
        counts
    }
}
