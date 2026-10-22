use crate::experiment::*;

pub fn draw(f: &mut Frame, t: f32) {
    let alignment = smooth(0.10, 0.42, t) * (1.0 - smooth(0.68, 0.93, t));
    let dissolve = smooth(0.70, 0.96, t);
    let rows = [
        ("ESTOY", 180.0, 190.0, 84.0),
        ("ENTRE", 335.0, 287.0, 84.0),
        ("TUS COSAS", 205.0, 392.0, 64.0),
    ];
    let mut ordinal = 0;
    // Each letter approaches from a different depth. At alignment the debris reads.
    for (word, x, y, size) in rows {
        for (j, ch) in word.chars().enumerate() {
            ordinal += 1;
            if ch == ' ' {
                continue;
            }
            let n = noise(ordinal);
            let target = (x + j as f32 * size * 0.64, y);
            let source = (70.0 + n * 790.0, 105.0 + noise(ordinal + 51) * 455.0);
            let local = smooth(n * 0.09, 0.83, alignment);
            let px = mix(source.0, target.0, local) + dissolve * (n - 0.5) * 230.0;
            let py = mix(source.1, target.1, local) - dissolve * (50.0 + n * 110.0);
            let scale = mix(0.15 + n * 0.30, 1.0, local) * (1.0 - dissolve * 0.5);
            // Displaced registration shadows are deliberately not a glow or a HUD.
            for depth in (1..=4).rev() {
                let d = depth as f32;
                let dx = (1.0 - local) * d * 22.0 + dissolve * d * 8.0;
                let dy = (1.0 - local) * d * -12.0;
                f.label(
                    ch.to_string(),
                    px + dx,
                    py + dy,
                    size * scale,
                    ICE.alpha((1.0 - local) * 0.12 / d),
                    true,
                );
            }
            f.label(
                ch.to_string(),
                px,
                py,
                size * scale,
                PAPER.alpha(0.4 + local * 0.6),
                true,
            );
            if local < 0.90 {
                f.text(
                    format!("{:02}.{:03}", ordinal, (n * 1000.0) as usize),
                    px,
                    py + size * scale + 5.0,
                    8.0,
                    RED.alpha((1.0 - local) * 0.8),
                );
            }
        }
    }
    // A narrow moving erasure changes depth ordering across the large phrase.
    let pass = smooth(0.44, 0.64, t);
    if pass > 0.0 && pass < 1.0 {
        let y = 170.0 + pass * 340.0;
        f.panel(145.0, y, 650.0, 3.5, DARK);
        f.line((145.0, y + 4.0), (795.0, y + 4.0), RED.alpha(0.65), 0.6);
        f.text("el espacio recuerda", 785.0, y - 16.0, 9.0, RED);
    }
    let annotation = smooth(0.35, 0.48, t) * (1.0 - smooth(0.72, 0.87, t));
    f.line(
        (147.0, 202.0),
        (147.0, 478.0),
        DIM.alpha(annotation * 0.45),
        0.7,
    );
    f.text(
        "NO ES UNA NOTIFICACION",
        180.0,
        538.0,
        10.0,
        DIM.alpha(annotation),
    );
    f.text(
        "es una coincidencia de posiciones",
        180.0,
        558.0,
        11.0,
        PAPER.alpha(annotation * 0.7),
    );
    f.text(
        "02 / PALIMPSESTO",
        720.0,
        565.0,
        10.0,
        RED.alpha(annotation),
    );
}
