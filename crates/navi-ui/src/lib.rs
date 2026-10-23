use navi_state::{Mode, Phase, Session};

pub fn effect_size(mask: u8) -> SurfaceSize {
    if mask & (navi_state::holograms::CONTEXT | navi_state::holograms::STACK) != 0 {
        SurfaceSize {
            width: 540.0,
            height: 410.0,
        }
    } else {
        SurfaceSize {
            width: 260.0,
            height: 240.0,
        }
    }
}

pub const WIDTH: f32 = 760.0;
pub const HEIGHT: f32 = 430.0;
pub const INK: (u32, f32) = (0x080f16, 0.90);
pub const MINT: (u32, f32) = (0xb7e3d6, 0.94);
pub const TEXT: (u32, f32) = (0xe0e9e6, 1.0);
pub const MUTED: (u32, f32) = (0x99b7af, 0.90);
pub const LINE: (u32, f32) = (0x94c9bb, 0.46);
pub const ERROR: (u32, f32) = (0xd58c8c, 1.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Invoke,
    Expand,
    Hide,
    Close,
    Pin,
    Voice,
    Text,
    Demo,
    Input,
    Drag,
    None,
}

pub fn hit(x: f32, y: f32) -> Hit {
    if !(24.0..=736.0).contains(&x) || !(24.0..=406.0).contains(&y) {
        return Hit::None;
    }
    if y < 72.0 {
        return Hit::Drag;
    }
    if y < 118.0 {
        return match x {
            x if x < 116.0 => Hit::Voice,
            x if x < 215.0 => Hit::Text,
            x if x > 608.0 => Hit::Demo,
            _ => Hit::None,
        };
    }
    Hit::Input
}

pub fn headline(s: &Session) -> &str {
    if let Some(entry) = s.keyboard_entry.filter(|_| s.input.is_empty()) {
        return if entry.prompting() {
            "¿Qué necesitás?"
        } else if entry.caret_visible() {
            "_"
        } else {
            ""
        };
    }
    if !s.answer.is_empty() && !s.executing && s.input.is_empty() {
        return &s.answer;
    }
    match s.phase {
        Phase::Idle => "En reposo.",
        Phase::Invoked if !s.input.is_empty() => &s.input,
        Phase::Invoked => "¿Qué necesitás?",
        Phase::Listening => "Te escucho.",
        Phase::Processing => "Interpretando…",
        Phase::Responding => "Estoy acá.",
        Phase::Error => "La señal se interrumpió.",
    }
}

pub fn caption(s: &Session) -> &str {
    if s.executing {
        return "Procesando una solicitud local.";
    }
    if !s.answer.is_empty() && !s.executing {
        return &s.answer;
    }
    match s.phase {
        Phase::Idle => "Alt + Espacio para conversar.",
        Phase::Invoked if s.mode == Mode::Text => {
            "Escribí un mensaje. Enter para probar la respuesta."
        }
        Phase::Invoked => "Un espacio para pensar en voz alta.",
        Phase::Listening => "Señal simulada / no se utiliza el micrófono",
        Phase::Processing => "Procesando tu mensaje en la sesión de demostración.",
        Phase::Responding => "Mensaje recibido. Esta respuesta es una demostración local.",
        Phase::Error => "Presioná F1 para volver a iniciar la sesión.",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    Beacon,
    Compact,
    Console,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSize {
    pub width: f32,
    pub height: f32,
}

pub fn compact_message(s: &Session) -> String {
    let text = headline(s);
    // A fixed monospace column width makes the content-driven height deterministic.
    // Explicit wrapping also keeps long unbroken input inside the native surface.
    let mut result = String::new();
    let mut column = 0;
    for word in text.split_whitespace() {
        let length = word.chars().count();
        if column > 0 {
            if column + 1 + length > 32 {
                result.push('\n');
                column = 0;
            } else {
                result.push(' ');
                column += 1;
            }
        }
        for c in word.chars() {
            if column == 32 {
                result.push('\n');
                column = 0;
            }
            result.push(c);
            column += 1;
        }
    }
    result
}

pub fn surface_size(kind: SurfaceKind, s: &Session) -> SurfaceSize {
    match kind {
        SurfaceKind::Beacon => SurfaceSize {
            width: 88.0,
            height: 28.0,
        },
        SurfaceKind::Console => SurfaceSize {
            width: WIDTH,
            height: HEIGHT,
        },
        SurfaceKind::Compact => SurfaceSize {
            width: 320.0,
            height: 84.0 + compact_message(s).lines().count().max(1) as f32 * 20.0,
        },
    }
}

pub fn surface_hit(kind: SurfaceKind, size: SurfaceSize, x: f32, y: f32) -> Hit {
    if x < 2.0 || y < 2.0 || x > size.width - 2.0 || y > size.height - 2.0 {
        return Hit::None;
    }
    match kind {
        SurfaceKind::Console => hit(x, y),
        SurfaceKind::Beacon => Hit::Invoke,
        SurfaceKind::Compact => Hit::Input,
    }
}

pub fn compact_status(s: &Session) -> &str {
    if s.keyboard_entry.is_some() {
        return "escribí / enter";
    }
    if s.executing {
        return "procesando solicitud";
    }
    if s.awaiting_confirmation {
        return "confirmación pendiente";
    }
    if !s.answer.is_empty() {
        return if s.input.is_empty() {
            "respuesta local"
        } else {
            "respuesta lista / borrador"
        };
    }
    match s.phase {
        Phase::Idle => "en reposo",
        Phase::Invoked => "escribí / enter",
        Phase::Listening => "escucha simulada",
        Phase::Processing => "procesando",
        Phase::Responding => "respuesta local",
        Phase::Error => "F1 · reintentar",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navi_state::Event;

    #[test]
    fn compact_keyboard_readiness_replaces_prompt_without_fake_audio() {
        let mut session = Session::default();
        session.apply(Event::KeyboardInvoke(50));
        assert_eq!(headline(&session), "¿Qué necesitás?");
        session.apply(Event::KeyboardTick(2_050));
        assert_eq!(headline(&session), "_");
        session.apply(Event::KeyboardTick(2_550));
        assert_eq!(headline(&session), "");
        assert_eq!(surface_size(SurfaceKind::Compact, &session).height, 104.0);
        session.apply(Event::Character('h'));
        assert_eq!(headline(&session), "h");
        assert_eq!(compact_status(&session), "escribí / enter");
    }
}
