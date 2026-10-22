//! Diagnostics target our own HWND only. No external input injection.
use super::*;
use std::{io::Write, path::Path, time::Duration};

pub(super) unsafe fn observer_probe(hwnd: HWND, path: &Path) -> Result<()> {
    pump(Duration::from_secs(12));
    let report = with_app(|a| {
        format!(
            "{{\"observer_installed\":{},\"events_received\":{},\"reactions_emitted\":{}}}",
            a.observer.is_some(),
            a.reactions.received,
            a.reactions.emitted
        )
    })
    .unwrap_or_default();
    let _ = hwnd;
    std::fs::write(path, report).map_err(|e| Error::new(E_FAIL, e.to_string()))
}

/// Synthetic focus events use the production coordinator and a real overlay.
/// This checks plumbing/resources, not physical Alt+Tab or another application's UI.
unsafe fn verify_focus_integration(hwnd: HWND, checks: &mut Vec<(&str, bool)>) -> Result<()> {
    use navi_state::reactions::{Bounds, DesktopEvent, DesktopKind, Point, ReactionEngine};
    checks.push((
        "focus_observer_registration_smoke",
        desktop_observer::DesktopObserver::new(hwnd).is_ok(),
    ));
    commit(hwnd, |app| {
        app.effects.clear();
        app.effects.enabled = 0;
        app.reactions = ReactionEngine::default();
        app.session.apply(Event::Reset);
        app.surfaces.idle();
    });
    let foreground = GetForegroundWindow();
    let event = with_app(|app| DesktopEvent {
        kind: DesktopKind::WindowFocus,
        source: 1,
        bounds: Bounds {
            left: app.work.left,
            top: app.work.top,
            right: app.work.right,
            bottom: app.work.bottom,
        },
        work_area: Bounds {
            left: app.work.left,
            top: app.work.top,
            right: app.work.right,
            bottom: app.work.bottom,
        },
        point: Point::default(),
        own: false,
        suppressed: false,
        dpi: app.dpi,
    })
    .ok_or_else(|| Error::new(E_FAIL, "Missing focus verification app"))?;
    commit(hwnd, |app| {
        let now = app.now();
        focus_reaction::apply(
            &mut app.reactions,
            &mut app.surfaces,
            &mut app.effects,
            event,
            now,
        );
    });
    pump(Duration::from_millis(220));
    let (drawn, passthrough) = with_app(|app| -> Result<(bool, bool)> {
        let overlay = app
            .overlay
            .as_ref()
            .ok_or_else(|| Error::new(E_FAIL, "Missing focus overlay"))?;
        let plan = app
            .surfaces
            .focus_effect_plan(app.now())
            .ok_or_else(|| Error::new(E_FAIL, "Missing focus plan"))?;
        let mut actual = RECT::default();
        GetWindowRect(overlay.hwnd, &mut actual)?;
        let (_, partial, _) = overlay.renderer.alpha_stats();
        let style = GetWindowLongPtrW(overlay.hwnd, GWL_EXSTYLE) as u32;
        // This point lies on a scaled LayerStack edge, not just in its empty margin.
        let point = POINT {
            x: actual.left + (actual.right - actual.left) * 48 / 540,
            y: actual.top + (actual.bottom - actual.top) * 90 / 300,
        };
        Ok((
            partial > 0
                && actual.left == plan.bounds.left
                && actual.top == plan.bounds.top
                && actual.right == plan.bounds.right
                && actual.bottom == plan.bounds.bottom,
            style & (WS_EX_TRANSPARENT.0 | WS_EX_NOACTIVATE.0)
                == (WS_EX_TRANSPARENT.0 | WS_EX_NOACTIVATE.0)
                && WindowFromPoint(point) != overlay.hwnd,
        ))
    })
    .ok_or_else(|| Error::new(E_FAIL, "Missing focus verification app"))??;
    checks.push(("synthetic_focus_draws_at_surface_manager_bounds", drawn));
    checks.push((
        "synthetic_focus_preserves_foreground_and_click_through",
        passthrough && GetForegroundWindow() == foreground,
    ));
    let duplicate = with_app(|app| {
        let now = app.now();
        let changed = focus_reaction::apply(
            &mut app.reactions,
            &mut app.surfaces,
            &mut app.effects,
            event,
            now,
        );
        !changed && app.reactions.emitted == 1
    })
    .unwrap_or(false);
    checks.push(("synthetic_focus_duplicate_does_not_restart", duplicate));
    commit(hwnd, |app| {
        let now = app.now();
        focus_reaction::apply(
            &mut app.reactions,
            &mut app.surfaces,
            &mut app.effects,
            DesktopEvent { source: 2, ..event },
            now,
        );
    });
    checks.push((
        "synthetic_focus_cooldown_cancels_obsolete_overlay",
        with_app(|app| app.overlay.is_none() && !app.timers[3]).unwrap_or(false),
    ));
    pump(Duration::from_millis(750));
    commit(hwnd, |app| {
        let now = app.now();
        focus_reaction::apply(
            &mut app.reactions,
            &mut app.surfaces,
            &mut app.effects,
            DesktopEvent { source: 3, ..event },
            now,
        );
    });
    checks.push((
        "synthetic_focus_resumes_after_cooldown",
        with_app(|app| app.overlay.is_some()).unwrap_or(false),
    ));
    commit(hwnd, |app| {
        let now = app.now();
        focus_reaction::apply(
            &mut app.reactions,
            &mut app.surfaces,
            &mut app.effects,
            DesktopEvent {
                source: 4,
                suppressed: true,
                ..event
            },
            now,
        );
    });
    checks.push((
        "synthetic_focus_suppression_removes_visual",
        with_app(|app| app.overlay.is_none() && !app.timers[3]).unwrap_or(false),
    ));
    commit(hwnd, |app| {
        app.reactions.set_enabled(true);
        let now = app.now();
        focus_reaction::apply(
            &mut app.reactions,
            &mut app.surfaces,
            &mut app.effects,
            DesktopEvent { source: 5, ..event },
            now,
        );
    });
    pump(Duration::from_millis(1050));
    checks.push((
        "synthetic_focus_expires_to_quiet_beacon",
        with_app(|app| {
            app.overlay.is_none()
                && app.surfaces.focus_effect_plan(app.now()).is_none()
                && app.reactions.active.is_empty()
                && app.timers == [false; 4]
                && app.surfaces.kind == SurfaceKind::Beacon
                && app.surfaces.visible
        })
        .unwrap_or(false),
    ));
    Ok(())
}

/// Bounded diagnostic workload: repeated bursts for sampling, never used in normal mode.
pub(super) unsafe fn benchmark(hwnd: HWND, root: &Path) -> Result<()> {
    use navi_state::holograms::{ALL, CONTEXT, SIGNAL, STACK};
    let io_error = |e: std::io::Error| Error::new(E_FAIL, e.to_string());
    std::fs::create_dir_all(root).map_err(io_error)?;
    let foreground = GetForegroundWindow();
    let mut results = Vec::new();
    for (name, mask) in [
        ("idle", 0),
        ("signal", SIGNAL),
        ("context", CONTEXT),
        ("stack", STACK),
        ("combined", ALL),
        ("fractured-halo", 8),
        ("click-pulse", 16),
        ("drag-connector", 32),
    ] {
        commit(hwnd, |app| {
            app.effects.clear();
            app.surfaces.clear_focus_effect();
            app.reactions = Default::default();
            app.effects.enabled = 0;
            app.session.apply(Event::Reset);
            app.surfaces.idle();
        });
        std::fs::write(
            root.join("stage.json"),
            format!("{{\"stage\":\"{name}\",\"mask\":{mask}}}"),
        )
        .map_err(io_error)?;
        let start = Instant::now();
        let mut captured = false;
        let mut background_unchanged = true;
        let mut foreground_class = String::new();
        while start.elapsed() < Duration::from_secs(9) {
            let elapsed = start.elapsed().as_millis() as u64;
            if mask == 32
                || (mask == 16
                    && with_app(|a| a.surfaces.focus_effect_plan(a.now()).is_none())
                        .unwrap_or(false))
            {
                commit(hwnd, |a| {
                    use navi_state::reactions::{Bounds, DesktopEvent, DesktopKind, Point};
                    let x = a.work.left + (a.work.right - a.work.left) / 2;
                    let y = a.work.top + (a.work.bottom - a.work.top) / 2;
                    let kind = if mask == 16 {
                        DesktopKind::MouseClick
                    } else if a.reactions.active.is_empty() {
                        DesktopKind::MouseDragStart
                    } else {
                        DesktopKind::MouseDragMove
                    };
                    let event = DesktopEvent {
                        kind,
                        source: 100,
                        bounds: Bounds {
                            left: x - 100,
                            top: y - 60,
                            right: x,
                            bottom: y,
                        },
                        work_area: Bounds {
                            left: a.work.left,
                            top: a.work.top,
                            right: a.work.right,
                            bottom: a.work.bottom,
                        },
                        point: Point {
                            x: x + (elapsed as f32 / 250.0).sin().mul_add(120.0, 0.0) as i32,
                            y,
                        },
                        dpi: a.dpi,
                        own: false,
                        suppressed: false,
                    };
                    let now = a.now();
                    focus_reaction::apply(
                        &mut a.reactions,
                        &mut a.surfaces,
                        &mut a.effects,
                        event,
                        now,
                    );
                });
            } else if mask != 0
                && with_app(|a| {
                    a.effects.frame(a.now()).is_none()
                        && a.surfaces.focus_effect_plan(a.now()).is_none()
                })
                .unwrap_or(false)
            {
                if mask == 8 {
                    reaction_options::preview(hwnd, navi_state::visual::Family::FracturedHalo);
                } else {
                    preview_effect(hwnd, mask);
                }
            }
            pump(Duration::from_millis(if mask == 32 { 33 } else { 100 }));
            if mask != 0 && !captured && start.elapsed() > Duration::from_millis(450) {
                let result = with_app(|app| -> Result<bool> {
                    let overlay = app
                        .overlay
                        .as_ref()
                        .ok_or_else(|| Error::new(E_FAIL, "Missing benchmark overlay"))?;
                    let mut rect = RECT::default();
                    GetWindowRect(overlay.hwnd, &mut rect)?;
                    overlay
                        .renderer
                        .snapshot(&root.join(format!("{name}-surface.bmp")))
                        .map_err(io_error)?;
                    Renderer::capture_composed_for_qa(
                        rect,
                        &root.join(format!("{name}-composed.bmp")),
                    )?;
                    let screen = GetDC(None);
                    let before = GetPixel(screen, rect.left + 2, rect.top + 2);
                    SetWindowPos(
                        overlay.hwnd,
                        None,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_HIDEWINDOW,
                    )?;
                    // Read the compositor again after hiding; corner alpha must leave it unchanged.
                    std::thread::sleep(Duration::from_millis(40));
                    let after = GetPixel(screen, rect.left + 2, rect.top + 2);
                    ReleaseDC(None, screen);
                    let under = WindowFromPoint(POINT {
                        x: rect.left + 2,
                        y: rect.top + 2,
                    });
                    let mut class = [0u16; 128];
                    let length = GetClassNameW(under, &mut class);
                    foreground_class = String::from_utf16_lossy(&class[..length.max(0) as usize]);
                    SetWindowPos(
                        overlay.hwnd,
                        None,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    )?;
                    Ok(before == after)
                })
                .ok_or_else(|| Error::new(E_FAIL, "Missing benchmark app"))??;
                background_unchanged = result;
                captured = true;
            }
        }
        commit(hwnd, |app| {
            app.effects.clear();
            app.surfaces.clear_focus_effect();
            app.reactions.clear();
        });
        let released = with_app(|a| a.overlay.is_none() && a.timers == [false; 4]).unwrap_or(false);
        results.push(format!("{{\"stage\":\"{name}\",\"released\":{released},\"transparent_corner_preserved\":{background_unchanged},\"underlying_window_class\":\"{foreground_class}\",\"focus_preserved\":{}}}",GetForegroundWindow()==foreground));
    }
    std::fs::write(root.join("stage.json"), "{\"stage\":\"done\"}").map_err(io_error)?;
    std::fs::write(root.join("native.json"), format!("[{}]", results.join(",")))
        .map_err(io_error)?;
    Ok(())
}

unsafe fn snapshot(path: &Path) -> Result<()> {
    with_app(|app| app.renderer.as_ref().unwrap().snapshot(path))
        .ok_or_else(|| Error::new(E_FAIL, "Missing application"))?
        .map_err(|e| Error::new(E_FAIL, e.to_string()))
}

pub(super) unsafe fn pump(duration: Duration) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                PostQuitMessage(msg.wParam.0 as i32);
                return;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub(super) unsafe fn verify(
    hwnd: HWND,
    path: &Path,
    initial_foreground: HWND,
    started: Instant,
) -> Result<()> {
    let first_surface_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut checks: Vec<(&str, bool)> = Vec::new();
    checks.push(("startup_visible", IsWindowVisible(hwnd).as_bool()));
    checks.push((
        "startup_preserves_focus",
        GetForegroundWindow() == initial_foreground,
    ));
    checks.push((
        "idle_has_no_timers",
        with_app(|a| a.timers == [false; 4]).unwrap_or(false),
    ));
    snapshot(&path.with_file_name("compact-idle.bmp"))?;
    SendMessageW(hwnd, WM_HOTKEY, WPARAM(1), LPARAM(0));
    pump(Duration::from_millis(350));
    snapshot(path)?;
    let mut rect = RECT::default();
    GetWindowRect(hwnd, &mut rect)?;
    checks.push((
        "compact_work_area_anchor",
        with_app(|a| {
            rect.right <= a.work.right
                && rect.bottom <= a.work.bottom
                && rect.left >= a.work.left
                && rect.top >= a.work.top
        })
        .unwrap_or(false),
    ));
    checks.push((
        "compact_320_by_104_dip",
        with_app(|a| {
            let plan = a.surfaces.plan(a.work, a.dpi, &a.session);
            plan.size.width == 320.0 && plan.size.height == 104.0
        })
        .unwrap_or(false),
    ));
    for c in "Hola, NAVI. ¿Estás?".encode_utf16() {
        SendMessageW(hwnd, WM_CHAR, WPARAM(c as usize), LPARAM(0));
    }
    checks.push((
        "unicode_input",
        with_app(|a| a.session.input == "Hola, NAVI. ¿Estás?").unwrap_or(false),
    ));
    for _ in 0..110 {
        SendMessageW(hwnd, WM_CHAR, WPARAM(b'x' as usize), LPARAM(0));
    }
    checks.push((
        "content_grows_within_work_area",
        with_app(|a| {
            let plan = a.surfaces.plan(a.work, a.dpi, &a.session);
            plan.size.height > 104.0 && plan.bounds.bottom <= a.work.bottom
        })
        .unwrap_or(false),
    ));
    snapshot(&path.with_file_name("compact-long-input.bmp"))?;
    SendMessageW(hwnd, WM_CHAR, WPARAM(13), LPARAM(0));
    checks.push((
        "processing",
        with_app(|a| a.session.phase == Phase::Processing).unwrap_or(false),
    ));
    snapshot(&path.with_file_name("compact-processing.bmp"))?;
    // Directly advance our own simulation, then verify the real response timer.
    for _ in 0..24 {
        SendMessageW(hwnd, WM_TIMER, WPARAM(1), LPARAM(0));
    }
    checks.push((
        "responding",
        with_app(|a| {
            a.session.phase == Phase::Responding && a.timers == [false, true, false, false]
        })
        .unwrap_or(false),
    ));
    snapshot(&path.with_file_name("compact-response.bmp"))?;
    pump(Duration::from_millis(4200));
    checks.push((
        "response_auto_dismissed_to_idle",
        with_app(|a| {
            a.session.phase == Phase::Idle
                && a.surfaces.kind == SurfaceKind::Beacon
                && a.timers == [false; 4]
        })
        .unwrap_or(false),
    ));
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(0x77), LPARAM(0));
    checks.push((
        "collapse_keeps_minimum_presence",
        IsWindowVisible(hwnd).as_bool(),
    ));
    // Passive demo notifications must not change the current foreground window.
    let foreground = GetForegroundWindow();
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(0x73), LPARAM(0));
    checks.push((
        "response_does_not_activate",
        GetForegroundWindow() == foreground,
    ));
    pump(Duration::from_millis(4200));
    checks.push((
        "response_always_returns_to_beacon",
        IsWindowVisible(hwnd).as_bool()
            && with_app(|a| a.surfaces.kind == SurfaceKind::Beacon && a.timers == [false; 4])
                .unwrap_or(false),
    ));
    SendMessageW(hwnd, WM_HOTKEY, WPARAM(1), LPARAM(0));
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(0x71), LPARAM(0));
    snapshot(&path.with_file_name("compact-listening.bmp"))?;
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(0x74), LPARAM(0));
    pump(Duration::from_millis(350));
    snapshot(&path.with_file_name("compact-error.bmp"))?;
    checks.push((
        "error_is_quiet_after_interference",
        with_app(|a| a.session.phase == Phase::Error && a.timers == [false; 4]).unwrap_or(false),
    ));
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(0x70), LPARAM(0));
    checks.push((
        "error_recovered",
        with_app(|a| a.session.phase == Phase::Invoked).unwrap_or(false),
    ));
    SendMessageW(hwnd, WM_HOTKEY, WPARAM(2), LPARAM(0));
    checks.push((
        "original_console_preserved",
        with_app(|a| a.surfaces.kind == SurfaceKind::Console).unwrap_or(false),
    ));
    snapshot(&path.with_file_name("console.bmp"))?;
    SendMessageW(hwnd, WM_HOTKEY, WPARAM(2), LPARAM(0));
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(VK_ESCAPE.0 as usize), LPARAM(0));
    checks.push((
        "escape_collapses_without_closing",
        IsWindow(hwnd).as_bool()
            && IsWindowVisible(hwnd).as_bool()
            && with_app(|a| a.surfaces.kind == SurfaceKind::Beacon).unwrap_or(false),
    ));
    SendMessageW(hwnd, WM_TIMER, WPARAM(1), LPARAM(0));
    checks.push((
        "stale_timer_does_not_resurrect_surface",
        with_app(|a| a.surfaces.kind == SurfaceKind::Beacon).unwrap_or(false),
    ));
    SendMessageW(hwnd, WM_HOTKEY, WPARAM(1), LPARAM(0));
    checks.push((
        "keyboard_reinvokes_hidden_surface",
        IsWindowVisible(hwnd).as_bool(),
    ));
    SendMessageW(hwnd, WM_CHAR, WPARAM(b'z' as usize), LPARAM(0));
    SendMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
    SendMessageW(hwnd, WM_HOTKEY, WPARAM(1), LPARAM(0));
    checks.push((
        "draft_survives_collapse_and_reinvoke",
        with_app(|a| a.session.input == "z").unwrap_or(false),
    ));
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(0x73), LPARAM(0));
    checks.push((
        "draft_prevents_automatic_dismissal",
        with_app(|a| a.surfaces.remaining(a.now()).is_none()).unwrap_or(false),
    ));
    checks.push((
        "tray_installed",
        with_app(|a| a.tray.is_some()).unwrap_or(false),
    ));
    let foreground = GetForegroundWindow();
    let mut signal_alpha = true;
    let mut signal_passthrough = true;
    for (phase, name) in [
        (Phase::Listening, "listening"),
        (Phase::Processing, "processing"),
        (Phase::Responding, "response"),
        (Phase::Error, "error"),
    ] {
        commit(hwnd, |a| {
            a.effects
                .preview(navi_state::holograms::SIGNAL, phase, a.now())
        });
        pump(Duration::from_millis(350));
        let result = with_app(|a| -> Result<(bool, bool)> {
            let o = a
                .overlay
                .as_ref()
                .ok_or_else(|| Error::new(E_FAIL, "Missing SignalField"))?;
            o.renderer
                .snapshot(&path.with_file_name(format!("signal-{name}.bmp")))
                .map_err(|e| Error::new(E_FAIL, e.to_string()))?;
            let alpha = o.renderer.alpha_stats();
            let total = (o.renderer.width * o.renderer.height) as usize;
            let style = GetWindowLongPtrW(o.hwnd, GWL_EXSTYLE) as u32;
            let mut rect = RECT::default();
            GetWindowRect(o.hwnd, &mut rect)?;
            let point = POINT {
                x: (rect.left + rect.right) / 2,
                y: (rect.top + rect.bottom) / 2,
            };
            Ok((
                alpha.0 > total * 8 / 10 && alpha.1 > 0 && alpha.2 == 0,
                style & (WS_EX_TRANSPARENT.0 | WS_EX_NOACTIVATE.0)
                    == WS_EX_TRANSPARENT.0 | WS_EX_NOACTIVATE.0
                    && WindowFromPoint(point) != o.hwnd,
            ))
        })
        .ok_or_else(|| Error::new(E_FAIL, "Missing application"))??;
        signal_alpha &= result.0;
        signal_passthrough &= result.1;
    }
    checks.push((
        "signal_has_transparent_background_in_all_states",
        signal_alpha,
    ));
    checks.push((
        "signal_does_not_take_focus_or_intercept_point",
        signal_passthrough && GetForegroundWindow() == foreground,
    ));
    pump(Duration::from_millis(1900));
    checks.push((
        "signal_releases_window_renderer_and_timer",
        with_app(|a| a.overlay.is_none() && !a.timers[3]).unwrap_or(false),
    ));
    commit(hwnd, |a| {
        a.effects
            .preview(navi_state::holograms::CONTEXT, Phase::Processing, a.now())
    });
    pump(Duration::from_millis(350));
    let context = with_app(|a| -> Result<bool> {
        let o = a
            .overlay
            .as_ref()
            .ok_or_else(|| Error::new(E_FAIL, "Missing ContextFragment"))?;
        o.renderer
            .snapshot(&path.with_file_name("context-fragment.bmp"))
            .map_err(|e| Error::new(E_FAIL, e.to_string()))?;
        let (zero, partial, _) = o.renderer.alpha_stats();
        Ok(zero > (o.renderer.width * o.renderer.height) as usize * 8 / 10 && partial > 0)
    })
    .ok_or_else(|| Error::new(E_FAIL, "Missing app"))??;
    checks.push(("context_fragment_is_independent_and_transparent", context));
    pump(Duration::from_millis(1900));
    checks.push((
        "context_fragment_releases_resources",
        with_app(|a| a.overlay.is_none() && !a.timers[3]).unwrap_or(false),
    ));
    let mut dpi_reports = Vec::new();
    for (mask, name) in [
        (navi_state::holograms::STACK, "layer-stack"),
        (navi_state::holograms::ALL, "combined"),
    ] {
        commit(hwnd, |a| {
            a.effects.preview(mask, Phase::Processing, a.now())
        });
        pump(Duration::from_millis(350));
        let valid = with_app(|a| -> Result<bool> {
            let o = a
                .overlay
                .as_ref()
                .ok_or_else(|| Error::new(E_FAIL, "Missing layers"))?;
            o.renderer
                .snapshot(&path.with_file_name(format!("{name}.bmp")))
                .map_err(|e| Error::new(E_FAIL, e.to_string()))?;
            let (zero, partial, _) = o.renderer.alpha_stats();
            Ok(zero > (o.renderer.width * o.renderer.height) as usize / 2 && partial > 0)
        })
        .ok_or_else(|| Error::new(E_FAIL, "Missing app"))??;
        checks.push((
            if mask == navi_state::holograms::STACK {
                "stack_is_independent_and_transparent"
            } else {
                "all_families_share_one_overlay"
            },
            valid,
        ));
        pump(Duration::from_millis(1900));
        checks.push((
            if mask == navi_state::holograms::STACK {
                "stack_exits_and_releases_resources"
            } else {
                "combined_exits_and_releases_resources"
            },
            with_app(|a| a.overlay.is_none() && !a.timers[3]).unwrap_or(false),
        ));
    }
    commit(hwnd, |a| {
        a.effects.clear();
        a.effects.enabled = 0;
        a.session.apply(Event::Reset);
        a.surfaces.idle();
    });
    SendMessageW(hwnd, WM_HOTKEY, WPARAM(1), LPARAM(0));
    checks.push((
        "functional_with_all_effects_disabled",
        with_app(|a| a.overlay.is_none() && a.surfaces.kind == SurfaceKind::Compact)
            .unwrap_or(false),
    ));
    verify_focus_integration(hwnd, &mut checks)?;
    checks.extend(manifestation_qa::verify(hwnd, path)?);
    checks.extend(cursor_qa::verify(hwnd)?);
    for dpi in [96, 120, 144, 192] {
        let session = Session {
            phase: Phase::Invoked,
            ..Default::default()
        };
        let size = navi_ui::surface_size(SurfaceKind::Compact, &session);
        let renderer = Renderer::new(dpi, size)?;
        renderer.draw(hwnd, &session, SurfaceKind::Compact, 0)?;
        renderer
            .snapshot(&path.with_file_name(format!("compact-{dpi}dpi.bmp")))
            .map_err(|e| Error::new(E_FAIL, e.to_string()))?;
        dpi_reports.push(format!(
            "{{\"dpi\":{dpi},\"width\":{},\"height\":{}}}",
            renderer.width, renderer.height
        ));
    }
    let hotkeys = with_app(|a| a.hotkeys).unwrap_or([false; 3]);
    let all_passed = checks.iter().all(|(_, pass)| *pass);
    let mut file = std::fs::File::create(path.with_extension("json"))
        .map_err(|e| Error::new(E_FAIL, e.to_string()))?;
    write!(file,"{{\n  \"first_surface_ms\":{first_surface_ms:.2},\n  \"hotkeys\":{hotkeys:?},\n  \"checks\":{{\n{}\n  }},\n  \"dpi_surfaces\":[{}],\n  \"all_passed\":{all_passed}\n}}\n",checks.iter().map(|(name,value)|format!("    \"{name}\":{value}")).collect::<Vec<_>>().join(",\n"),dpi_reports.join(",")).map_err(|e| Error::new(E_FAIL,e.to_string()))?;
    if !all_passed {
        return Err(Error::new(
            E_FAIL,
            "Native compact verification failed; inspect JSON",
        ));
    }
    Ok(())
}
