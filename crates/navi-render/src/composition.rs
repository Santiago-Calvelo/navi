//! Isolated D3D11/DXGI/DirectComposition proof, not a dependency of daily UI.
use std::{io::Write, path::Path};
use windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            Direct3D::*,
            Direct3D11::*,
            DirectComposition::*,
            Dxgi::{Common::*, *},
        },
    },
};

pub struct CompositionProbe {
    _device: ID3D11Device,
    composition: IDCompositionDevice,
    _target: IDCompositionTarget,
    _visual: IDCompositionVisual,
    _surface: IDCompositionSurface,
    pub hardware: bool,
    pub alpha: [u8; 4],
}

impl CompositionProbe {
    /// # Safety
    /// Requires a COM-initialized UI thread and a live, non-redirection HWND.
    pub unsafe fn new(hwnd: HWND, path: &Path) -> Result<Self> {
        let (mut device, mut context) = (None, None);
        let hardware = D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
        .is_ok();
        if !hardware {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_WARP,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
        }
        let device = device.unwrap();
        let context = context.unwrap();
        let dxgi: IDXGIDevice = device.cast()?;
        let composition: IDCompositionDevice = DCompositionCreateDevice(&dxgi)?;
        let target = composition.CreateTargetForHwnd(hwnd, true)?;
        let visual = composition.CreateVisual()?;
        let surface = composition.CreateSurface(
            240,
            180,
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_ALPHA_MODE_PREMULTIPLIED,
        )?;
        let mut offset = POINT::default();
        let dxgi_surface: IDXGISurface = surface.BeginDraw(None, &mut offset)?;
        let drawing = (|| -> Result<[u8; 4]> {
            let d2d = D2D1CreateDevice(&dxgi, None)?;
            let dc = d2d.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            let bitmap = dc.CreateBitmapFromDxgiSurface(
                &dxgi_surface,
                Some(&D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                    ..Default::default()
                }),
            )?;
            dc.SetTarget(&bitmap);
            dc.SetTransform(&Matrix3x2 {
                M11: 1.0,
                M22: 1.0,
                M31: offset.x as f32,
                M32: offset.y as f32,
                ..Default::default()
            });
            dc.BeginDraw();
            dc.PushAxisAlignedClip(
                &D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: 240.0,
                    bottom: 180.0,
                },
                D2D1_ANTIALIAS_MODE_ALIASED,
            );
            dc.Clear(Some(&D2D1_COLOR_F::default()));
            for (x, a) in [(50.0, 0.2), (120.0, 0.5), (190.0, 1.0)] {
                let brush = dc.CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 0.65,
                        g: 0.85,
                        b: 0.78,
                        a,
                    },
                    None,
                )?;
                dc.FillEllipse(
                    &D2D1_ELLIPSE {
                        point: D2D_POINT_2F { x, y: 90.0 },
                        radiusX: 25.0,
                        radiusY: 25.0,
                    },
                    &brush,
                );
            }
            dc.PopAxisAlignedClip();
            dc.EndDraw(None, None)?;
            dc.SetTarget(None);
            let source: ID3D11Texture2D = dxgi_surface.cast()?;
            let mut staging = None;
            device.CreateTexture2D(
                &D3D11_TEXTURE2D_DESC {
                    Width: 240,
                    Height: 180,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Usage: D3D11_USAGE_STAGING,
                    CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                    ..Default::default()
                },
                None,
                Some(&mut staging),
            )?;
            let staging = staging.unwrap();
            context.CopySubresourceRegion(
                &staging,
                0,
                0,
                0,
                0,
                &source,
                0,
                Some(&D3D11_BOX {
                    left: offset.x as u32,
                    top: offset.y as u32,
                    front: 0,
                    right: offset.x as u32 + 240,
                    bottom: offset.y as u32 + 180,
                    back: 1,
                }),
            );
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;
            let mut pixels = Vec::with_capacity(240 * 180 * 4);
            for y in 0..180 {
                pixels.extend_from_slice(std::slice::from_raw_parts(
                    (mapped.pData as *const u8).add(y * mapped.RowPitch as usize),
                    240 * 4,
                ));
            }
            context.Unmap(&staging, 0);
            let alpha = [
                pixels[3],
                pixels[(90 * 240 + 50) * 4 + 3],
                pixels[(90 * 240 + 120) * 4 + 3],
                pixels[(90 * 240 + 190) * 4 + 3],
            ];
            let mut file =
                std::fs::File::create(path).map_err(|e| Error::new(E_FAIL, e.to_string()))?;
            let length = pixels.len() as u32;
            let saved = (|| -> std::io::Result<()> {
                file.write_all(b"BM")?;
                file.write_all(&(54 + length).to_le_bytes())?;
                file.write_all(&[0; 4])?;
                file.write_all(&54u32.to_le_bytes())?;
                file.write_all(&40u32.to_le_bytes())?;
                file.write_all(&240i32.to_le_bytes())?;
                file.write_all(&(-180i32).to_le_bytes())?;
                file.write_all(&1u16.to_le_bytes())?;
                file.write_all(&32u16.to_le_bytes())?;
                file.write_all(&[0; 24])?;
                file.write_all(&pixels)
            })();
            saved.map_err(|e| Error::new(E_FAIL, e.to_string()))?;
            Ok(alpha)
        })();
        let end = surface.EndDraw();
        let alpha = drawing?;
        end?;
        visual.SetContent(&surface)?;
        target.SetRoot(&visual)?;
        composition.Commit()?;
        composition.WaitForCommitCompletion()?;
        if alpha[0] != 0
            || !(49..=53).contains(&alpha[1])
            || !(126..=130).contains(&alpha[2])
            || alpha[3] != 255
        {
            return Err(Error::new(
                E_FAIL,
                format!("Unexpected alpha readback: {alpha:?}"),
            ));
        }
        Ok(Self {
            _device: device,
            composition,
            _target: target,
            _visual: visual,
            _surface: surface,
            hardware,
            alpha,
        })
    }
}

impl Drop for CompositionProbe {
    fn drop(&mut self) {
        unsafe {
            let _ = self.composition.Commit();
        }
    }
}
