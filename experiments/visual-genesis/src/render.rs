use crate::experiment::{Frame, Ink, Mark, Point, HEIGHT, WIDTH};
use std::collections::HashMap;
use windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
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

struct Dib {
    dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
}
impl Drop for Dib {
    fn drop(&mut self) {
        unsafe {
            if self.old.0 != 0 {
                SelectObject(self.dc, self.old);
            }
            if self.bitmap.0 != 0 {
                let _ = DeleteObject(self.bitmap);
            }
            if self.dc.0 != 0 {
                let _ = DeleteDC(self.dc);
            }
        }
    }
}

pub struct Renderer {
    // COM references drop before the selected GDI bitmap and DC.
    target: ID2D1DCRenderTarget,
    factory: ID2D1Factory,
    write: IDWriteFactory,
    brush: ID2D1SolidColorBrush,
    formats: HashMap<(u32, bool), IDWriteTextFormat>,
    dib: Dib,
    width: i32,
    height: i32,
    canvas: (f32, f32),
}

fn color(ink: Ink, opacity: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: ((ink.0 >> 16) & 255) as f32 / 255.0,
        g: ((ink.0 >> 8) & 255) as f32 / 255.0,
        b: (ink.0 & 255) as f32 / 255.0,
        a: (ink.1 * opacity).clamp(0.0, 1.0),
    }
}
fn point(p: Point) -> D2D_POINT_2F {
    D2D_POINT_2F { x: p.0, y: p.1 }
}

impl Renderer {
    pub unsafe fn new(width: i32, height: i32) -> Result<Self> {
        Self::canvas(width, height, WIDTH, HEIGHT)
    }

    pub unsafe fn canvas(
        width: i32,
        height: i32,
        logical_width: f32,
        logical_height: f32,
    ) -> Result<Self> {
        if width <= 0 || height <= 0 {
            return Err(Error::new(E_INVALIDARG, "Invalid surface dimensions"));
        }
        let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let target = factory.CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            ..Default::default()
        })?;
        let dc = CreateCompatibleDC(None);
        if dc.0 == 0 {
            return Err(Error::from_win32());
        }
        let mut dib = Dib {
            dc,
            bitmap: HBITMAP::default(),
            old: HGDIOBJ::default(),
        };
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
        dib.bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, None, 0)?;
        dib.old = SelectObject(dc, dib.bitmap);
        if dib.old.0 == 0 || dib.old.0 == -1 {
            return Err(Error::from_win32());
        }
        target.BindDC(
            dc,
            &RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            },
        )?;
        target.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        let scale = (width as f32 / logical_width).min(height as f32 / logical_height);
        target.SetTransform(&Matrix3x2 {
            M11: scale,
            M12: 0.0,
            M21: 0.0,
            M22: scale,
            M31: (width as f32 - logical_width * scale) * 0.5,
            M32: (height as f32 - logical_height * scale) * 0.5,
        });
        let brush = target.CreateSolidColorBrush(&color(Ink(0xffffff, 1.0), 1.0), None)?;
        let write: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
        Ok(Self {
            target,
            factory,
            write,
            brush,
            formats: HashMap::new(),
            dib,
            width,
            height,
            canvas: (logical_width, logical_height),
        })
    }

    unsafe fn marks(&mut self, frame: &Frame) -> Result<()> {
        for mark in &frame.marks {
            match mark {
                Mark::Clip(r) => self.target.PushAxisAlignedClip(
                    &D2D_RECT_F {
                        left: r[0],
                        top: r[1],
                        right: r[2],
                        bottom: r[3],
                    },
                    D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                ),
                Mark::Unclip => self.target.PopAxisAlignedClip(),
                Mark::Path {
                    points,
                    closed,
                    fill,
                    stroke,
                } => {
                    let geometry = self.factory.CreatePathGeometry()?;
                    let sink = geometry.Open()?;
                    sink.SetFillMode(D2D1_FILL_MODE_WINDING);
                    sink.BeginFigure(
                        point(points[0]),
                        if fill.is_some() {
                            D2D1_FIGURE_BEGIN_FILLED
                        } else {
                            D2D1_FIGURE_BEGIN_HOLLOW
                        },
                    );
                    let vertices: Vec<_> = points[1..].iter().map(|p| point(*p)).collect();
                    sink.AddLines(&vertices);
                    sink.EndFigure(if *closed {
                        D2D1_FIGURE_END_CLOSED
                    } else {
                        D2D1_FIGURE_END_OPEN
                    });
                    sink.Close()?;
                    if let Some(ink) = fill {
                        self.brush.SetColor(&color(*ink, frame.opacity));
                        self.target.FillGeometry(&geometry, &self.brush, None);
                    }
                    if let Some((ink, width)) = stroke {
                        self.brush.SetColor(&color(*ink, frame.opacity));
                        self.target
                            .DrawGeometry(&geometry, &self.brush, *width, None);
                    }
                }
                Mark::Text {
                    value,
                    origin,
                    size,
                    ink,
                    serif,
                } => {
                    let key = (size.to_bits(), *serif);
                    // Experimental font sizes can change continuously: bound the cache.
                    if self.formats.len() > 160 {
                        self.formats.clear();
                    }
                    if !self.formats.contains_key(&key) {
                        let format = self.write.CreateTextFormat(
                            if *serif {
                                w!("Georgia")
                            } else {
                                w!("Consolas")
                            },
                            None,
                            DWRITE_FONT_WEIGHT_NORMAL,
                            DWRITE_FONT_STYLE_NORMAL,
                            DWRITE_FONT_STRETCH_NORMAL,
                            *size,
                            w!("es-AR"),
                        )?;
                        format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
                        self.formats.insert(key, format);
                    }
                    self.brush.SetColor(&color(*ink, frame.opacity));
                    self.target.DrawText(
                        &value.encode_utf16().collect::<Vec<_>>(),
                        &self.formats[&key],
                        &D2D_RECT_F {
                            left: origin.0,
                            top: origin.1,
                            right: self.canvas.0,
                            bottom: self.canvas.1,
                        },
                        &self.brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                        DWRITE_MEASURING_MODE_NATURAL,
                    );
                }
            }
        }
        Ok(())
    }

    pub unsafe fn draw(&mut self, hwnd: HWND, frame: &Frame) -> Result<()> {
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F::default()));
        let drawing = self.marks(frame);
        let end = self.target.EndDraw(None, None);
        drawing?;
        end?;
        UpdateLayeredWindow(
            hwnd,
            None,
            None,
            Some(&SIZE {
                cx: self.width,
                cy: self.height,
            }),
            self.dib.dc,
            Some(&POINT::default()),
            COLORREF(0),
            Some(&BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            }),
            ULW_ALPHA,
        )?;
        Ok(())
    }
}
