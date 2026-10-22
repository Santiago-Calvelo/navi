use crate::experiment::*;

const ITEMS: [(&str, &str, &str); 4] = [
    (
        "01",
        "una idea sin terminar",
        "Un lugar reservado para lo que todavia no tiene forma.",
    ),
    (
        "02",
        "el ultimo silencio",
        "No fue una interrupcion. Fue espacio para pensar.",
    ),
    (
        "03",
        "una ventana abierta",
        "Hay cosas que permanecen incluso cuando no las miras.",
    ),
    (
        "04",
        "volver al principio",
        "El recorrido no desaparece al cambiar de direccion.",
    ),
];

pub fn draw(f: &mut Frame, t: f32, reduced: bool) {
    let grow = smooth(0.03, 0.28, t);
    let change = if reduced { 0.0 } else { smooth(0.46, 0.57, t) };
    let selected = if change < 0.5 { 1 } else { 2 };
    let focus = if change < 0.5 {
        1.0 - change * 2.0
    } else {
        (change - 0.5) * 2.0
    };
    f.label("Herbario", 83.0, 101.0, 43.0, PAPER.alpha(grow), true);
    f.text("DE AUSENCIAS", 88.0, 158.0, 10.0, RED.alpha(grow));
    f.text(
        "04 MEMORIAS  /  NINGUN ARCHIVO REAL",
        88.0,
        183.0,
        9.0,
        DIM.alpha(grow * 0.8),
    );
    // The archive has no cards: entries are pressed, branching specimens hung on threads.
    for (i, item) in ITEMS.iter().enumerate() {
        let x = 115.0 + i as f32 * 126.0;
        let top = 240.0 + (i as f32 * 2.0).sin() * 20.0;
        let active = if i == selected { focus } else { 0.0 };
        let height = 190.0 + active * 40.0;
        let ink = if i == selected { ICE } else { DIM };
        f.line(
            (x, top),
            (x, top + height * grow),
            ink.alpha(0.35 + active * 0.4),
            0.7,
        );
        for b in 0..17 {
            let u = b as f32 / 17.0;
            let emergence = smooth(u * 0.12, 0.22 + u * 0.12, t);
            let y = top + height * u;
            let dir = if b % 2 == 0 { -1.0 } else { 1.0 };
            let span = (12.0 + noise(i * 21 + b) * 30.0 + active * 12.0) * emergence;
            let tip = (x + dir * span, y - 17.0 - span * 0.4);
            let p = vec![
                (x, y),
                (x + dir * span * 0.4, y - span * 0.12),
                tip,
                (x + dir * span * 0.54, y - 12.0 - span * 0.24),
            ];
            f.path(
                p,
                true,
                Some(ink.alpha(0.04 + active * 0.07)),
                Some((ink.alpha(0.28 + active * 0.42), 0.65)),
            );
            if b % 5 == 0 {
                f.line(
                    tip,
                    (tip.0 + dir * 9.0, tip.1 - 15.0),
                    RED.alpha(0.4 * emergence),
                    0.65,
                );
            }
        }
        f.text(item.0, x - 8.0, top + height + 24.0, 12.0, ink.alpha(grow));
        f.text(
            if i == selected { "PRESENTE" } else { "LATENTE" },
            x - 20.0,
            top + height + 44.0,
            8.0,
            ink.alpha(grow * 0.7),
        );
    }
    // Selected specimen unfolds a reading margin, not a rectangular detail panel.
    let detail = grow * focus;
    let sx = 115.0 + selected as f32 * 126.0;
    f.path(
        vec![
            (sx, 432.0),
            (sx + 30.0, 555.0),
            (615.0, 555.0),
            (653.0, 467.0),
        ],
        false,
        None,
        Some((ICE.alpha(detail * 0.4), 0.7)),
    );
    f.line((653.0, 229.0), (653.0, 467.0), ICE.alpha(detail * 0.5), 0.8);
    f.panel(669.0, 242.0, 273.0, 178.0, DARK.alpha(detail * 0.8));
    f.text(
        format!("ESPÉCIMEN {} / MEMORIA SIMULADA", ITEMS[selected].0),
        681.0,
        251.0,
        9.0,
        RED.alpha(detail),
    );
    let title = ITEMS[selected].1;
    let words: Vec<_> = title.split_whitespace().collect();
    f.label(
        words[..2].join(" "),
        679.0,
        281.0,
        30.0,
        PAPER.alpha(detail),
        true,
    );
    f.label(
        words[2..].join(" "),
        679.0,
        316.0,
        30.0,
        PAPER.alpha(detail),
        true,
    );
    let desc = ITEMS[selected].2;
    let mut line = String::new();
    let mut y = 374.0;
    for word in desc.split_whitespace() {
        if line.len() + word.len() > 34 {
            f.text(&line, 681.0, y, 10.0, DIM.alpha(detail));
            line.clear();
            y += 17.0;
        }
        line.push_str(word);
        line.push(' ');
    }
    f.text(line, 681.0, y, 10.0, DIM.alpha(detail));
    f.text(
        "SELECCION ENSAYADA / SIN CAPTURA DE CLICS",
        679.0,
        453.0,
        8.0,
        ICE.alpha(detail * 0.8),
    );
    f.text(
        "03  /  conservar no es poseer",
        88.0,
        590.0,
        10.0,
        DIM.alpha(grow),
    );
}
