use super::*;
use navi_state::holograms::{EffectFrame, CONTEXT, SIGNAL, STACK};
use windows::Foundation::Numerics::Matrix3x2;

fn stack_v2_transform(view: (f32, f32), scale_rotation: (f32, f32)) -> Result<Matrix3x2> {
    let (width, height) = view;
    let (scale, rotation) = scale_rotation;
    if ![width, height, scale, rotation]
        .iter()
        .all(|value| value.is_finite())
        || width <= 0.0
        || height <= 0.0
        || scale <= 0.0
    {
        return Err(Error::new(E_INVALIDARG, "Invalid StackV2 transform"));
    }
    let (sin, cos) = rotation.sin_cos();
    let mut matrix = Matrix3x2 {
        M11: scale * cos,
        M12: scale * sin,
        M21: -scale * sin,
        M22: scale * cos,
        ..Default::default()
    };
    matrix.M31 = width / 2.0 - 270.0 * matrix.M11 - 150.0 * matrix.M21;
    matrix.M32 = height / 2.0 - 270.0 * matrix.M12 - 150.0 * matrix.M22;
    if ![matrix.M31, matrix.M32]
        .iter()
        .all(|value| value.is_finite())
    {
        return Err(Error::new(E_INVALIDARG, "StackV2 transform overflow"));
    }
    Ok(matrix)
}

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
        context: &[(String, String)],
    ) -> Result<()> {
        use windows::Foundation::Numerics::Matrix3x2;
        self.target.SetTransform(&Matrix3x2 {
            M11: scale,
            M22: scale,
            ..Default::default()
        });
        let result = self.draw_holograms(hwnd, frame, context);
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
    pub unsafe fn draw_holograms(
        &self,
        hwnd: HWND,
        frame: EffectFrame,
        context: &[(String, String)],
    ) -> Result<()> {
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F::default()));
        let drawn = (|| -> Result<()> {
            if frame.mask & STACK != 0 {
                crate::layer_stack_v2::draw(self, frame, Default::default())?;
            }
            if frame.mask & SIGNAL != 0 {
                SignalField::draw(self, frame)?;
            }
            if frame.mask & CONTEXT != 0 {
                crate::context_fragment::ContextFragment::draw(self, frame, context)?;
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
        let transform = Matrix3x2 {
            M11: scale,
            M22: scale,
            ..Default::default()
        };
        let restore = Matrix3x2 {
            M11: 1.0,
            M22: 1.0,
            ..Default::default()
        };
        self.draw_manifestation_transformed(
            hwnd,
            frame,
            family,
            style,
            (transform, restore),
            points,
        )
    }

    /// Center the canonical StackV2 canvas in the logical viewport, scaling and
    /// rotating about (270, 150). Other families retain their legacy placement.
    /// Opacity, rhythm and rotated viewport bounds are resolved by the caller.
    /// # Safety
    /// Requires an owning UI thread and live layered HWND.
    pub unsafe fn draw_manifestation_varied(
        &self,
        hwnd: HWND,
        frame: EffectFrame,
        family: navi_state::visual::Family,
        style: navi_state::visual::VisualStyle,
        scale_rotation: (f32, f32),
        points: [(f32, f32); 2],
    ) -> Result<()> {
        if !matches!(
            family,
            navi_state::visual::Family::StackV2
                | navi_state::visual::Family::Sutura
                | navi_state::visual::Family::FracturedHalo
        ) {
            return self.draw_manifestation(hwnd, frame, family, style, scale_rotation.0, points);
        }
        let transform = stack_v2_transform(
            (
                self.width as f32 / self.scale,
                self.height as f32 / self.scale,
            ),
            scale_rotation,
        )?;
        let mut restore = Matrix3x2::default();
        self.target.GetTransform(&mut restore);
        self.draw_manifestation_transformed(
            hwnd,
            frame,
            family,
            style,
            (transform, restore),
            points,
        )
    }

    unsafe fn draw_manifestation_transformed(
        &self,
        hwnd: HWND,
        frame: EffectFrame,
        family: navi_state::visual::Family,
        style: navi_state::visual::VisualStyle,
        transforms: (Matrix3x2, Matrix3x2),
        points: [(f32, f32); 2],
    ) -> Result<()> {
        use navi_state::visual::Family;
        self.target.SetTransform(&transforms.0);
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F::default()));
        let drawn = match family {
            Family::StackV2 => crate::layer_stack_v2::draw(self, frame, style),
            Family::Sutura => crate::sutura::draw(self, frame, style),
            Family::FracturedHalo => crate::fractured_halo::draw(self, frame, style),
            Family::ClickPulse | Family::DragConnector => {
                crate::cursor_effects::draw(self, frame, family, style, points)
            }
        };
        let ended = self.target.EndDraw(None, None);
        self.target.SetTransform(&transforms.1);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn point(matrix: Matrix3x2, x: f32, y: f32) -> (f32, f32) {
        (
            x * matrix.M11 + y * matrix.M21 + matrix.M31,
            x * matrix.M12 + y * matrix.M22 + matrix.M32,
        )
    }

    fn near(actual: (f32, f32), expected: (f32, f32)) {
        assert!((actual.0 - expected.0).abs() < 0.001, "{actual:?}");
        assert!((actual.1 - expected.1).abs() < 0.001, "{actual:?}");
    }

    #[test]
    fn varied_stack_center_stays_at_viewport_center() {
        for scale in [0.4, 1.0, 1.8] {
            for rotation in [-1.2, 0.0, 0.7, std::f32::consts::PI] {
                let matrix = stack_v2_transform((800.0, 600.0), (scale, rotation)).unwrap();
                near(point(matrix, 270.0, 150.0), (400.0, 300.0));
            }
        }
    }

    #[test]
    fn varied_stack_corners_follow_scale_and_rotation() {
        let matrix = stack_v2_transform((800.0, 600.0), (0.5, 0.0)).unwrap();
        near(point(matrix, 0.0, 0.0), (265.0, 225.0));
        near(point(matrix, 540.0, 300.0), (535.0, 375.0));
        let matrix =
            stack_v2_transform((800.0, 600.0), (2.0, std::f32::consts::FRAC_PI_2)).unwrap();
        for (corner, expected) in [
            ((0.0, 0.0), (700.0, -240.0)),
            ((540.0, 0.0), (700.0, 840.0)),
            ((540.0, 300.0), (100.0, 840.0)),
            ((0.0, 300.0), (100.0, -240.0)),
        ] {
            near(point(matrix, corner.0, corner.1), expected);
        }
    }

    #[test]
    fn varied_stack_rejects_invalid_or_overflowing_transforms() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(stack_v2_transform((invalid, 300.0), (1.0, 0.0)).is_err());
            assert!(stack_v2_transform((540.0, invalid), (1.0, 0.0)).is_err());
            assert!(stack_v2_transform((540.0, 300.0), (invalid, 0.0)).is_err());
            assert!(stack_v2_transform((540.0, 300.0), (1.0, invalid)).is_err());
        }
        for invalid in [0.0, -1.0] {
            assert!(stack_v2_transform((invalid, 300.0), (1.0, 0.0)).is_err());
            assert!(stack_v2_transform((540.0, invalid), (1.0, 0.0)).is_err());
            assert!(stack_v2_transform((540.0, 300.0), (invalid, 0.0)).is_err());
        }
        assert!(stack_v2_transform((540.0, 300.0), (f32::MAX, 0.0)).is_err());
    }
    /// Own hidden surface only: no screen capture or synthesized external input.
    #[test]
    #[ignore = "requires a native Windows desktop and software Direct2D"]
    fn native_temporal_alpha_and_transform_restore() {
        use navi_state::{
            reactions::Intensity,
            visual::{Family, TemporalMotion, VisualStyle},
        };
        use windows::Win32::System::Com::{
            CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED,
        };
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                w!("STATIC"),
                w!("NAVI temporal render test"),
                WS_POPUP,
                0,
                0,
                540,
                300,
                None,
                None,
                None,
                None,
            );
            let result = (|| -> Result<()> {
                if hwnd.0 == 0 {
                    return Err(Error::from_win32());
                }
                let r = Renderer::new(
                    96,
                    SurfaceSize {
                        width: 540.0,
                        height: 300.0,
                    },
                )?;
                let original = Matrix3x2 {
                    M11: 0.9,
                    M22: 0.9,
                    M31: 3.0,
                    M32: 7.0,
                    ..Default::default()
                };
                r.target.SetTransform(&original);
                let foreground = GetForegroundWindow();
                for family in [Family::StackV2, Family::Sutura] {
                    for intensity in [
                        Intensity::Subtle,
                        Intensity::Experimental,
                        Intensity::Anomalous,
                    ] {
                        let temporal = TemporalMotion::sample(42, intensity, true);
                        let style = VisualStyle {
                            seed: 42,
                            intensity,
                            temporal: Some(temporal),
                        };
                        for progress in [0.0, 0.5, 1.0] {
                            let frame = EffectFrame {
                                layers: 4,
                                mask: STACK,
                                phase: Phase::Processing,
                                progress,
                                opacity: temporal.envelope(progress),
                                motion: true,
                            };
                            r.draw_manifestation_varied(
                                hwnd,
                                frame,
                                family,
                                style,
                                (1.0, 0.0),
                                [(0.0, 0.0); 2],
                            )?;
                            let (zero, partial, opaque) = r.alpha_stats();
                            assert!(zero > 0 && opaque == 0);
                            assert_eq!(
                                partial > 0,
                                progress == 0.5,
                                "{family:?}/{intensity:?}/{progress}"
                            );
                            let mut restored = Matrix3x2::default();
                            r.target.GetTransform(&mut restored);
                            assert_eq!(
                                (restored.M11, restored.M22, restored.M31, restored.M32),
                                (original.M11, original.M22, original.M31, original.M32)
                            );
                        }
                    }
                }
                // The exact DIB must remain identical across time without motion,
                // even when callers supply a previously animated temporal style.
                for family in [Family::StackV2, Family::Sutura] {
                    let style = VisualStyle {
                        seed: 99,
                        intensity: Intensity::Anomalous,
                        temporal: Some(TemporalMotion::sample(99, Intensity::Anomalous, true)),
                    };
                    let mut previous = None;
                    for progress in [0.1, 0.5, 0.9] {
                        let frame = EffectFrame {
                            layers: 4,
                            mask: STACK,
                            phase: Phase::Processing,
                            progress,
                            opacity: 1.0,
                            motion: false,
                        };
                        r.draw_manifestation_varied(
                            hwnd,
                            frame,
                            family,
                            style,
                            (1.0, 0.0),
                            [(0.0, 0.0); 2],
                        )?;
                        let pixels = std::slice::from_raw_parts(
                            r.pixels as *const u8,
                            (r.width * r.height * 4) as usize,
                        )
                        .to_vec();
                        if let Some(old) = previous {
                            assert_eq!(pixels, old, "{family:?} static output changed");
                        }
                        previous = Some(pixels);
                    }
                }
                assert_eq!(GetForegroundWindow(), foreground);
                // Invalid input must leave the caller's transform intact as well.
                let frame = EffectFrame {
                    layers: 3,
                    mask: STACK,
                    phase: Phase::Processing,
                    progress: 0.5,
                    opacity: 1.0,
                    motion: true,
                };
                assert!(r
                    .draw_manifestation_varied(
                        hwnd,
                        frame,
                        Family::Sutura,
                        VisualStyle::default(),
                        (f32::NAN, 0.0),
                        [(0.0, 0.0); 2]
                    )
                    .is_err());
                let mut restored = Matrix3x2::default();
                r.target.GetTransform(&mut restored);
                assert_eq!(restored.M31, original.M31);
                Ok(())
            })();
            if hwnd.0 != 0 {
                let _ = DestroyWindow(hwnd);
            }
            CoUninitialize();
            result.unwrap();
        }
    }
}
