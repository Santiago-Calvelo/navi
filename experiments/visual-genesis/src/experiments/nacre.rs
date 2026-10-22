use crate::experiment::*;
use std::f32::consts::{PI, TAU};

pub fn draw(f: &mut Frame, t: f32) {
    let reveal = smooth(0.04, 0.32, t);
    let split = smooth(0.30, 0.54, t) * (1.0 - smooth(0.67, 0.86, t));
    let rotation = -0.60 + smooth(0.12, 0.84, t) * 1.1;
    // A sectional fossil, not rings: irregular closed cuts through a swept volume.
    for i in 0..58 {
        let u = i as f32 / 57.0;
        if u > reveal {
            continue;
        }
        let side = if u < 0.5 { -1.0 } else { 1.0 };
        let radius = 18.0 + 122.0 * (PI * u).sin().max(0.0).powf(0.75);
        let lift = side * split * (35.0 + 45.0 * (u - 0.5).abs());
        let mut contour = Vec::with_capacity(97);
        for j in 0..97 {
            let a = j as f32 / 96.0 * TAU;
            let r = radius
                * (1.0 + 0.13 * (a * 3.0 + u * 8.0).sin() + 0.06 * (a * 5.0 - u * 3.0).cos());
            let x = r * a.cos();
            let z = r * a.sin();
            let y = (u - 0.5) * 310.0 + lift + 15.0 * (a * 2.0 + u * 5.0).sin();
            contour.push(project(
                x * rotation.cos() + z * rotation.sin() + split * side * 34.0,
                y * 0.86 - z * 0.42,
                z * rotation.cos() - x * rotation.sin(),
                500.0,
                335.0,
            ));
        }
        let edge = if i == 28 || i == 29 { RED } else { PAPER };
        f.path(
            contour,
            true,
            Some(DARK.alpha(0.82)),
            Some((edge.alpha(0.20 + u * 0.65), if i == 28 { 1.4 } else { 0.7 })),
        );
    }
    // A blank indexed interval is the discovery; no icon sits in the centre.
    let visible = smooth(0.31, 0.53, t) * (1.0 - smooth(0.65, 0.83, t));
    f.line(
        (521.0, 337.0),
        (755.0, 305.0),
        RED.alpha(visible * 0.7),
        0.65,
    );
    f.line(
        (755.0, 305.0),
        (813.0, 305.0),
        RED.alpha(visible * 0.7),
        0.65,
    );
    f.text("[  AUSENCIA  ]", 758.0, 273.0, 11.0, RED.alpha(visible));
    f.text(
        "no se encontro un nucleo",
        758.0,
        317.0,
        9.0,
        PAPER.alpha(visible),
    );
    f.label(
        "lo que falta",
        100.0,
        470.0,
        32.0,
        PAPER.alpha(reveal),
        true,
    );
    f.text(
        "TAMBIEN TIENE VOLUMEN",
        103.0,
        518.0,
        10.0,
        DIM.alpha(reveal),
    );
    for i in 0..12 {
        let y = 200.0 + i as f32 * 16.0;
        f.line(
            (183.0, y),
            (if i % 3 == 0 { 201.0 } else { 190.0 }, y),
            DIM.alpha(reveal * 0.3),
            0.7,
        );
    }
    f.text("03 / NACRE", 758.0, 544.0, 10.0, RED.alpha(reveal));
}
