use super::model::{caret_line, lines, message_height, Model, Phase, COLS, INPUT_LIMIT};
use crate::experiment::{smooth, Frame, Ink, Mark};

pub const W: f32 = 1032.0;
pub const H: f32 = 832.0;
pub const CW: f32 = 108.0;
pub const CH: f32 = 220.0;
pub const TOP: f32 = 228.0;
pub const BOTTOM: f32 = 540.0;
pub const LINE: f32 = 25.0;
pub const TEXT_X: f32 = 226.0;
pub const EDIT_X: f32 = 220.0;
pub const EDIT_Y: f32 = 662.0;
pub const EDIT_COLS: usize = 62;
pub const ADVANCE: f32 = 10.45;
const INK: Ink = Ink(0xedddce, 1.0);
const MUTED: Ink = Ink(0xb9a1b3, 1.0);
const LILAC: Ink = Ink(0xc6b0e2, 1.0);
const COPPER: Ink = Ink(0xd99975, 1.0);
const VEIL: Ink = Ink(0x18131d, 0.98);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    None,
    Open,
    Fold,
    Quit,
    Tour,
    History,
    Copy,
    Cancel,
    Allow,
    Deny,
    Latest,
    Input,
    Scroll,
    Jump(usize),
}

pub struct Layout {
    pub offsets: Vec<f32>,
    pub total: f32,
}
pub fn layout(model: &Model) -> Layout {
    let mut offsets = Vec::new();
    let mut y = 0.0;
    for m in &model.messages {
        offsets.push(y);
        y += message_height(m);
    }
    Layout { offsets, total: y }
}
pub fn max_scroll(model: &Model) -> f32 {
    (layout(model).total - (BOTTOM - TOP)).max(0.0)
}
pub fn editor_rows(model: &Model) -> (Vec<std::ops::Range<usize>>, usize) {
    let rows = lines(&model.editor.chars, EDIT_COLS);
    let first = caret_line(&rows, model.editor.caret).saturating_sub(2);
    (rows, first)
}
pub fn editor_position(model: &Model, x: f32, y: f32) -> usize {
    let (rows, first) = editor_rows(model);
    let row = (first + ((y - EDIT_Y).max(0.0) / LINE) as usize).min(rows.len() - 1);
    let col = ((x - EDIT_X).max(0.0) / ADVANCE).round() as usize;
    (rows[row].start + col).min(rows[row].end)
}
fn within(x: f32, y: f32, r: [f32; 4]) -> bool {
    x >= r[0] && x <= r[2] && y >= r[1] && y <= r[3]
}
pub fn hit(model: &Model, expanded: bool, x: f32, y: f32) -> Action {
    if !expanded {
        return if x > 78.0 && y < 30.0 {
            Action::Quit
        } else if x >= 12.0 {
            Action::Open
        } else {
            Action::None
        };
    }
    if within(x, y, [977.0, 27.0, 1025.0, 64.0]) {
        return Action::Quit;
    }
    if within(x, y, [875.0, 27.0, 972.0, 64.0]) {
        return Action::Fold;
    }
    if within(x, y, [193.0, 75.0, 388.0, 109.0]) {
        return Action::Tour;
    }
    if within(x, y, [687.0, 174.0, 806.0, 210.0]) {
        return Action::History;
    }
    if within(x, y, [814.0, 174.0, 948.0, 210.0]) {
        return Action::Copy;
    }
    if within(x, y, [203.0, 645.0, 906.0, 751.0]) {
        return Action::Input;
    }
    if within(x, y, [916.0, TOP, 948.0, BOTTOM]) {
        return Action::Scroll;
    }
    if within(x, y, [691.0, 541.0, 910.0, 570.0]) {
        return Action::Latest;
    }
    if model.phase == Phase::Permission {
        if within(x, y, [641.0, 576.0, 790.0, 623.0]) {
            return Action::Allow;
        }
        if within(x, y, [797.0, 576.0, 932.0, 623.0]) {
            return Action::Deny;
        }
    }
    if model.phase == Phase::Thinking && within(x, y, [805.0, 576.0, 935.0, 623.0]) {
        return Action::Cancel;
    }
    if within(x, y, [155.0, TOP, 201.0, BOTTOM]) {
        let n = model.messages.len();
        let index = (((y - TOP) / (BOTTOM - TOP)) * n as f32) as usize;
        return Action::Jump(index.min(n - 1));
    }
    Action::None
}

fn word(f: &mut Frame, s: &str, x: f32, y: f32, ink: Ink, a: Action, hover: Action) {
    f.text(s, x, y, 11.0, if a == hover { INK } else { ink });
    if a == hover {
        f.line(
            (x, y + 18.0),
            (x + s.chars().count() as f32 * 6.0, y + 18.0),
            COPPER,
            0.8,
        );
    }
}

pub fn compact(model: &Model, hover: Action) -> Frame {
    let mut f = Frame::new(1.0);
    // A vertical graphite incision attached to the physical screen edge.
    f.path(
        vec![
            (36.0, 31.0),
            (108.0, 0.0),
            (108.0, 220.0),
            (37.0, 187.0),
            (48.0, 112.0),
        ],
        true,
        Some(VEIL),
        None,
    );
    for i in 0..9 {
        let d = i as f32;
        f.path(
            vec![
                (48.0 + d * 2.2, 41.0 - d),
                (45.0 + d * 2.5, 80.0),
                (58.0 + d, 116.0),
                (42.0 + d * 3.0, 178.0 + d),
            ],
            false,
            None,
            Some((LILAC.alpha(0.13 + d * 0.045), 0.7)),
        );
    }
    for (i, ch) in "NAVI".chars().enumerate() {
        f.text(ch.to_string(), 81.0, 68.0 + i as f32 * 18.0, 12.0, INK);
    }
    f.text(
        if model.phase == Phase::Permission {
            "decidir"
        } else if model.phase == Phase::Thinking {
            "..."
        } else {
            "abrir"
        },
        56.0,
        181.0,
        10.0,
        if hover == Action::Open { INK } else { COPPER },
    );
    f.text("×", 87.0, 9.0, 15.0, MUTED);
    if !model.editor.chars.is_empty() {
        f.line((67.0, 158.0), (92.0, 158.0), COPPER, 2.0);
    }
    f
}

pub fn expanded(
    model: &Model,
    opening: f32,
    now: f32,
    focused: bool,
    hover: Action,
    tour: bool,
    reduced: bool,
) -> Frame {
    let mut f = Frame::new(1.0);
    let q = smooth(0.0, 1.0, opening);
    let edge = 1003.0 - 838.0 * q;
    // The membrane grows out of the screen edge; irregular cuts leave desktop visible.
    f.path(
        vec![
            (edge + 40.0, 0.0),
            (W, 0.0),
            (W, H),
            (edge - 18.0, H),
            (edge + 12.0, 592.0),
            (edge - 15.0, 331.0),
            (edge + 28.0, 150.0),
        ],
        true,
        Some(VEIL),
        None,
    );
    for i in 0..18 {
        let x = edge - 40.0 + i as f32 * 3.0;
        f.line(
            (x, 130.0 + i as f32 * 3.0),
            (x + 10.0, H - i as f32 * 5.0),
            LILAC.alpha(0.012 + i as f32 * 0.0015),
            2.0,
        );
    }
    let energy = if model.phase == Phase::Thinking && !reduced {
        (now * 2.5).sin() * 8.0
    } else {
        0.0
    };
    for i in 0..15 {
        let k = i as f32;
        let x = 995.0 + k * 1.8;
        f.path(
            vec![
                (x, 120.0 + k * 5.0),
                (x - 15.0 - energy, 282.0),
                (x + 4.0, 414.0),
                (x - 24.0 + energy, 541.0),
                (x, 720.0 - k * 3.0),
            ],
            false,
            None,
            Some((LILAC.alpha(0.08 + k * 0.025), 0.7)),
        );
    }
    if q < 0.06 {
        return f;
    }
    f.marks.push(Mark::Clip([edge + 12.0, 0.0, 976.0, H]));
    let a = smooth(0.25, 0.95, q);
    f.text("N A V I", 205.0, 36.0, 12.0, INK.alpha(a));
    f.text(
        "LA COMPUTADORA, EN EL MARGEN",
        322.0,
        39.0,
        9.0,
        MUTED.alpha(a),
    );
    word(
        &mut f,
        "retirar",
        888.0,
        39.0,
        MUTED.alpha(a),
        Action::Fold,
        hover,
    );
    word(
        &mut f,
        if tour {
            "recorrido en curso"
        } else {
            "recorrer la experiencia →"
        },
        205.0,
        85.0,
        COPPER.alpha(a),
        Action::Tour,
        hover,
    );
    f.label(
        if model.phase == Phase::Permission {
            "Antes de cambiar algo."
        } else if model.phase == Phase::Thinking {
            "Estoy reuniendo el hilo."
        } else {
            "Podemos seguir acá."
        },
        201.0,
        118.0,
        36.0,
        INK.alpha(a),
        true,
    );
    f.text(
        "PROTOTIPO LOCAL / RESPUESTAS SIMULADAS",
        206.0,
        184.0,
        9.0,
        MUTED.alpha(a),
    );
    word(
        &mut f,
        "hilo extenso",
        696.0,
        184.0,
        MUTED.alpha(a),
        Action::History,
        hover,
    );
    word(
        &mut f,
        "copiar hilo",
        825.0,
        184.0,
        MUTED.alpha(a),
        Action::Copy,
        hover,
    );
    f.line((205.0, 215.0), (943.0, 215.0), MUTED.alpha(0.25 * a), 0.7);

    let lay = layout(model);
    let maximum = (lay.total - (BOTTOM - TOP)).max(0.0);
    let scroll = model.scroll.clamp(0.0, maximum);
    let from = maximum - scroll;
    // Reading is one continuous manuscript; author labels remain in its margin.
    f.marks.push(Mark::Clip([201.0, TOP, 908.0, BOTTOM]));
    for (index, m) in model.messages.iter().enumerate() {
        let y = TOP + lay.offsets[index] - from;
        let chars: Vec<_> = m.text.chars().collect();
        let rows = lines(&chars, COLS);
        if y + rows.len() as f32 * LINE + 45.0 < TOP || y > BOTTOM {
            continue;
        }
        let fresh = if index + 1 == model.messages.len() && !m.user && !reduced {
            smooth(0.0, 0.75, now - model.reveal)
        } else {
            1.0
        };
        f.text(
            if m.user {
                "VOS"
            } else if m.note {
                "HUELLA"
            } else {
                "NAVI"
            },
            TEXT_X,
            y,
            9.0,
            if m.user {
                COPPER.alpha(a)
            } else {
                LILAC.alpha(a)
            },
        );
        for (row, r) in rows.iter().enumerate() {
            let text: String = chars[r.clone()].iter().collect();
            f.text(
                text,
                TEXT_X,
                y + 21.0 + row as f32 * LINE,
                18.0,
                INK.alpha(a * fresh),
            );
        }
        if m.note {
            f.line(
                (210.0, y),
                (210.0, y + rows.len() as f32 * LINE + 20.0),
                COPPER.alpha(a * 0.7),
                1.2,
            );
        }
    }
    f.marks.push(Mark::Unclip);
    // A ledger of interventions doubles as a sparse navigation instrument.
    let n = model.messages.len();
    for i in 0..n {
        let y = TOP + (i as f32 + 0.5) / n as f32 * (BOTTOM - TOP);
        let active = lay.offsets[i] >= from - 35.0 && lay.offsets[i] < from + BOTTOM - TOP;
        f.line(
            (176.0, y),
            (if active { 195.0 } else { 185.0 }, y),
            if active {
                COPPER.alpha(a)
            } else {
                MUTED.alpha(a * 0.35)
            },
            if active { 1.5 } else { 0.7 },
        );
    }
    if maximum > 0.0 {
        let track = BOTTOM - TOP;
        let thumb = (track * track / lay.total).max(22.0);
        let y = TOP + (from / maximum) * (track - thumb);
        f.line((930.0, TOP), (930.0, BOTTOM), MUTED.alpha(a * 0.16), 1.0);
        f.line((930.0, y), (930.0, y + thumb), LILAC.alpha(a * 0.75), 2.0);
        word(
            &mut f,
            if model.unread {
                "hay algo nuevo ↓"
            } else if scroll > 1.0 {
                "volver al presente ↓"
            } else {
                "rueda para leer el hilo"
            },
            701.0,
            548.0,
            MUTED.alpha(a),
            Action::Latest,
            hover,
        );
    }
    match model.phase {
        Phase::Permission => {
            f.path(
                vec![
                    (204.0, 579.0),
                    (219.0, 568.0),
                    (938.0, 568.0),
                    (938.0, 628.0),
                    (219.0, 628.0),
                ],
                true,
                Some(Ink(0x31232c, 0.94 * a)),
                None,
            );
            f.text("REUNIR ESTE HILO", 229.0, 580.0, 11.0, COPPER.alpha(a));
            f.text(
                "Una memoria simulada. Ningún archivo cambia.",
                229.0,
                601.0,
                10.0,
                MUTED.alpha(a),
            );
            word(
                &mut f,
                "sí, continuar",
                651.0,
                592.0,
                INK.alpha(a),
                Action::Allow,
                hover,
            );
            word(
                &mut f,
                "dejarlo así",
                808.0,
                592.0,
                MUTED.alpha(a),
                Action::Deny,
                hover,
            );
        }
        Phase::Thinking => {
            for i in 0..22 {
                let k = i as f32;
                let p = if reduced {
                    0.5
                } else {
                    ((now * 0.7 + k * 0.033).fract() * std::f32::consts::PI).sin()
                };
                f.line(
                    (220.0 + k * 3.0, 599.0 - p * 12.0),
                    (220.0 + k * 3.0, 602.0 + p * 8.0),
                    LILAC.alpha(a * (0.2 + p * 0.65)),
                    0.8,
                );
            }
            f.text(
                "La respuesta se está formando. Podés seguir escribiendo.",
                305.0,
                592.0,
                10.0,
                MUTED.alpha(a),
            );
            word(
                &mut f,
                "interrumpir",
                815.0,
                592.0,
                COPPER.alpha(a),
                Action::Cancel,
                hover,
            );
        }
        Phase::Quiet => {
            f.text(
                if model.notice.is_empty() {
                    "El espacio permanece. No hace falta llenarlo."
                } else {
                    &model.notice
                },
                220.0,
                596.0,
                10.0,
                MUTED.alpha(a * 0.8),
            );
        }
    }

    f.text(
        "TU ESCRITURA",
        220.0,
        635.0,
        9.0,
        if focused {
            COPPER.alpha(a)
        } else {
            MUTED.alpha(a)
        },
    );
    f.line(
        (214.0, 653.0),
        (896.0, 653.0),
        if focused {
            COPPER.alpha(a * 0.65)
        } else {
            MUTED.alpha(a * 0.3)
        },
        0.8,
    );
    f.marks.push(Mark::Clip([210.0, 658.0, 900.0, 747.0]));
    let (rows, first) = editor_rows(model);
    let select = model.editor.selection();
    if model.editor.chars.is_empty() {
        f.text(
            "Escribí acá. Enter continúa el hilo.",
            EDIT_X,
            EDIT_Y,
            18.0,
            MUTED.alpha(a * 0.7),
        );
    }
    for (row, r) in rows.iter().enumerate().skip(first).take(3) {
        let y = EDIT_Y + (row - first) as f32 * LINE;
        let start = select.start.max(r.start);
        let end = select.end.min(r.end);
        if end > start {
            f.panel(
                EDIT_X + (start - r.start) as f32 * ADVANCE,
                y,
                (end - start) as f32 * ADVANCE,
                24.0,
                LILAC.alpha(0.22 * a),
            );
        }
        let s: String = model.editor.chars[r.clone()].iter().collect();
        f.text(s, EDIT_X, y, 19.0, INK.alpha(a));
    }
    if focused {
        let row = caret_line(&rows, model.editor.caret);
        let x = EDIT_X + (model.editor.caret - rows[row].start) as f32 * ADVANCE;
        let y = EDIT_Y + (row - first) as f32 * LINE;
        f.line((x, y + 3.0), (x, y + 23.0), COPPER.alpha(a), 1.5);
    }
    f.marks.push(Mark::Unclip);
    f.text(
        match model.phase {
            Phase::Quiet => "Enter · continuar    Shift+Enter · nueva línea    Esc · retirar",
            Phase::Thinking => {
                "Prepará el próximo mensaje mientras se forma la respuesta. Esc · retirar"
            }
            Phase::Permission => {
                "Tu borrador se conserva. Decidí arriba antes de continuar. Esc · retirar"
            }
        },
        220.0,
        757.0,
        10.0,
        MUTED.alpha(a),
    );
    f.text(
        format!("{} / {}", model.editor.chars.len(), INPUT_LIMIT),
        811.0,
        782.0,
        9.0,
        MUTED.alpha(a * 0.65),
    );
    f.text(
        if tour {
            "RECORRIDO AUTOMÁTICO · TOCÁ O ESCRIBÍ PARA TOMAR EL CONTROL"
        } else {
            "BORRADOR Y CONVERSACIÓN CONSERVADOS DURANTE ESTA SESIÓN"
        },
        220.0,
        802.0,
        8.0,
        MUTED.alpha(a * 0.7),
    );
    f.marks.push(Mark::Unclip);
    f.text("×", 991.0, 34.0, 18.0, MUTED);
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_states_keep_clips_balanced_and_geometry_finite() {
        let mut model = Model::new();
        model.history();
        model.editor.set("Un borrador que permanece.\nOtra línea.");
        for phase in [Phase::Quiet, Phase::Thinking, Phase::Permission] {
            model.phase = phase;
            for opening in [0.0, 0.1, 0.5, 1.0] {
                let frame = expanded(&model, opening, 1.0, true, Action::Input, false, false);
                let mut clips = 0;
                for mark in frame.marks {
                    match mark {
                        Mark::Clip(r) => {
                            assert!(r[2] >= r[0] && r[3] >= r[1]);
                            clips += 1;
                        }
                        Mark::Unclip => {
                            clips -= 1;
                            assert!(clips >= 0);
                        }
                        Mark::Path { points, .. } => {
                            assert!(points.iter().all(|(x, y)| x.is_finite() && y.is_finite()))
                        }
                        Mark::Text { origin: (x, y), .. } => {
                            assert!(x.is_finite() && y.is_finite())
                        }
                    }
                }
                assert_eq!(clips, 0);
            }
        }
    }
}
