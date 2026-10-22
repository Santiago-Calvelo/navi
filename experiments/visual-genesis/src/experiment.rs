pub const WIDTH: f32 = 1000.0;
pub const HEIGHT: f32 = 700.0;
pub type Point = (f32, f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ink(pub u32, pub f32);
impl Ink {
    pub fn alpha(self, alpha: f32) -> Self {
        Self(self.0, self.1 * alpha.clamp(0.0, 1.0))
    }
}
pub const PAPER: Ink = Ink(0xe6e2d4, 1.0);
pub const DIM: Ink = Ink(0x9baca9, 1.0);
pub const RED: Ink = Ink(0xde674d, 1.0);
pub const ICE: Ink = Ink(0x9dd6cf, 1.0);
pub const DARK: Ink = Ink(0x101c21, 0.96);

#[derive(Debug, PartialEq)]
pub enum Mark {
    Path {
        points: Vec<Point>,
        closed: bool,
        fill: Option<Ink>,
        stroke: Option<(Ink, f32)>,
    },
    Text {
        value: String,
        origin: Point,
        size: f32,
        ink: Ink,
        serif: bool,
    },
}

#[derive(Debug, PartialEq)]
pub struct Frame {
    pub marks: Vec<Mark>,
    pub opacity: f32,
}
impl Frame {
    pub fn new(opacity: f32) -> Self {
        Self {
            marks: Vec::with_capacity(700),
            opacity,
        }
    }
    pub fn path(
        &mut self,
        points: Vec<Point>,
        closed: bool,
        fill: Option<Ink>,
        stroke: Option<(Ink, f32)>,
    ) {
        if points.len() >= 2 {
            self.marks.push(Mark::Path {
                points,
                closed,
                fill,
                stroke,
            });
        }
    }
    pub fn line(&mut self, a: Point, b: Point, ink: Ink, width: f32) {
        self.path(vec![a, b], false, None, Some((ink, width)));
    }
    pub fn text(&mut self, value: impl Into<String>, x: f32, y: f32, size: f32, ink: Ink) {
        self.label(value, x, y, size, ink, false);
    }
    pub fn label(
        &mut self,
        value: impl Into<String>,
        x: f32,
        y: f32,
        size: f32,
        ink: Ink,
        serif: bool,
    ) {
        self.marks.push(Mark::Text {
            value: value.into(),
            origin: (x, y),
            size,
            ink,
            serif,
        });
    }
    pub fn panel(&mut self, x: f32, y: f32, w: f32, h: f32, ink: Ink) {
        self.path(
            vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)],
            true,
            Some(ink),
            None,
        );
    }
}

pub struct Entry {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub thesis: &'static str,
}
pub const CATALOG: &[Entry] = &[
    Entry { id:"sutura", name:"Sutura", kind:"animation", thesis:"A wound in desktop space is stitched into a suspended, impossible silk surface." },
    Entry { id:"palimpsesto", name:"Palimpsesto", kind:"animation", thesis:"A scattered phrase briefly acquires a shared perspective, then loses its language." },
    Entry { id:"nacre", name:"Nacre", kind:"animation", thesis:"An excavated solid opens into disjoint contour sections and seals around an absent core." },
    Entry { id:"herbario", name:"Herbario de ausencias", kind:"interface", thesis:"An archive of suspended specimens lets one memory grow while the others recede." },
    Entry { id:"umbral", name:"Umbral de escucha", kind:"interface", thesis:"A conversational instrument separates an unspoken question from a possible response." },
];

pub fn smooth(a: f32, b: f32, t: f32) -> f32 {
    let x = ((t - a) / (b - a)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn noise(i: usize) -> f32 {
    ((i as f32 * 127.1 + 31.7).sin() * 43_758.547).fract().abs()
}
pub fn project(x: f32, y: f32, z: f32, cx: f32, cy: f32) -> Point {
    let s = 700.0 / (700.0 + z);
    (cx + x * s, cy + y * s)
}

pub fn frame(index: usize, progress: f32, reduced: bool) -> Frame {
    let t = if reduced {
        0.56
    } else {
        progress.clamp(0.0, 1.0)
    };
    let mut f = Frame::new(if reduced {
        1.0
    } else {
        smooth(0.0, 0.07, t) * (1.0 - smooth(0.88, 1.0, t))
    });
    crate::experiments::draw(index, &mut f, t, reduced);
    f.text(
        "N A V I   /   V I S U A L   G E N E S I S",
        50.0,
        35.0,
        10.0,
        DIM.alpha(0.8),
    );
    f.text(
        format!("0{}   /   {}", index + 1, CATALOG[index].id.to_uppercase()),
        50.0,
        57.0,
        11.0,
        PAPER.alpha(0.75),
    );
    f.text(
        "LOCAL STUDY   /   SIMULATED EVENT   /   PASSIVE",
        50.0,
        654.0,
        9.0,
        DIM.alpha(0.8),
    );
    f.line((50.0, 640.0), (950.0, 640.0), DIM.alpha(0.18), 0.7);
    f.line(
        (50.0, 640.0),
        (50.0 + 900.0 * t, 640.0),
        PAPER.alpha(0.6),
        1.0,
    );
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_frames_are_finite_and_bounded_in_complexity() {
        for id in 0..CATALOG.len() {
            for step in 0..=100 {
                let f = frame(id, step as f32 / 100.0, false);
                assert!(f.marks.len() < 2000);
                assert!((0.0..=1.0).contains(&f.opacity));
                for mark in f.marks {
                    match mark {
                        Mark::Path { points, .. } => {
                            for (x, y) in points {
                                assert!(
                                    x.is_finite() && y.is_finite(),
                                    "experiment {id}, step {step}: {x}, {y}"
                                );
                            }
                        }
                        Mark::Text {
                            origin: (x, y),
                            size,
                            ..
                        } => assert!(x.is_finite() && y.is_finite() && size > 0.0),
                    }
                }
            }
        }
    }
    #[test]
    fn reduced_motion_is_static() {
        for id in 0..5 {
            assert_eq!(frame(id, 0.1, true), frame(id, 0.9, true));
        }
    }
}
