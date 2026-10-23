mod action_surface;
mod cursor_observer;
mod cursor_qa;
mod desktop_observer;
mod focus_reaction;
mod holograms;
mod keyboard_entry;
mod manifestation_qa;
mod monitor_catalog;
mod organic;
mod placement;
mod pointer_reaction;
mod reaction_options;
pub mod surface;
mod system_status;
mod text_backend;
mod tray;
mod verification;

use navi_render::Renderer;
use navi_state::{Event, Mode, Phase, Session};
use navi_ui::{Hit, SurfaceKind, SurfaceSize};
use std::{cell::RefCell, path::PathBuf, time::Instant};
use surface::{Rect, SurfaceManager};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, LibraryLoader::*},
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};

struct App {
    action_visual: navi_state::action_visual::ActionVisual,
    action_surface: Option<action_surface::ActionSurface>,
    text: text_backend::TextBackend,
    demo_submission: bool,
    observer: Option<desktop_observer::DesktopObserver>,
    cursor: Option<cursor_observer::CursorObserver>,
    passive_observers: bool,
    reactions: navi_state::reactions::ReactionEngine,
    effects: navi_state::holograms::Holograms,
    overlay: Option<holograms::Overlay>,
    tray: Option<tray::Tray>,
    taskbar_created: u32,
    session: Session,
    surfaces: SurfaceManager,
    renderer: Option<Renderer>,
    render_key: Option<(u32, i32, i32)>,
    work: Rect,
    dpi: u32,
    started: Instant,
    previous_focus: HWND,
    interference: u8,
    interference_until: Option<u64>,
    timers: [bool; 4],
    hotkeys: [bool; 3],
    effect_hotkeys: [bool; 4],
    failure: Option<String>,
    surrogate: Option<u16>,
    context_entries: Vec<(String, String)>,
}

impl App {
    fn now(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
}

thread_local! { static APP: RefCell<Option<App>> = const { RefCell::new(None) }; }

// Win32 sizing/showing/render calls may synchronously re-enter the WndProc.
// Temporarily take ownership instead of holding a RefCell borrow across them.
fn with_app<T>(action: impl FnOnce(&mut App) -> T) -> Option<T> {
    let mut app = APP.with(|slot| slot.borrow_mut().take())?;
    let result = action(&mut app);
    APP.with(|slot| *slot.borrow_mut() = Some(app));
    Some(result)
}

unsafe fn monitor_work(hwnd: HWND) -> Result<Rect> {
    let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    GetMonitorInfoW(monitor, &mut info).ok()?;
    Ok(Rect {
        left: info.rcWork.left,
        top: info.rcWork.top,
        right: info.rcWork.right,
        bottom: info.rcWork.bottom,
    })
}

unsafe fn restore_focus(hwnd: HWND, app: &App) {
    if GetForegroundWindow() == hwnd
        && app.previous_focus.0 != 0
        && IsWindow(app.previous_focus).as_bool()
    {
        let _ = SetForegroundWindow(app.previous_focus);
    }
}

unsafe fn sync(hwnd: HWND, app: &mut App) -> Result<()> {
    keyboard_entry::sync(hwnd, &app.session, app.surfaces.kind, app.now());
    app.interference = app.interference_until.map_or(0, |end| {
        end.saturating_sub(app.now()).div_ceil(40).min(4) as u8
    });
    if app.interference == 0 {
        app.interference_until = None;
    }
    let plan = app.surfaces.plan(app.work, app.dpi, &app.session);
    for id in 1..=4 {
        let _ = KillTimer(hwnd, id);
    }
    app.timers = [false; 4];
    if !plan.visible {
        restore_focus(hwnd, app);
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_HIDEWINDOW,
        )?;
        return Ok(());
    }
    let width = plan.bounds.right - plan.bounds.left;
    let height = plan.bounds.bottom - plan.bounds.top;
    let topmost = plan.kind != SurfaceKind::Console || app.session.pinned;
    if plan.kind == SurfaceKind::Beacon {
        restore_focus(hwnd, app);
    }
    let mut current = RECT::default();
    GetWindowRect(hwnd, &mut current)?;
    let was_topmost = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 != 0;
    if current.left != plan.bounds.left
        || current.top != plan.bounds.top
        || current.right - current.left != width
        || current.bottom - current.top != height
        || was_topmost != topmost
    {
        SetWindowPos(
            hwnd,
            if topmost {
                HWND_TOPMOST
            } else {
                HWND_NOTOPMOST
            },
            plan.bounds.left,
            plan.bounds.top,
            width,
            height,
            SWP_NOACTIVATE,
        )?;
    }
    let key = (app.dpi, width, height);
    let size = SurfaceSize {
        width: width as f32 * 96.0 / app.dpi as f32,
        height: height as f32 * 96.0 / app.dpi as f32,
    };
    if app.render_key != Some(key) || app.renderer.is_none() {
        app.renderer = Some(Renderer::new(app.dpi, size)?);
        app.render_key = Some(key);
    }
    if app
        .renderer
        .as_ref()
        .unwrap()
        .draw(hwnd, &app.session, plan.kind, app.interference)
        .is_err()
    {
        app.renderer = Some(Renderer::new(app.dpi, size)?);
        app.renderer
            .as_ref()
            .unwrap()
            .draw(hwnd, &app.session, plan.kind, app.interference)?;
    }
    if !IsWindowVisible(hwnd).as_bool() {
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )?;
    }
    if app.session.animated() && plan.kind != SurfaceKind::Beacon {
        if SetTimer(hwnd, 1, 50, None) == 0 {
            return Err(Error::from_win32());
        }
        app.timers[0] = true;
    }
    if let Some(remaining) = app.surfaces.remaining(app.now()) {
        if SetTimer(hwnd, 2, remaining, None) == 0 {
            return Err(Error::from_win32());
        }
        app.timers[1] = true;
    }
    if app.interference > 0 && plan.kind == SurfaceKind::Compact {
        if SetTimer(hwnd, 3, 40, None) == 0 {
            return Err(Error::from_win32());
        }
        app.timers[2] = true;
    }
    holograms::sync(hwnd, app);
    keyboard_entry::sync(hwnd, &app.session, plan.kind, app.now());
    Ok(())
}

unsafe fn commit(hwnd: HWND, action: impl FnOnce(&mut App)) {
    let error = with_app(|app| {
        action(app);
        sync(hwnd, app).err()
    })
    .flatten();
    if let Some(error) = error {
        with_app(|app| app.failure = Some(error.to_string()));
        let _ = DestroyWindow(hwnd);
    }
}

unsafe fn update(hwnd: HWND, event: Event) {
    commit(hwnd, |app| {
        let old = app.session.phase;
        app.session.apply(event);
        app.surfaces.protect(&app.session);
        if app.session.phase != old && matches!(app.session.phase, Phase::Invoked | Phase::Error) {
            app.interference = 4;
            app.interference_until = Some(app.now() + 160);
        }
        app.surfaces.present(app.session.phase, app.now());
        if app.session.phase != old {
            app.effects.phase_changed(app.session.phase, app.now());
        }
    });
}

unsafe fn invoke(hwnd: HWND, focus: bool) {
    if !with_app(|a| a.demo_submission).unwrap_or(true) {
        keyboard_entry::invoke(hwnd, focus);
        return;
    }
    let previous = GetForegroundWindow();
    commit(hwnd, |app| {
        focus_reaction::cancel(&mut app.reactions, &mut app.surfaces);
        if previous != hwnd {
            app.previous_focus = previous;
        }
        if app.session.input.is_empty() && !app.session.animated() {
            app.session.apply(Event::Reset);
        }
        if !app.session.animated() {
            app.session.apply(Event::SetPhase(Phase::Invoked));
        }
        app.surfaces.kind = SurfaceKind::Compact;
        app.surfaces.protect(&app.session);
        app.surfaces.present(app.session.phase, app.now());
        app.interference = 4;
        app.interference_until = Some(app.now() + 160);
    });
    if focus {
        let _ = SetForegroundWindow(hwnd);
    }
}

unsafe fn hide(hwnd: HWND) {
    keyboard_entry::cancel(hwnd);
    commit(hwnd, |app| {
        focus_reaction::cancel(&mut app.reactions, &mut app.surfaces);
        app.surfaces.hide();
        app.effects.clear();
        app.action_visual.clear();
        app.interference = 0;
        app.interference_until = None;
    });
}

unsafe fn toggle_console(hwnd: HWND) {
    keyboard_entry::cancel(hwnd);
    let previous = GetForegroundWindow();
    commit(hwnd, |app| {
        focus_reaction::cancel(&mut app.reactions, &mut app.surfaces);
        if previous != hwnd {
            app.previous_focus = previous;
        }
        if app.session.phase == Phase::Idle {
            app.session.apply(Event::SetPhase(Phase::Invoked));
            app.surfaces.present(Phase::Invoked, app.now());
        }
        app.surfaces.toggle_console(app.now());
    });
    let _ = SetForegroundWindow(hwnd);
}

unsafe fn cycle_demo(hwnd: HWND) {
    let next = with_app(|app| navi_demo::next_phase(app.session.phase));
    if let Some(next) = next {
        update(hwnd, Event::SetPhase(next));
    }
}

unsafe fn current_hit(hwnd: HWND, lp: LPARAM) -> Hit {
    with_app(|app| {
        let scale = app.dpi as f32 / 96.0;
        navi_ui::surface_hit(
            app.surfaces.kind,
            navi_ui::surface_size(app.surfaces.kind, &app.session),
            lp.0 as i16 as f32 / scale,
            (lp.0 >> 16) as i16 as f32 / scale,
        )
    })
    .unwrap_or_else(|| {
        let _ = hwnd;
        Hit::None
    })
}

unsafe fn preview_effect(hwnd: HWND, mask: u8) {
    if mask & navi_state::holograms::STACK != 0 && !with_app(|a| a.demo_submission).unwrap_or(false)
    {
        return;
    }
    if mask == navi_state::holograms::STACK {
        reaction_options::preview(hwnd, navi_state::visual::Family::StackV2);
        return;
    }
    commit(hwnd, |app| {
        focus_reaction::cancel(&mut app.reactions, &mut app.surfaces);
        let phase = if matches!(app.session.phase, Phase::Idle | Phase::Invoked) {
            Phase::Listening
        } else {
            app.session.phase
        };
        app.effects.preview(mask, phase, app.now());
    });
}

unsafe fn menu(hwnd: HWND) {
    let Ok(menu) = CreatePopupMenu() else {
        return;
    };
    let _ = AppendMenuW(menu, MF_STRING, 10, w!("Escribir · Ctrl+Alt+Espacio"));
    let _ = AppendMenuW(menu, MF_STRING, 11, w!("Consola · Ctrl+Alt+N"));
    let _ = AppendMenuW(menu, MF_STRING, 12, w!("Contraer a presencia mínima · Esc"));
    let _ = AppendMenuW(menu, MF_STRING, 13, w!("Siempre visible · consola"));
    let (focus_enabled, observer_ready) = with_app(|app| {
        (
            app.reactions.config.enabled && app.reactions.config.events & 1 != 0,
            app.observer.is_some(),
        )
    })
    .unwrap_or((false, false));
    let focus_flags = MF_STRING
        | if !observer_ready {
            MF_GRAYED
        } else if focus_enabled {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
    let _ = AppendMenuW(menu, focus_flags, 50, w!("Reaccionar a cambios de foco"));
    reaction_options::append(menu);
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        30,
        w!("SignalField · diagnóstico · sin uso asignado"),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        31,
        w!("ContextFragment · diagnóstico simulado"),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        39,
        w!("Posición: junto a NAVI / superior izquierda"),
    );
    let _ = AppendMenuW(menu, MF_STRING, 36, w!("Animación: activar/desactivar"));
    let _ = AppendMenuW(menu, MF_STRING, 38, w!("Desactivar todos los efectos"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    for (id, title) in [
        (20, w!("Demo · reposo")),
        (21, w!("Demo · invocación")),
        (22, w!("Demo · escucha")),
        (23, w!("Demo · procesamiento")),
        (24, w!("Demo · respuesta")),
        (25, w!("Demo · error")),
    ] {
        let _ = AppendMenuW(menu, MF_STRING, id, title);
    }
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, 99, w!("Salir de NAVI"));
    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);
    let _ = SetForegroundWindow(hwnd);
    let command = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_NONOTIFY,
        point.x,
        point.y,
        0,
        hwnd,
        None,
    )
    .0;
    // A tray menu needs a null message after TrackPopupMenu so Explorer does
    // not immediately dismiss it when the owner window is a hidden tool window.
    let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(menu);
    if reaction_options::command(hwnd, command as u32) {
        return;
    }
    match command {
        10 => invoke(hwnd, true),
        11 => toggle_console(hwnd),
        12 => hide(hwnd),
        13 => update(hwnd, Event::TogglePin),
        50 => reaction_options::toggle_event(hwnd, 1),
        30 => preview_effect(hwnd, navi_state::holograms::SIGNAL),
        31 => preview_effect(hwnd, navi_state::holograms::CONTEXT),
        32 => preview_effect(hwnd, navi_state::holograms::STACK),
        33 => preview_effect(hwnd, navi_state::holograms::ALL),
        42 => commit(hwnd, |app| {
            app.effects.layers = if app.effects.layers == 2 { 4 } else { 2 }
        }),
        39 => commit(hwnd, |app| {
            app.surfaces.effect_anchor = match app.surfaces.effect_anchor {
                surface::SurfaceAnchor::WorkArea => surface::SurfaceAnchor::ContextRect(Rect {
                    left: app.work.left + 20,
                    top: app.work.top + 20,
                    right: app.work.left + 20,
                    bottom: app.work.top + 20,
                }),
                _ => surface::SurfaceAnchor::WorkArea,
            }
        }),
        35 => commit(hwnd, |app| {
            app.effects.enabled = if app.effects.enabled == 0 {
                navi_state::holograms::ALL
            } else {
                0
            }
        }),
        36 => commit(hwnd, |app| {
            app.effects.motion = !app.effects.motion;
            // Accessibility changes cancel the old plan rather than re-sampling it.
            app.surfaces.clear_focus_effect();
            app.reactions.clear();
        }),
        37 => commit(hwnd, |app| {
            app.effects.duration_ms = match app.effects.duration_ms {
                800 => 1800,
                1800 => 3000,
                _ => 800,
            }
        }),
        38 => commit(hwnd, |app| {
            app.action_visual.enabled = false;
            app.action_visual.clear();
            app.effects.enabled = 0;
            app.effects.clear();
            app.reactions.set_enabled(false);
            app.reactions.config.events = 0;
            app.surfaces.clear_focus_effect();
            app.cursor = None;
        }),
        20..=25 => update(
            hwnd,
            Event::SetPhase(
                [
                    Phase::Idle,
                    Phase::Invoked,
                    Phase::Listening,
                    Phase::Processing,
                    Phase::Responding,
                    Phase::Error,
                ][command as usize - 20],
            ),
        ),
        99 => {
            let _ = DestroyWindow(hwnd);
        }
        _ => {}
    }
}

unsafe extern "system" fn procedure(hwnd: HWND, message: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let taskbar_created = with_app(|app| app.taskbar_created).unwrap_or(0);
    if message != 0 && message == taskbar_created {
        with_app(|app| {
            app.tray = None;
            app.tray = tray::Tray::new(hwnd).ok();
        });
        return LRESULT(0);
    }
    match message {
        text_backend::MESSAGE => {
            text_backend::complete(hwnd);
            return LRESULT(0);
        }
        desktop_observer::MESSAGE => {
            // Windows can report foreground before a taskbar restore has
            // finished updating visibility, minimized state and geometry.
            let _ = SetTimer(hwnd, 6, 120, None);
            return LRESULT(0);
        }
        WM_TIMER if wp.0 == 6 => {
            let _ = KillTimer(hwnd, 6);
            let _ = PostMessageW(
                hwnd,
                desktop_observer::SETTLED_MESSAGE,
                WPARAM(desktop_observer::FOCUS_EVENT as usize),
                LPARAM(GetForegroundWindow().0),
            );
            return LRESULT(0);
        }
        desktop_observer::SETTLED_MESSAGE => {
            let source = HWND(lp.0);
            if !desktop_observer::is_current_focus_message(
                wp.0 as u32,
                source,
                GetForegroundWindow(),
            ) {
                return LRESULT(0);
            }
            let event = desktop_observer::resolve(source);
            let changed = with_app(|app| {
                if let Some(event) = event {
                    if !event.own
                        && !event.suppressed
                        && app.reactions.config.focus_family == navi_state::visual::Family::StackV2
                    {
                        monitor_catalog::prepare(app);
                    }
                    let now = app.now();
                    focus_reaction::apply(
                        &mut app.reactions,
                        &mut app.surfaces,
                        &mut app.effects,
                        event,
                        now,
                    )
                } else {
                    focus_reaction::cancel(&mut app.reactions, &mut app.surfaces)
                }
            })
            .unwrap_or(false);
            if changed {
                commit(hwnd, |_| {});
            }
            return LRESULT(0);
        }
        WM_INPUT => {
            let changed = with_app(|app| {
                let now = app.now();
                let sample = app.cursor.as_mut().and_then(|c| c.observe(lp, now));
                sample.is_some_and(|sample| pointer_reaction::apply(app, sample, now))
            })
            .unwrap_or(false);
            if changed {
                commit(hwnd, |_| {});
            }
            // Win32 requires the default handler to release foreground raw input.
            return DefWindowProcW(hwnd, message, wp, lp);
        }
        WM_KILLFOCUS => {
            let _ = KillTimer(hwnd, 5);
            return LRESULT(0);
        }
        WM_SETFOCUS => {
            commit(hwnd, |_| {});
            return LRESULT(0);
        }
        WM_ACTIVATE if wp.0 as u16 != WA_INACTIVE as u16 => {
            let changed =
                with_app(|app| focus_reaction::cancel(&mut app.reactions, &mut app.surfaces))
                    .unwrap_or(false);
            if changed {
                commit(hwnd, |_| {});
            }
        }
        tray::CALLBACK => {
            match lp.0 as u32 & 0xffff {
                WM_CONTEXTMENU | WM_RBUTTONUP => menu(hwnd),
                0x400 | 0x401 => invoke(hwnd, true),
                _ => {}
            }
            return LRESULT(0);
        }
        WM_ERASEBKGND => return LRESULT(1),
        WM_MOUSEACTIVATE => {
            let previous = GetForegroundWindow();
            let beacon = with_app(|app| {
                if previous != hwnd {
                    app.previous_focus = previous;
                }
                app.surfaces.kind == SurfaceKind::Beacon
            })
            .unwrap_or(true);
            return LRESULT(if beacon { MA_NOACTIVATE } else { MA_ACTIVATE } as isize);
        }
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut paint);
            let _ = EndPaint(hwnd, &paint);
            commit(hwnd, |_| {});
            return LRESULT(0);
        }
        WM_NCHITTEST => {
            let mut point = POINT {
                x: lp.0 as i16 as i32,
                y: (lp.0 >> 16) as i16 as i32,
            };
            let _ = ScreenToClient(hwnd, &mut point);
            let local =
                LPARAM((point.x as u16 as usize | ((point.y as u16 as usize) << 16)) as isize);
            return LRESULT(match current_hit(hwnd, local) {
                Hit::Drag => HTCAPTION as isize,
                Hit::None => HTTRANSPARENT as isize,
                _ => HTCLIENT as isize,
            });
        }
        WM_LBUTTONUP => {
            match current_hit(hwnd, lp) {
                Hit::Invoke => invoke(hwnd, true),
                Hit::Hide | Hit::Close => {
                    let beacon =
                        with_app(|app| app.surfaces.kind == SurfaceKind::Beacon).unwrap_or(false);
                    let _ = beacon;
                    hide(hwnd);
                }
                Hit::Expand => toggle_console(hwnd),
                Hit::Pin => update(hwnd, Event::TogglePin),
                Hit::Voice => update(hwnd, Event::Select(Mode::Voice)),
                Hit::Text => update(hwnd, Event::Select(Mode::Text)),
                Hit::Demo => cycle_demo(hwnd),
                _ => {}
            }
            return LRESULT(0);
        }
        WM_RBUTTONUP => {
            menu(hwnd);
            return LRESULT(0);
        }
        WM_HOTKEY => {
            match wp.0 {
                1 | 16 => {
                    if !with_app(|a| a.demo_submission).unwrap_or(true) {
                        if with_app(|a| {
                            a.surfaces.kind != SurfaceKind::Beacon && GetForegroundWindow() == hwnd
                        })
                        .unwrap_or(false)
                        {
                            hide(hwnd);
                        } else {
                            keyboard_entry::invoke(hwnd, true);
                        }
                        return LRESULT(0);
                    }
                    let active = with_app(|app| {
                        app.surfaces.visible && app.surfaces.kind != SurfaceKind::Beacon
                    })
                    .unwrap_or(false);
                    if active {
                        hide(hwnd);
                    } else {
                        invoke(hwnd, true);
                    }
                }
                2 => toggle_console(hwnd),
                3 => cycle_demo(hwnd),
                14 | 15 => {}
                10..=13 => preview_effect(
                    hwnd,
                    [
                        navi_state::holograms::SIGNAL,
                        navi_state::holograms::CONTEXT,
                        navi_state::holograms::STACK,
                        navi_state::holograms::ALL,
                    ][wp.0 - 10],
                ),
                _ => {}
            }
            return LRESULT(0);
        }
        WM_TIMER => {
            if wp.0 == 5 {
                keyboard_entry::tick(hwnd);
                return LRESULT(0);
            }
            let enabled =
                with_app(|app| wp.0 >= 1 && wp.0 <= 4 && app.timers[wp.0 - 1]).unwrap_or(false);
            if !enabled {
                return LRESULT(0);
            }
            match wp.0 {
                1 => {
                    if !text_backend::complete(hwnd) {
                        update(hwnd, Event::Tick);
                    }
                }
                2 => commit(hwnd, |app| {
                    if app.surfaces.expire(app.now()) {
                        app.session.apply(Event::SetPhase(Phase::Idle));
                        app.interference = 0;
                        app.interference_until = None;
                        restore_focus(hwnd, app);
                    }
                }),
                3 => commit(hwnd, |_| {}),
                4 => commit(hwnd, |_| {}),
                _ => {}
            }
            return LRESULT(0);
        }
        WM_KEYDOWN => {
            if with_app(|a| !a.demo_submission && a.surfaces.kind == SurfaceKind::Compact)
                .unwrap_or(false)
                && ((0x70..=0x7b).contains(&wp.0)
                    || wp.0 == VK_TAB.0 as usize
                    || (wp.0 == b'K' as usize && GetKeyState(VK_CONTROL.0 as i32) < 0))
            {
                return LRESULT(0);
            }
            if let Some(phase) = navi_demo::phase_for_key(wp.0) {
                update(hwnd, Event::SetPhase(phase));
            } else if wp.0 == VK_ESCAPE.0 as usize {
                hide(hwnd);
            } else if wp.0 == 0x75 {
                cycle_demo(hwnd);
            } else if wp.0 == 0x77 {
                hide(hwnd);
            } else if wp.0 == 0x78 {
                preview_effect(hwnd, navi_state::holograms::SIGNAL);
            } else if wp.0 == 0x79 {
                preview_effect(hwnd, navi_state::holograms::CONTEXT);
            } else if wp.0 == 0x7a {
                preview_effect(hwnd, navi_state::holograms::STACK);
            } else if wp.0 == 0x7b {
                preview_effect(hwnd, navi_state::holograms::ALL);
            } else if wp.0 == b'K' as usize && GetKeyState(VK_CONTROL.0 as i32) < 0 {
                update(hwnd, Event::SetPhase(Phase::Listening));
            } else if wp.0 == b'N' as usize && GetKeyState(VK_CONTROL.0 as i32) < 0 {
                toggle_console(hwnd);
            } else if wp.0 == b'Q' as usize
                && GetKeyState(VK_CONTROL.0 as i32) < 0
                && GetKeyState(VK_SHIFT.0 as i32) < 0
            {
                let _ = DestroyWindow(hwnd);
            } else if wp.0 == VK_TAB.0 as usize {
                let next = with_app(|app| {
                    if app.session.mode == Mode::Voice {
                        Mode::Text
                    } else {
                        Mode::Voice
                    }
                });
                if let Some(next) = next {
                    update(hwnd, Event::Select(next));
                }
            }
            return LRESULT(0);
        }
        WM_CHAR => {
            if with_app(|a| !a.demo_submission && a.surfaces.kind == SurfaceKind::Compact)
                .unwrap_or(false)
            {
                keyboard_entry::character(hwnd, wp);
                return LRESULT(0);
            }
            match wp.0 {
                13 => {
                    let demo = with_app(|a| {
                        a.demo_submission
                            || (a.session.input.is_empty() && a.session.phase == Phase::Listening)
                    })
                    .unwrap_or(false);
                    if demo {
                        update(hwnd, Event::Submit);
                    } else {
                        text_backend::submit(hwnd);
                    }
                }
                8 => update(hwnd, Event::Backspace),
                v if v >= 32 => {
                    let unit = v as u16;
                    let c = with_app(|app| {
                        if (0xD800..=0xDBFF).contains(&unit) {
                            app.surrogate = Some(unit);
                            return None;
                        }
                        if let Some(high) = app.surrogate.take() {
                            char::decode_utf16([high, unit]).next().and_then(|r| r.ok())
                        } else {
                            char::from_u32(unit as u32)
                        }
                    })
                    .flatten();
                    if let Some(c) = c {
                        update(hwnd, Event::Character(c));
                    }
                }
                _ => {}
            }
            return LRESULT(0);
        }
        WM_DPICHANGED => {
            let suggested = *(lp.0 as *const RECT);
            let monitor = MonitorFromRect(&suggested, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(monitor, &mut info).as_bool() {
                commit(hwnd, |app| {
                    app.dpi = (wp.0 as u16 as u32).max(96);
                    app.work = Rect {
                        left: info.rcWork.left,
                        top: info.rcWork.top,
                        right: info.rcWork.right,
                        bottom: info.rcWork.bottom,
                    };
                    app.render_key = None;
                    monitor_catalog::refresh(app);
                });
            }
            return LRESULT(0);
        }
        WM_DISPLAYCHANGE | WM_SETTINGCHANGE => {
            if let Ok(work) = monitor_work(hwnd) {
                commit(hwnd, |app| {
                    app.work = work;
                    monitor_catalog::refresh(app);
                });
            }
            return LRESULT(0);
        }
        WM_EXITSIZEMOVE => {
            let mut bounds = RECT::default();
            if GetWindowRect(hwnd, &mut bounds).is_ok() {
                let work = monitor_work(hwnd).ok();
                commit(hwnd, |app| {
                    if app.surfaces.kind == SurfaceKind::Console {
                        app.surfaces.console_position = Some((bounds.left, bounds.top));
                        if let Some(work) = work {
                            app.work = work;
                        }
                    }
                });
            }
            return LRESULT(0);
        }
        WM_CLOSE => {
            hide(hwnd);
            return LRESULT(0);
        }
        WM_DESTROY => {
            keyboard_entry::cancel(hwnd);
            with_app(|app| {
                app.observer = None;
                app.cursor = None;
            });
            for id in 1..=4 {
                let _ = KillTimer(hwnd, id);
            }
            PostQuitMessage(0);
            return LRESULT(0);
        }
        _ => {}
    }
    DefWindowProcW(hwnd, message, wp, lp)
}

pub fn run(snapshot: Option<PathBuf>) -> Result<()> {
    run_mode(snapshot, None, None, None)
}

pub fn benchmark(path: PathBuf) -> Result<()> {
    run_mode(None, Some(path), None, None)
}
pub fn observer_probe(path: PathBuf) -> Result<()> {
    run_mode(None, None, Some(path), None)
}

pub fn input_check(path: PathBuf) -> Result<()> {
    run_mode(None, None, None, Some(path))
}

fn run_mode(
    snapshot: Option<PathBuf>,
    benchmark: Option<PathBuf>,
    observer_probe: Option<PathBuf>,
    input_check: Option<PathBuf>,
) -> Result<()> {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let result = run_window(snapshot, benchmark, observer_probe, input_check);
        APP.with(|s| {
            s.borrow_mut().take();
        });
        CoUninitialize();
        result
    }
}

unsafe extern "system" fn probe_procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCHITTEST {
        return LRESULT(HTTRANSPARENT as isize);
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

pub fn composition_probe(path: &std::path::Path) -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let result = (|| -> Result<()> {
            let instance = GetModuleHandleW(None)?;
            let class = w!("NaviCompositionProbe");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(probe_procedure),
                hInstance: instance.into(),
                lpszClassName: class,
                ..Default::default()
            };
            if RegisterClassW(&wc) == 0 {
                return Err(Error::from_win32());
            }
            let foreground = GetForegroundWindow();
            let hwnd = CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
                class,
                w!("NAVI / composition probe"),
                WS_POPUP,
                80,
                80,
                240,
                180,
                None,
                None,
                instance,
                None,
            );
            if hwnd.0 == 0 {
                return Err(Error::from_win32());
            }
            let result = (|| -> Result<()> {
                let probe = navi_render::composition::CompositionProbe::new(hwnd, path)?;
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    80,
                    80,
                    240,
                    180,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                )?;
                verification::pump(std::time::Duration::from_millis(1200));
                let report = format!("{{\"hardware\":{},\"alpha_readback\":{:?},\"visible\":{},\"focus_preserved\":{},\"committed\":true}}",probe.hardware,probe.alpha,IsWindowVisible(hwnd).as_bool(),GetForegroundWindow()==foreground);
                std::fs::write(path.with_extension("json"), report)
                    .map_err(|e| Error::new(E_FAIL, e.to_string()))?;
                Ok(())
            })();
            let _ = DestroyWindow(hwnd);
            let _ = UnregisterClassW(class, instance);
            result
        })();
        CoUninitialize();
        result
    }
}

unsafe fn run_window(
    snapshot: Option<PathBuf>,
    benchmark: Option<PathBuf>,
    observer_probe: Option<PathBuf>,
    input_check: Option<PathBuf>,
) -> Result<()> {
    let instance = GetModuleHandleW(None)?;
    let class = w!("NaviSessionWindow");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(procedure),
        hInstance: instance.into(),
        lpszClassName: class,
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        ..Default::default()
    };
    if RegisterClassW(&wc) == 0 {
        return Err(Error::from_win32());
    }
    let mut work = RECT::default();
    SystemParametersInfoW(
        SPI_GETWORKAREA,
        0,
        Some(&mut work as *mut _ as _),
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
    )?;
    let dpi = GetDpiForSystem().max(96);
    let started = Instant::now();
    APP.with(|s| {
        *s.borrow_mut() = Some(App {
            action_visual: Default::default(),
            action_surface: None,
            text: Default::default(),
            demo_submission: snapshot.is_some() || benchmark.is_some(),
            observer: None,
            cursor: None,
            passive_observers: snapshot.is_none() && benchmark.is_none() && input_check.is_none(),
            reactions: Default::default(),
            effects: Default::default(),
            overlay: None,
            tray: None,
            taskbar_created: RegisterWindowMessageW(w!("TaskbarCreated")),
            session: Session::default(),
            surfaces: SurfaceManager::default(),
            renderer: None,
            render_key: None,
            work: Rect {
                left: work.left,
                top: work.top,
                right: work.right,
                bottom: work.bottom,
            },
            dpi,
            started,
            previous_focus: HWND::default(),
            interference: 0,
            interference_until: None,
            timers: [false; 4],
            hotkeys: [false; 3],
            effect_hotkeys: [false; 4],
            failure: None,
            surrogate: None,
            context_entries: Vec::new(),
        })
    });
    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TOOLWINDOW,
        class,
        w!("NAVI"),
        WS_POPUP | WS_SYSMENU,
        work.right - 150,
        work.bottom - 80,
        110,
        35,
        None,
        None,
        instance,
        None,
    );
    if hwnd.0 == 0 {
        return Err(Error::from_win32());
    }
    if snapshot.is_none() && benchmark.is_none() && input_check.is_none() {
        with_app(|app| {
            app.surfaces.monitors = monitor_catalog::collect();
            app.surfaces.session_seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |d| d.as_nanos() as u64);
        });
    }
    with_app(|app| app.tray = tray::Tray::new(hwnd).ok());
    if snapshot.is_none() && benchmark.is_none() && input_check.is_none() {
        with_app(|app| match desktop_observer::DesktopObserver::new(hwnd) {
            Ok(observer) => app.observer = Some(observer),
            Err(error) => {
                app.reactions.config.events &= !1;
                let _ = std::fs::write(
                    std::env::temp_dir().join("navi-observer-error.log"),
                    error.to_string(),
                );
            }
        });
    }
    if snapshot.is_none() && benchmark.is_none() && input_check.is_none() {
        with_app(|app| match cursor_observer::CursorObserver::new(hwnd) {
            Ok(cursor) => app.cursor = Some(cursor),
            Err(error) => {
                app.reactions.config.events &= !6;
                let _ = std::fs::write(
                    std::env::temp_dir().join("navi-cursor-error.log"),
                    error.to_string(),
                );
            }
        });
    }
    let primary_key = RegisterHotKey(
        hwnd,
        1,
        MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
        VK_SPACE.0 as u32,
    )
    .is_ok();
    let legacy_key = RegisterHotKey(hwnd, 16, MOD_ALT | MOD_NOREPEAT, VK_SPACE.0 as u32).is_ok();
    let invoke_key = primary_key || legacy_key;
    let console_key =
        RegisterHotKey(hwnd, 2, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, b'N' as u32).is_ok();
    let demo_key =
        RegisterHotKey(hwnd, 3, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, b'D' as u32).is_ok();
    with_app(|app| app.hotkeys = [invoke_key, console_key, demo_key]);
    let mut effect_keys = [false; 4];
    for (i, registered) in effect_keys
        .iter_mut()
        .enumerate()
        .filter(|_| snapshot.is_some() || benchmark.is_some())
    {
        *registered = RegisterHotKey(
            hwnd,
            10 + i as i32,
            MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
            b'1' as u32 + i as u32,
        )
        .is_ok();
    }
    with_app(|app| app.effect_hotkeys = effect_keys);
    let initial_foreground = GetForegroundWindow();
    commit(hwnd, |_| {});
    if !invoke_key {
        let _ = std::fs::write(std::env::temp_dir().join("navi-hotkeys.log"), "Alt+Space y Ctrl+Alt+Space no disponibles. Invocar con clic en el indicador o Ctrl+Alt+N si está disponible.");
    }
    let verification = if let Some(path) = input_check {
        let result = verification::input_only(hwnd, &path);
        let _ = DestroyWindow(hwnd);
        result
    } else if let Some(path) = snapshot {
        let result = verification::verify(hwnd, &path, initial_foreground, started);
        let _ = DestroyWindow(hwnd);
        result
    } else if let Some(path) = benchmark {
        let result = verification::benchmark(hwnd, &path);
        let _ = DestroyWindow(hwnd);
        result
    } else if let Some(path) = observer_probe {
        let result = verification::observer_probe(hwnd, &path);
        let _ = DestroyWindow(hwnd);
        result
    } else {
        Ok(())
    };
    // The diagnostic runner closes its own window. Normal mode blocks without polling.
    let mut message = MSG::default();
    loop {
        let status = GetMessageW(&mut message, None, 0, 0).0;
        if status == -1 {
            return Err(Error::from_win32());
        }
        if status == 0 {
            break;
        }
        let _ = TranslateMessage(&message);
        DispatchMessageW(&message);
    }
    for (id, registered) in [invoke_key, console_key, demo_key].into_iter().enumerate() {
        if registered {
            let _ = UnregisterHotKey(hwnd, id as i32 + 1);
        }
    }
    for (i, registered) in effect_keys.into_iter().enumerate() {
        if registered {
            let _ = UnregisterHotKey(hwnd, 10 + i as i32);
        }
    }
    let _ = UnregisterClassW(class, instance);
    verification?;
    if let Some(failure) = with_app(|app| app.failure.take()).flatten() {
        return Err(Error::new(E_FAIL, failure));
    }
    Ok(())
}
