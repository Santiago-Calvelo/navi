use crate::experiment::*;
use std::f32::consts::PI;

// A ruled silk membrane: every filament belongs to the same folded surface.
fn surface(u: f32, v: f32, opening: f32, turn: f32) -> Point {
    let bend = u * PI * 1.45 + turn;
    let width = opening * (24.0 + 108.0 * (u * PI).sin().max(0.0).powf(0.65));
    let x = (u - 0.5) * 620.0 + v * width * bend.cos();
    let y = 38.0 * (u * PI * 2.0 + turn * 0.4).sin() + v * width * bend.sin();
    let z = v * width * (bend + 0.8).cos() * 1.5 + 80.0 * (u * PI).sin();
    project(x, y, z, 500.0, 335.0)
}
pub fn draw(f: &mut Frame, t: f32) {
    let open = smooth(0.06, 0.36, t) * (1.0 - smooth(0.74, 0.96, t));
    let turn = -0.8 + smooth(0.30, 0.70, t) * PI * 1.2;
    let length = smooth(0.0, 0.20, t);
    // Narrow opaque ribbons occlude the back filaments, without an enclosing panel.
    for strip in 0..34 {
        let v = strip as f32 / 33.0 * 2.0 - 1.0;
        let mut points = Vec::new();
        for j in 0..81 {
            points.push(surface(j as f32 / 80.0 * length, v, open, turn));
        }
        let brightness = (v * 3.0 + turn).cos() * 0.5 + 0.5;
        let ink = if strip % 9 == 0 {
            RED.alpha(0.60)
        } else {
            PAPER.alpha(0.17 + brightness * 0.62)
        };
        if strip % 2 == 0 {
            let mut band = points.clone();
            for j in (0..81).rev() {
                band.push(surface(j as f32 / 80.0 * length, v + 0.023, open, turn));
            }
            f.path(band, true, Some(DARK.alpha(0.75)), None);
        }
        f.path(
            points,
            false,
            None,
            Some((ink, if strip == 0 || strip == 33 { 1.2 } else { 0.65 })),
        );
    }
    for j in 0..55 {
        let u = j as f32 / 54.0;
        if u > length {
            continue;
        }
        let points = (0..31)
            .map(|k| surface(u, k as f32 / 30.0 * 2.0 - 1.0, open, turn))
            .collect();
        f.path(
            points,
            false,
            None,
            Some((ICE.alpha(0.08 + 0.14 * open), 0.55)),
        );
    }
    // The seam is sewn with finite, travelling stitches, then vanishes into the cloth.
    let stitch = smooth(0.03, 0.25, t) * (1.0 - smooth(0.34, 0.48, t));
    for i in 0..24 {
        let u = i as f32 / 24.0;
        if u > length {
            continue;
        }
        let (x, y) = surface(u, 0.0, open, turn);
        let h = 8.0 + noise(i) * 17.0;
        f.line(
            (x - 4.0, y - h * stitch),
            (x + 4.0, y + h * stitch),
            RED.alpha(stitch),
            1.1,
        );
    }
    let (x, y) = surface(0.74, 1.0, open, turn);
    let tag = smooth(0.30, 0.48, t) * (1.0 - smooth(0.73, 0.84, t));
    f.line((x, y), (785.0, 480.0), DIM.alpha(tag * 0.45), 0.7);
    f.line((785.0, 480.0), (853.0, 480.0), DIM.alpha(tag * 0.45), 0.7);
    f.text("CONTINUIDAD", 785.0, 490.0, 10.0, PAPER.alpha(tag));
    f.text(
        "sin superficie de origen",
        785.0,
        510.0,
        9.0,
        DIM.alpha(tag),
    );
    f.label(
        "algo permanece",
        88.0,
        507.0,
        30.0,
        PAPER.alpha(tag * 0.85),
        true,
    );
    f.text("01  /  SUTURA", 90.0, 551.0, 10.0, RED.alpha(tag));
}
