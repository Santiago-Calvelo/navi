//! Calm, opaque writing surface. The native Edit control supplies its own caret.
use super::*;

impl Renderer {
    /// # Safety
    /// Paint DC and renderer must belong to the current UI thread.
    pub unsafe fn paint_user_panel(&self, dc: HDC, busy: bool, focused: bool) -> Result<()> {
        let w = self.width as f32 / self.scale;
        let h = self.height as f32 / self.scale;
        self.target.BeginDraw();
        self.target.Clear(Some(&D2D1_COLOR_F {
            r: 25.0 / 255.0,
            g: 29.0 / 255.0,
            b: 41.0 / 255.0,
            a: 1.0,
        }));
        let drawn = (|| -> Result<()> {
            self.rect([0.5, 0.5, w - 0.5, h - 0.5], (0x73808c, 0.55), false)?;
            self.text(
                "Tu espacio",
                [24.0, 15.0, 220.0, 37.0],
                (0xabb2bd, 1.0),
                &self.small,
            )?;
            self.text(
                "Escribile a NAVI",
                [24.0, 40.0, w - 24.0, 77.0],
                (0xeceee9, 1.0),
                &self.title,
            )?;
            self.rect(
                [23.0, 89.0, w - 23.0, 137.0],
                (if focused { 0xb7e3d6 } else { 0x667283 }, 1.0),
                false,
            )?;
            self.text(
                if busy {
                    "NAVI está trabajando. Podés seguir escribiendo."
                } else if focused {
                    "Listo para escribir. Ej.: abrí la calculadora"
                } else {
                    "Hacé clic en el campo para escribir"
                },
                [24.0, 147.0, w - 139.0, 186.0],
                (0xb7c4c5, 1.0),
                &self.small,
            )?;
            self.line(24.0, h - 44.0, w - 24.0, h - 44.0, (0x64707f, 0.4), 0.7)?;
            self.text(
                "Enter: enviar   Esc: ocultar",
                [24.0, h - 31.0, w - 24.0, h - 8.0],
                (0xaab3c0, 1.0),
                &self.small,
            )?;
            Ok(())
        })();
        let end = self.target.EndDraw(None, None);
        drawn?;
        end?;
        BitBlt(dc, 0, 0, self.width, self.height, self.dc, 0, 0, SRCCOPY)
    }
}
