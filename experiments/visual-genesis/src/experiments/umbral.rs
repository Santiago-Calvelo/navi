use crate::experiment::*;
use std::f32::consts::PI;

pub fn draw(f: &mut Frame, t: f32, reduced: bool) {
    let enter = smooth(0.03, 0.25, t);
    let answer = if reduced { 1.0 } else { smooth(0.43, 0.63, t) };
    let settle = if reduced { 1.0 } else { smooth(0.67, 0.84, t) };
    f.text("U M B R A L", 83.0, 112.0, 11.0, RED.alpha(enter));
    f.label("Antes de", 80.0, 157.0, 43.0, PAPER.alpha(enter), true);
    f.label("decir algo.", 80.0, 205.0, 43.0, PAPER.alpha(enter), true);
    f.text(
        "UNA CONVERSACION SIN CAJA DE TEXTO",
        84.0,
        276.0,
        9.0,
        DIM.alpha(enter * 0.8),
    );
    // A suspended pleat is the listening boundary: dense material becomes an aperture.
    for i in 0..75 {
        let u = i as f32 / 74.0;
        let envelope = (u * PI).sin().max(0.0).powf(0.7);
        let ripple = (u * PI * 7.0 - t * 13.0).sin() * (1.0 - settle) * 10.0;
        let aperture = envelope * answer * 65.0;
        let x = 505.0 + (u * PI * 2.0).sin() * 27.0;
        let y = 145.0 + u * 365.0;
        let width = (17.0 + 44.0 * envelope + ripple) * enter;
        let left = vec![
            (x - width - aperture, y),
            (x - 8.0 - aperture, y - 8.0 * envelope),
            (x - aperture, y - 5.0),
        ];
        let right = vec![
            (x + aperture, y + 5.0),
            (x + 8.0 + aperture, y + 8.0 * envelope),
            (x + width + aperture, y),
        ];
        f.path(
            left,
            false,
            None,
            Some((PAPER.alpha((0.15 + envelope * 0.55) * enter), 0.85)),
        );
        f.path(
            right,
            false,
            None,
            Some((ICE.alpha((0.15 + envelope * 0.55) * enter), 0.85)),
        );
        if i % 12 == 0 {
            f.line(
                (x - aperture, y - 5.0),
                (x + aperture, y + 5.0),
                RED.alpha((1.0 - answer) * enter * 0.7),
                0.6,
            );
        }
    }
    f.text("01 / INTENCION", 83.0, 357.0, 9.0, RED.alpha(enter));
    f.label("¿Que dejamos", 82.0, 385.0, 26.0, PAPER.alpha(enter), true);
    f.label("para mañana?", 82.0, 417.0, 26.0, PAPER.alpha(enter), true);
    f.line((86.0, 473.0), (319.0, 473.0), DIM.alpha(enter * 0.3), 0.7);
    f.text("ENTRADA SIMULADA", 84.0, 487.0, 8.0, DIM.alpha(enter));
    let a = answer;
    f.text("02 / POSIBILIDAD", 669.0, 218.0, 9.0, ICE.alpha(a));
    f.label("No todo", 665.0, 250.0, 39.0, PAPER.alpha(a), true);
    f.label("necesita", 665.0, 294.0, 39.0, PAPER.alpha(a), true);
    f.label("cerrarse hoy.", 665.0, 338.0, 34.0, PAPER.alpha(a), true);
    f.line((669.0, 408.0), (903.0, 408.0), ICE.alpha(a * 0.4), 0.7);
    f.text("CONSERVAR EL HILO", 669.0, 429.0, 10.0, PAPER.alpha(a));
    f.text(
        "sin ejecutar ninguna accion",
        669.0,
        451.0,
        9.0,
        DIM.alpha(a),
    );
    f.text(
        "RESPUESTA ESCRITA / SIN LLM NI MICROFONO",
        669.0,
        489.0,
        8.0,
        ICE.alpha(a * 0.8),
    );
    let status = if answer < 0.1 {
        "01   FORMAR UNA INTENCION"
    } else if settle < 0.5 {
        "02   ABRIR UN ESPACIO"
    } else {
        "03   DEJARLO REPOSAR"
    };
    f.text(status, 390.0, 568.0, 10.0, RED.alpha(enter));
    f.text(
        "INTERFAZ ESCENICA / TRANSICION SIMULADA",
        376.0,
        590.0,
        8.0,
        DIM.alpha(enter * 0.7),
    );
}
