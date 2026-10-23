use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::{Shell::*, WindowsAndMessaging::*},
    },
};

pub const CALLBACK: u32 = WM_APP + 30;

pub struct Tray {
    data: NOTIFYICONDATAW,
}

impl Tray {
    pub unsafe fn new(hwnd: HWND) -> Result<Self> {
        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: CALLBACK,
            hIcon: LoadIconW(None, IDI_APPLICATION)?,
            ..Default::default()
        };
        for (slot, unit) in data
            .szTip
            .iter_mut()
            .zip("Escribir a NAVI · Ctrl+Alt+Espacio".encode_utf16())
        {
            *slot = unit;
        }
        Shell_NotifyIconW(NIM_ADD, &data).ok()?;
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        let _ = Shell_NotifyIconW(NIM_SETVERSION, &data);
        Ok(Self { data })
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.data);
        }
    }
}
