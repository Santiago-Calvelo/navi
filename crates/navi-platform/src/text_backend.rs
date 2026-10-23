//! User-triggered single-flight work; no tool runs on the Win32 message thread.
use super::*;
use navi_agent::{
    model::{ErrorCode, Response, Status, Timings, ToolError},
    tools::ToolHost,
    Agent,
};
use std::{
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
};
use windows::Win32::System::SystemInformation::{GetLocalTime, GetSystemDirectoryW};

pub(super) const MESSAGE: u32 = WM_APP + 41;

#[derive(Default)]
pub(super) struct TextBackend {
    agent: Option<Agent>,
    receiver: Option<Receiver<(Agent, Response)>>,
    alive: Arc<AtomicBool>,
    pub latest: Option<Response>,
}

impl TextBackend {
    pub(super) fn strict_for_diagnostics() -> Self {
        let mut agent = Agent::default();
        agent.permissions.confirm_launch = true;
        let mut backend = Self::default();
        backend.agent = Some(agent);
        backend
    }

    fn busy(&self) -> bool {
        self.receiver.is_some()
    }

    fn start(&mut self, hwnd: HWND, text: String, clock: Instant) -> std::io::Result<()> {
        self.latest = None;
        let mut agent = self.agent.take().unwrap_or_default();
        let (tx, rx) = mpsc::sync_channel(1);
        let alive = self.alive.clone();
        alive.store(true, Ordering::Release);
        let sink = hwnd.0;
        std::thread::Builder::new()
            .name("navi-text".into())
            .spawn(move || {
                let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    agent.handle(&text, &mut WindowsHost, clock.elapsed().as_millis() as u64)
                }))
                .unwrap_or_else(|_| backend_error("Falló la ejecución de la herramienta."));
                if tx.send((agent, response)).is_ok() && alive.load(Ordering::Acquire) {
                    unsafe {
                        let _ = PostMessageW(HWND(sink), MESSAGE, WPARAM(0), LPARAM(0));
                    }
                }
            })?;
        self.receiver = Some(rx);
        Ok(())
    }

    fn receive(&mut self) -> Option<Response> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok((agent, response)) => {
                self.agent = Some(agent);
                self.receiver = None;
                self.latest = Some(response.clone());
                Some(response)
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                self.agent = Some(Agent::default());
                let response =
                    backend_error("Se interrumpió la tarea. La sesión textual fue restablecida.");
                self.latest = Some(response.clone());
                Some(response)
            }
        }
    }
}

impl Drop for TextBackend {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Release);
    }
}

pub(super) unsafe fn submit(hwnd: HWND) {
    commit(hwnd, |app| {
        if app.text.busy() || app.session.input.trim().is_empty() {
            return;
        }
        app.session.apply(Event::BeginRequest);
        app.surfaces.protect(&app.session);
        app.surfaces.present(app.session.phase, app.now());
        app.effects.phase_changed(app.session.phase, app.now());
        let clock = app.started;
        if app
            .text
            .start(hwnd, app.session.submitted.clone(), clock)
            .is_err()
        {
            app.session.apply(Event::CompleteRequest {
                text: "No se pudo iniciar la tarea; la sesión textual fue restablecida.".into(),
                confirmation: false,
                failed: true,
            });
            app.surfaces.protect(&app.session);
            app.surfaces.present(app.session.phase, app.now());
            app.effects.phase_changed(app.session.phase, app.now());
        }
    });
}

pub(super) unsafe fn complete(hwnd: HWND) -> bool {
    let response = with_app(|a| a.text.receive()).flatten();
    let Some(response) = response else {
        return false;
    };
    commit(hwnd, |a| {
        let was_hidden = a.surfaces.kind == SurfaceKind::Beacon;
        let is_system_status =
            response.tool.as_deref() == Some("system_status") && response.status == Status::Success;
        a.session.apply(Event::CompleteRequest {
            text: response.text,
            confirmation: response.status == Status::NeedsConfirmation,
            failed: matches!(response.status, Status::Error | Status::Rejected),
        });
        a.context_entries = if is_system_status {
            response
                .data
                .iter()
                .filter(|(key, _)| key.starts_with("cpu_app_"))
                .map(|(_, value)| {
                    value.split_once('|').map_or_else(
                        || ("Aplicación".into(), value.clone()),
                        |(name, cpu)| {
                            let label = name.trim_end_matches(".exe").replace('_', " ");
                            let readable_cpu = if cpu == "no disponible" {
                                "CPU no disponible".into()
                            } else {
                                format!("{cpu} % CPU").replace('.', ",")
                            };
                            (label, readable_cpu)
                        },
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        a.surfaces.protect(&a.session);
        if is_system_status && a.surfaces.kind != SurfaceKind::Console {
            a.surfaces.idle();
        } else if !was_hidden {
            a.surfaces.present(a.session.phase, a.now());
        }
        if is_system_status || !was_hidden {
            if !is_system_status {
                a.effects.phase_changed(a.session.phase, a.now());
            }
            let show_action_halo = matches!(response.status, Status::Success | Status::Error)
                && response.tool.as_deref() != Some("system_status");
            if show_action_halo {
                a.surfaces.clear_focus_effect();
                a.effects.clear();
                a.action_visual
                    .start(a.now(), response.status == Status::Error);
                a.action_surface = action_surface::focused()
                    .map(|(work, dpi)| a.surfaces.result_surface(work, dpi, a.action_visual.seed));
            }
            if !a.context_entries.is_empty() {
                a.surfaces.clear_focus_effect();
                a.action_visual.clear();
                a.effects.clear();
                a.effects.preview_timed(
                    navi_state::holograms::CONTEXT,
                    Phase::Responding,
                    a.now(),
                    5_600,
                );
            }
        }
    });
    true
}

fn backend_error(text: &str) -> Response {
    Response {
        request_id: 0,
        status: Status::Error,
        text: text.into(),
        tool: None,
        data: Vec::new(),
        confirmation: None,
        error: Some(ToolError::new(ErrorCode::ToolFailed, text)),
        timings: Timings::default(),
    }
}

pub(super) struct WindowsHost;

impl ToolHost for WindowsHost {
    fn current_time(&mut self) -> std::result::Result<String, ToolError> {
        let time = unsafe { GetLocalTime() };
        Ok(format!(
            "{:02}:{:02}:{:02}",
            time.wHour, time.wMinute, time.wSecond
        ))
    }
    fn system_status(&mut self) -> std::result::Result<Vec<(String, String)>, ToolError> {
        let threads = std::thread::available_parallelism().map_err(|_| {
            ToolError::new(ErrorCode::ToolFailed, "No se pudo consultar el sistema.")
        })?;
        let mut data = vec![
            ("sistema".into(), "Windows".into()),
            ("arquitectura".into(), std::env::consts::ARCH.into()),
            ("procesadores lógicos".into(), threads.to_string()),
        ];
        for (index, (name, cpu)) in system_status::applications()?.into_iter().enumerate() {
            let cpu = cpu.map_or_else(|| "no disponible".into(), |cpu| format!("{cpu:.1}"));
            data.push((format!("cpu_app_{index}"), format!("{name}|{cpu}")));
        }
        Ok(data)
    }
    fn open_calculator(&mut self) -> std::result::Result<(), ToolError> {
        // GetSystemDirectory supplies an absolute OS directory; neither the
        // executable nor arguments are derived from user text or PATH lookup.
        let mut directory = [0u16; 32768];
        let length = unsafe { GetSystemDirectoryW(Some(&mut directory)) } as usize;
        if length == 0 || length >= directory.len() {
            return Err(ToolError::new(
                ErrorCode::ToolFailed,
                "No se pudo localizar el directorio del sistema.",
            ));
        }
        use std::os::windows::ffi::OsStringExt;
        let executable =
            std::path::PathBuf::from(std::ffi::OsString::from_wide(&directory[..length]))
                .join("calc.exe");
        Command::new(executable)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|_| {
                ToolError::new(
                    ErrorCode::ToolFailed,
                    "Windows no pudo iniciar la calculadora.",
                )
            })
    }
}
