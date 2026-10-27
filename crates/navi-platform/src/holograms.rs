use super::*;
use navi_state::holograms::EffectFrame;

pub(super) struct Overlay {
    pub hwnd: HWND,
    pub renderer: Renderer,
    dpi: u32,
    size: SurfaceSize,
}

unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => return LRESULT(HTTRANSPARENT as isize),
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_ERASEBKGND => return LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut ps);
            let _ = EndPaint(hwnd, &ps);
            return LRESULT(0);
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

impl Overlay {
    pub unsafe fn new(dpi: u32, size: SurfaceSize) -> Result<Self> {
        let instance = GetModuleHandleW(None)?;
        let class = w!("NaviHolograms");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: class,
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            class,
            w!("NAVI / holograms"),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            instance,
            None,
        );
        if hwnd.0 == 0 {
            return Err(Error::from_win32());
        }
        let renderer = match Renderer::new(dpi, size) {
            Ok(r) => r,
            Err(e) => {
                let _ = DestroyWindow(hwnd);
                return Err(e);
            }
        };
        Ok(Self {
            hwnd,
            renderer,
            dpi,
            size,
        })
    }
    pub unsafe fn manifestation(
        &self,
        frame: EffectFrame,
        bounds: Rect,
        family: navi_state::visual::Family,
        style: navi_state::visual::VisualStyle,
        points: [(f32, f32); 2],
    ) -> Result<()> {
        self.manifestation_with_transform(frame, bounds, family, style, points, None)
    }

    unsafe fn manifestation_with_transform(
        &self,
        frame: EffectFrame,
        bounds: Rect,
        family: navi_state::visual::Family,
        style: navi_state::visual::VisualStyle,
        points: [(f32, f32); 2],
        transform: Option<(f32, f32)>,
    ) -> Result<()> {
        use navi_state::visual::Family;
        let scale = if matches!(family, Family::ClickPulse | Family::DragConnector) {
            1.0
        } else {
            self.size.width / 540.0
        };
        SetWindowPos(
            self.hwnd,
            HWND_TOPMOST,
            bounds.left,
            bounds.top,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
            SWP_NOACTIVATE,
        )?;
        if let Some(transform) = transform {
            self.renderer
                .draw_manifestation_varied(self.hwnd, frame, family, style, transform, points)?;
        } else {
            self.renderer
                .draw_manifestation(self.hwnd, frame, family, style, scale, points)?;
        }
        if !IsWindowVisible(self.hwnd).as_bool() {
            SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
        }
        Ok(())
    }
    pub unsafe fn draw_scaled(
        &self,
        frame: EffectFrame,
        bounds: Rect,
        scale: f32,
        context: &[(String, String)],
    ) -> Result<()> {
        SetWindowPos(
            self.hwnd,
            HWND_TOPMOST,
            bounds.left,
            bounds.top,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
            SWP_NOACTIVATE,
        )?;
        self.renderer
            .draw_holograms_scaled(self.hwnd, frame, scale, context)?;
        if !IsWindowVisible(self.hwnd).as_bool() {
            SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
        }
        Ok(())
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

pub(super) unsafe fn sync(hwnd: HWND, app: &mut App) {
    if startup::sync(hwnd, app) {
        return;
    }
    let now = app.now();
    app.reactions.prune(now);
    app.surfaces.expire_focus_effect(now);
    if !app.demo_submission
        && app
            .action_surface
            .is_none_or(|plan| action_surface::focused().is_none_or(|(work, _)| work != plan.work))
    {
        app.action_visual.clear();
    }
    let action = app.action_visual.frame(now, app.effects.motion);
    if action.is_none() {
        app.action_surface = None;
    }
    let focus = if action.is_none() {
        app.surfaces.focus_effect_plan(now)
    } else {
        None
    };
    let frame = if let Some((frame, _)) = action {
        Some(frame)
    } else if let Some(plan) = focus {
        let progress = plan
            .variation
            .map_or(plan.progress, |v| v.progress(plan.progress));
        let motion = if plan.variation.is_some() {
            plan.motion
        } else {
            app.effects.motion
        };
        Some(EffectFrame {
            layers: app.effects.layers.clamp(2, 4),
            mask: navi_state::holograms::STACK,
            phase: Phase::Processing,
            progress,
            opacity: (if !motion {
                1.0
            } else if let Some(v) = plan.variation {
                v.temporal.envelope(progress)
            } else {
                (progress * 8.0).min(1.0) * ((1.0 - progress) * 6.0).min(1.0)
            }) * plan.variation.map_or(1.0, |v| v.opacity_gain),
            motion,
        })
    } else {
        app.effects.frame(now)
    };
    let Some(frame) = frame else {
        app.overlay = None;
        return;
    };
    let result = (|| -> Result<()> {
        let result_plan = app.action_surface.unwrap_or_else(|| {
            app.surfaces
                .result_surface(app.work, app.dpi, app.action_visual.seed)
        });
        let size = if action.is_some() {
            result_plan.size
        } else {
            focus.map_or_else(|| navi_ui::effect_size(frame.mask), |plan| plan.size)
        };
        let dpi = if action.is_some() {
            result_plan.dpi
        } else {
            focus.map_or(app.dpi, |plan| plan.dpi)
        };
        if app
            .overlay
            .as_ref()
            .is_none_or(|o| o.dpi != dpi || o.size != size)
        {
            app.overlay = Some(Overlay::new(dpi, size)?);
        }
        let bounds = if action.is_some() {
            result_plan.bounds
        } else {
            focus.map_or_else(
                || app.surfaces.effect_bounds(app.work, app.dpi, size),
                |plan| plan.bounds,
            )
        };
        if action.is_some() {
            app.overlay.as_ref().unwrap().manifestation_with_transform(
                frame,
                bounds,
                navi_state::visual::Family::FracturedHalo,
                navi_state::visual::VisualStyle {
                    intensity: app.reactions.config.intensity,
                    seed: app.action_visual.seed,
                    temporal: None,
                },
                [(0.0, 0.0); 2],
                Some((result_plan.scale, result_plan.rotation)),
            )?;
        } else if let Some(plan) = focus {
            let mut frame = frame;
            if plan.drag {
                frame.progress = 0.5;
                frame.opacity = 0.85;
                frame.motion = false;
            }
            app.overlay.as_ref().unwrap().manifestation_with_transform(
                frame,
                bounds,
                plan.family,
                plan.style,
                plan.points,
                plan.variation
                    .map(|v| (plan.draw_scale, v.rotation_radians)),
            )?;
        } else {
            app.overlay
                .as_ref()
                .unwrap()
                .draw_scaled(frame, bounds, 1.0, &app.context_entries)?;
        }
        let remaining = if let Some((_, remaining)) = action {
            remaining
        } else {
            focus.map_or_else(
                || app.effects.remaining(app.now()),
                |plan| plan.remaining_ms,
            )
        };
        let interval = if frame.motion && focus.is_none_or(|p| !p.drag) {
            50.min(remaining).max(1)
        } else {
            remaining
        };
        if SetTimer(hwnd, 4, interval, None) == 0 {
            return Err(Error::from_win32());
        }
        app.timers[3] = true;
        Ok(())
    })();
    if let Err(e) = result {
        app.overlay = None;
        app.effects.clear();
        app.action_visual.clear();
        // A transient allocation/draw failure must not disable every future
        // reaction or unregister the mouse observer for the entire session.
        app.reactions.clear();
        app.surfaces.clear_focus_effect();
        let _ = std::fs::write(
            std::env::temp_dir().join("navi-effects-error.log"),
            e.to_string(),
        );
    }
}
