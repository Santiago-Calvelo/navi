//! Session-local visual options. No persistence of desktop activity.
use super::*;
use navi_state::{
    reactions::{Bounds, DesktopEvent, DesktopKind, Intensity, Point, Reaction},
    visual::{Family, Placement},
};

unsafe fn item(menu: HMENU, id: usize, label: &str, checked: bool) {
    let text: Vec<u16> = label.encode_utf16().chain(Some(0)).collect();
    let _ = AppendMenuW(
        menu,
        MF_STRING | if checked { MF_CHECKED } else { MF_UNCHECKED },
        id,
        PCWSTR(text.as_ptr()),
    );
}

pub(super) unsafe fn append(menu: HMENU) {
    with_app(|a| {
        let c = &a.reactions.config;
        item(
            menu,
            51,
            &format!("Intensidad: {:?} · cambiar", c.intensity),
            false,
        );
        item(
            menu,
            52,
            &format!("Foco: {:?} · cambiar familia", c.focus_family),
            false,
        );
        item(
            menu,
            53,
            &format!("Posición: {:?} · cambiar", a.surfaces.placement),
            false,
        );
        item(
            menu,
            54,
            "Semilla reproducible: 42",
            a.surfaces.seed.is_some(),
        );
        item(
            menu,
            55,
            "Reaccionar a clics",
            c.enabled && c.events & 2 != 0,
        );
        item(
            menu,
            56,
            "Reaccionar a arrastres",
            c.enabled && c.events & 4 != 0,
        );
        item(
            menu,
            57,
            &format!("Clic: {:?} · cambiar familia", c.click_family),
            false,
        );
        item(
            menu,
            58,
            &format!("Arrastre: {:?} · cambiar familia", c.drag_family),
            false,
        );
        item(menu, 59, "FracturedHalo · Ctrl+Alt+5", false);
        item(menu, 60, "LayerStack v1 · Ctrl+Alt+6 · debug", false);
        item(menu, 61, "FracturedHalo al invocar NAVI", a.invocation_halo);
    });
}

pub(super) unsafe fn toggle_event(hwnd: HWND, bit: u8) {
    commit(hwnd, |a| {
        let enabled = !a.reactions.config.enabled || a.reactions.config.events & bit == 0;
        a.reactions.set_enabled(true);
        if enabled {
            a.reactions.config.events |= bit;
        } else {
            a.reactions.config.events &= !bit;
        }
        a.surfaces.clear_focus_effect();
        a.effects.clear();
        if a.reactions.config.events & 6 == 0 {
            a.cursor = None;
        } else if a.cursor.is_none() && a.passive_observers {
            match cursor_observer::CursorObserver::new(hwnd) {
                Ok(cursor) => a.cursor = Some(cursor),
                Err(e) => {
                    a.reactions.config.events &= !6;
                    let _ = std::fs::write(
                        std::env::temp_dir().join("navi-cursor-error.log"),
                        e.to_string(),
                    );
                }
            }
        }
    });
}

pub(super) unsafe fn command(hwnd: HWND, id: u32) -> bool {
    match id {
        55 => toggle_event(hwnd, 2),
        56 => toggle_event(hwnd, 4),
        59 => preview(hwnd, Family::FracturedHalo),
        60 => preview(hwnd, Family::StackV1),
        51..=54 | 57..=58 | 61 => commit(hwnd, |a| {
            a.surfaces.clear_focus_effect();
            a.reactions.clear();
            a.effects.clear();
            let c = &mut a.reactions.config;
            match id {
                51 => {
                    c.intensity = match c.intensity {
                        Intensity::Subtle => Intensity::Experimental,
                        Intensity::Experimental => Intensity::Anomalous,
                        Intensity::Anomalous => Intensity::Subtle,
                    }
                }
                52 => {
                    c.focus_family = match c.focus_family {
                        Family::StackV2 => Family::FracturedHalo,
                        Family::FracturedHalo => Family::StackV1,
                        _ => Family::StackV2,
                    }
                }
                53 => {
                    a.surfaces.placement = match a.surfaces.placement {
                        Placement::WindowAnchor => Placement::NearCursor,
                        Placement::NearCursor => Placement::SafeRandom,
                        Placement::SafeRandom => Placement::WindowAnchor,
                    }
                }
                54 => {
                    a.surfaces.seed = if a.surfaces.seed.is_some() {
                        None
                    } else {
                        Some(42)
                    }
                }
                57 => {
                    c.click_family = if c.click_family == Family::ClickPulse {
                        Family::StackV1
                    } else {
                        Family::ClickPulse
                    }
                }
                58 => {
                    c.drag_family = if c.drag_family == Family::DragConnector {
                        Family::StackV2
                    } else {
                        Family::DragConnector
                    }
                }
                61 => a.invocation_halo = !a.invocation_halo,
                _ => {}
            }
        }),
        _ => return false,
    }
    true
}

pub(super) unsafe fn preview(hwnd: HWND, family: Family) {
    commit(hwnd, |a| {
        focus_reaction::cancel(&mut a.reactions, &mut a.surfaces);
        a.effects.clear();
        let work = a.work;
        let mut p = POINT::default();
        let _ = GetCursorPos(&mut p);
        let monitor = MonitorFromPoint(p, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let (work, dpi) = if GetMonitorInfoW(monitor, &mut info).as_bool() {
            let mut dx = 96;
            let mut dy = 96;
            let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
            (
                Rect {
                    left: info.rcWork.left,
                    top: info.rcWork.top,
                    right: info.rcWork.right,
                    bottom: info.rcWork.bottom,
                },
                dx.max(96),
            )
        } else {
            (work, a.dpi)
        };
        let bounds = Bounds {
            left: work.left,
            top: work.top,
            right: work.right,
            bottom: work.bottom,
        };
        let event = DesktopEvent {
            kind: DesktopKind::WindowFocus,
            source: hwnd.0 as usize,
            bounds,
            work_area: bounds,
            point: Point { x: p.x, y: p.y },
            own: false,
            suppressed: false,
            dpi,
        };
        a.surfaces.begin_focus_effect(Reaction {
            event,
            start: a.now(),
            duration_ms: a.effects.duration_ms,
            intensity: a.reactions.config.intensity,
            family,
        });
    });
}
