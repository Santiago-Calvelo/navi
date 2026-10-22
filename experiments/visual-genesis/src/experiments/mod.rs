mod herbario;
mod nacre;
mod palimpsesto;
mod sutura;
mod umbral;

use crate::experiment::Frame;
pub fn draw(id: usize, f: &mut Frame, t: f32, reduced: bool) {
    match id {
        0 => sutura::draw(f, t),
        1 => palimpsesto::draw(f, t),
        2 => nacre::draw(f, t),
        3 => herbario::draw(f, t, reduced),
        _ => umbral::draw(f, t, reduced),
    }
}
