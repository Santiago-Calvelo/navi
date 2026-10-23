use navi_state::{Mode, Phase, Session};
pub mod composition;
pub mod context_fragment;
mod cursor_effects;
mod fractured_halo;
pub mod holograms;
mod layer_stack_v2;
mod sutura;
use navi_ui::ERROR;
use navi_ui::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            DirectWrite::*,
            Dxgi::Common::*,
            Gdi::*,
        },
        UI::WindowsAndMessaging::*,
    },
};

type Color = (u32, f32);

pub struct Renderer {
    target: ID2D1DCRenderTarget,
    small: IDWriteTextFormat,
    label: IDWriteTextFormat,
    title: IDWriteTextFormat,
    body: IDWriteTextFormat,
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    pixels: *mut core::ffi::c_void,
    pub width: i32,
    pub height: i32,
    scale: f32,
}

impl Renderer {
    /// # Safety
    /// The caller must keep a COM-initialized UI thread alive for this renderer's lifetime.
    pub unsafe fn new(dpi: u32, size: SurfaceSize) -> Result<Self> {
        let scale = dpi as f32 / 96.0;
        let width = (size.width * scale).round() as i32;
        let height = (size.height * scale).round() as i32;
        let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let target = factory.CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: dpi as f32,
            dpiY: dpi as f32,
            ..Default::default()
        })?;
        let write: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
        let format = |size| -> Result<IDWriteTextFormat> {
            let f = write.CreateTextFormat(
                w!("Cascadia Mono"),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("es-AR"),
            )?;
            f.SetWordWrapping(DWRITE_WORD_WRAPPING_WRAP)?;
            Ok(f)
        };
        let small = format(11.0)?;
        let label = format(13.0)?;
        let title = format(25.0)?;
        let body = format(14.0)?;
        let dc = CreateCompatibleDC(None);
        if dc.0 == 0 {
            return Err(Error::from_win32());
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = std::ptr::null_mut();
        let bitmap = match CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, None, 0) {
            Ok(bitmap) => bitmap,
            Err(e) => {
                let _ = DeleteDC(dc);
                return Err(e);
            }
        };
        let previous = SelectObject(dc, bitmap);
        let renderer = Self {
            target,
            small,
            label,
            title,
            body,
            dc,
            bitmap,
            previous,
            pixels,
            width,
            height,
            scale,
        };
        renderer.target.BindDC(
            dc,
            &RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            },
        )?;
        renderer
            .target
            .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        Ok(renderer)
    }

    unsafe fn brush(&self, c: Color) -> Result<ID2D1SolidColorBrush> {
        self.target.CreateSolidColorBrush(
            &D2D1_COLOR_F {
                r: ((c.0 >> 16) & 255) as f32 / 255.0,
                g: ((c.0 >> 8) & 255) as f32 / 255.0,
                b: (c.0 & 255) as f32 / 255.0,
                a: c.1,
            },
            None,
        )
    }

    unsafe fn rect(&self, r: [f32; 4], color: Color, fill: bool) -> Result<()> {
        let r = D2D_RECT_F {
            left: r[0],
            top: r[1],
            right: r[2],
            bottom: r[3],
        };
        let brush = self.brush(color)?;
        if fill {
            self.target.FillRectangle(&r, &brush);
        } else {
            self.target.DrawRectangle(&r, &brush, 0.7, None);
        }
        Ok(())
    }

    unsafe fn line(
        &self,
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        color: Color,
        width: f32,
    ) -> Result<()> {
        self.target.DrawLine(
            D2D_POINT_2F { x, y },
            D2D_POINT_2F { x: x2, y: y2 },
            &self.brush(color)?,
            width,
            None,
        );
        Ok(())
    }

    unsafe fn text(
        &self,
        text: &str,
        r: [f32; 4],
        color: Color,
        format: &IDWriteTextFormat,
    ) -> Result<()> {
        self.target.DrawText(
            &text.encode_utf16().collect::<Vec<_>>(),
            format,
            &D2D_RECT_F {
                left: r[0],
                top: r[1],
                right: r[2],
                bottom: r[3],
            },
            &self.brush(color)?,
            D2D1_DRAW_TEXT_OPTIONS_CLIP,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        Ok(())
    }

    /// # Safety
    /// `hwnd` must be a live layered window owned by the calling UI thread.
    pub unsafe fn draw(
        &self,
        hwnd: HWND,
        s: &Session,
        kind: SurfaceKind,
        interference: u8,
    ) -> Result<()> {
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F::default()));
        let result = match kind {
            SurfaceKind::Console => self.scene(s),
            SurfaceKind::Compact => self.compact(s, interference),
            SurfaceKind::Beacon => self.beacon(),
        };
        let end = self.target.EndDraw(None, None);
        result?;
        end?;
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

    unsafe fn beacon(&self) -> Result<()> {
        self.rect([1.0, 1.0, 87.0, 27.0], (0x080f16, 0.72), true)?;
        self.line(3.0, 26.0, 65.0, 26.0, (0x94c9bb, 0.32), 0.6)?;
        self.target.FillEllipse(
            &D2D1_ELLIPSE {
                point: D2D_POINT_2F { x: 12.0, y: 14.0 },
                radiusX: 2.0,
                radiusY: 2.0,
            },
            &self.brush(MINT)?,
        );
        self.text("navi_", [22.0, 6.0, 64.0, 24.0], MUTED, &self.small)?;
        Ok(())
    }

    unsafe fn compact(&self, s: &Session, interference: u8) -> Result<()> {
        let h = surface_size(SurfaceKind::Compact, s).height;
        let accent = if s.phase == Phase::Error { ERROR } else { MINT };
        // A quiet edge, no titlebar and no floating application chrome.
        self.rect([1.0, 1.0, 319.0, h - 1.0], INK, true)?;
        self.line(1.0, 1.0, 319.0, 1.0, (0x94c9bb, 0.30), 0.6)?;
        self.line(1.0, h - 1.0, 319.0, h - 1.0, (0x94c9bb, 0.21), 0.6)?;
        self.line(1.0, 1.0, 1.0, 27.0, accent, 1.0)?;
        self.text("navi_", [16.0, 10.0, 90.0, 28.0], accent, &self.small)?;
        self.text(
            &compact_message(s),
            [17.0, 34.0, 302.0, h - 42.0],
            TEXT,
            &self.body,
        )?;
        self.text(
            compact_status(s),
            [17.0, h - 24.0, 204.0, h - 6.0],
            MUTED,
            &self.small,
        )?;
        if s.keyboard_entry.is_some() {
            self.text(
                "esc / ocultar",
                [219.0, h - 24.0, 308.0, h - 6.0],
                MUTED,
                &self.small,
            )?;
        } else if matches!(s.phase, Phase::Listening | Phase::Processing) {
            for i in 0..28 {
                let x = 205.0 + i as f32 * 3.2;
                let height = navi_demo::amplitude(i * 3, s.frame, true) * 0.24;
                self.line(x, h - 17.0 - height, x, h - 17.0 + height, accent, 0.8)?;
            }
        } else {
            self.text(
                if s.awaiting_confirmation {
                    "confirmar"
                } else if !s.answer.is_empty() {
                    "04s"
                } else if s.phase == Phase::Responding {
                    "04s / demo"
                } else {
                    "F6 / demo"
                },
                [219.0, h - 24.0, 308.0, h - 6.0],
                MUTED,
                &self.small,
            )?;
        }
        if interference > 0 {
            let y = 28.0 + interference as f32 * 9.0;
            self.line(160.0, y, 245.0, y, (0x94c9bb, 0.17), 0.7)?;
            self.line(18.0, y + 2.0, 55.0, y + 2.0, (0x94c9bb, 0.12), 0.7)?;
        }
        Ok(())
    }

    unsafe fn scene(&self, s: &Session) -> Result<()> {
        // Offset panes are geometry within the same HWND; transparent outer pixels remain click-through.
        self.rect([9.5, 9.5, 716.5, 389.5], (0x132824, 0.12), true)?;
        self.rect([9.5, 9.5, 716.5, 389.5], (0x9dc9be, 0.24), false)?;
        self.rect([38.5, 39.5, 750.5, 419.5], (0x8bb5aa, 0.18), false)?;
        self.rect([24.5, 24.5, 735.5, 405.5], INK, true)?;
        self.rect([25.0, 25.0, 34.0, 405.0], (0xa4e1cf, 0.06), true)?;
        self.rect([24.5, 24.5, 735.5, 405.5], LINE, false)?;
        self.line(25.0, 25.0, 185.0, 25.0, (0xd5f6eb, 0.76), 1.0)?;
        self.line(25.0, 25.0, 25.0, 76.0, MINT, 1.0)?;
        self.line(736.0, 353.0, 736.0, 405.0, MINT, 1.0)?;
        self.line(674.0, 405.0, 736.0, 405.0, MINT, 1.0)?;
        self.text(
            "NAVI / SESSION 001",
            [48.0, 42.0, 420.0, 65.0],
            TEXT,
            &self.label,
        )?;
        self.line(38.0, 73.0, 722.0, 73.0, LINE, 0.6)?;
        self.text(
            "VOZ",
            [51.0, 88.0, 109.0, 110.0],
            if s.mode == Mode::Voice { MINT } else { MUTED },
            &self.label,
        )?;
        self.text(
            "TEXTO",
            [137.0, 88.0, 206.0, 110.0],
            if s.mode == Mode::Text { MINT } else { MUTED },
            &self.label,
        )?;
        self.text(
            "LOCAL / DEMO",
            [603.0, 90.0, 704.0, 110.0],
            MINT,
            &self.small,
        )?;
        self.target.FillEllipse(
            &D2D1_ELLIPSE {
                point: D2D_POINT_2F { x: 714.0, y: 97.0 },
                radiusX: 3.2,
                radiusY: 3.2,
            },
            &self.brush(if s.phase == Phase::Error { ERROR } else { MINT })?,
        );
        self.line(38.0, 118.0, 722.0, 118.0, LINE, 0.6)?;
        let tab = if s.mode == Mode::Voice { 48.0 } else { 133.0 };
        self.line(tab, 118.0, tab + 62.0, 118.0, MINT, 1.3)?;
        self.text(
            headline(s),
            [66.0, 181.0, 695.0, 251.0],
            if s.phase == Phase::Error { ERROR } else { TEXT },
            &self.title,
        )?;
        self.text(caption(s), [67.0, 257.0, 698.0, 292.0], MUTED, &self.small)?;
        for i in 0..130u32 {
            let x = 68.0 + i as f32 * 4.8;
            let h = navi_demo::amplitude(i, s.frame, s.animated());
            self.line(
                x,
                324.0 - h,
                x,
                324.0 + h * 0.70,
                (0xb5e5d5, if i < 90 { 0.76 } else { 0.30 }),
                1.0,
            )?;
        }
        self.line(38.0, 361.0, 722.0, 361.0, LINE, 0.6)?;
        self.text(
            "[ enter ] enviar   [ ctrl + k ] voz   [ esc ] ocultar",
            [49.0, 380.0, 626.0, 399.0],
            MUTED,
            &self.small,
        )?;
        self.text("F1–F5", [668.0, 380.0, 726.0, 400.0], MINT, &self.small)?;
        Ok(())
    }

    /// Exports the actual premultiplied backing surface for reproducible visual QA.
    ///
    /// # Safety
    /// Call on the renderer's owning thread after a completed draw, with no concurrent writes.
    pub unsafe fn snapshot(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let length = (self.width * self.height * 4) as u32;
        let mut file = std::fs::File::create(path)?;
        file.write_all(b"BM")?;
        file.write_all(&(54 + length).to_le_bytes())?;
        file.write_all(&[0; 4])?;
        file.write_all(&54u32.to_le_bytes())?;
        file.write_all(&40u32.to_le_bytes())?;
        file.write_all(&self.width.to_le_bytes())?;
        file.write_all(&(-self.height).to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&32u16.to_le_bytes())?;
        file.write_all(&[0; 24])?;
        file.write_all(std::slice::from_raw_parts(
            self.pixels as *const u8,
            length as usize,
        ))
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(self.bitmap);
            let _ = DeleteDC(self.dc);
        }
    }
}
