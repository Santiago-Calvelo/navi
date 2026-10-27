//! Original startup phrases and a bounded, serializable shuffle bag. No OS access.
use crate::visual::variation_unit;

pub const DURATION_MS: u64 = 6_000;
pub const PHRASES: &[&str] = &[
    "Hay señal\nentre las cosas.",
    "La habitación\ntiene otra orilla.",
    "Todo empieza\ncon una conexión.",
    "Un hilo de luz\ncruza el silencio.",
    "La pantalla\ntambién es un umbral.",
    "Algo toma forma\ndel otro lado.",
    "Entre dos pulsos\ncabe una presencia.",
    "La señal llega\nsin hacer ruido.",
    "Los cables dibujan\ncaminos invisibles.",
    "El silencio\ntiene profundidad.",
    "Una luz pequeña\ncambia la habitación.",
    "No toda distancia\nse mide en pasos.",
    "Hay otro espacio\nentre las letras.",
    "La memoria\ntiene bordes suaves.",
    "Una presencia\nempieza en un punto.",
    "El mundo continúa\nfuera del marco.",
    "Entre aquí y allá\nhay una señal.",
    "La noche deja\nlos circuitos claros.",
    "También los ecos\nencuentran una forma.",
    "Lo cotidiano\ntiene capas ocultas.",
    "Una ventana\npuede ser una orilla.",
    "El aire quieto\ntambién lleva señales.",
    "La luz del monitor\nroza el mundo.",
    "Toda conexión\nempieza en algún lugar.",
    "Un murmullo eléctrico\nal borde del día.",
    "Las palabras llegan\ndesde otra profundidad.",
    "El límite\na veces es de luz.",
    "Un punto se enciende.\nEl espacio cambia.",
    "Hay caminos\nque sólo son señal.",
    "El reflejo\nno termina en el vidrio.",
    "Una pausa\nentre dos mundos.",
    "La habitación respira\ncon otra luz.",
    "Lo visible\nes apenas una capa.",
    "El ruido se aparta.\nQueda una línea.",
    "Los hilos invisibles\ntambién hacen lugar.",
    "Una frecuencia leve\natraviesa la tarde.",
    "La memoria deja\nespacio entre sus capas.",
    "Las letras encuentran\nsu lugar en el aire.",
    "Del otro lado\ntambién empieza aquí.",
    "Hay un horizonte\ndetrás del píxel.",
    "El zumbido se vuelve\nparte del silencio.",
    "La distancia cambia\ncuando aparece un hilo.",
    "En el borde del vidrio\nempieza otra superficie.",
    "Una línea de luz\nsostiene el momento.",
    "Los circuitos\ntambién tienen orillas.",
    "La señal encuentra\nun espacio tranquilo.",
    "Entre las capas\nqueda sitio para hoy.",
    "La primera letra\nya cambia el silencio.",
];

#[derive(Default)]
pub struct PhraseRotation {
    remaining: Vec<usize>,
    last: Option<usize>,
}

impl PhraseRotation {
    pub fn decode(saved: &str) -> Option<Self> {
        let mut fields = saved.split_whitespace();
        if fields.next()? != "v1" {
            return None;
        }
        let last = match fields.next()? {
            "-" => None,
            value => Some(value.parse::<usize>().ok()?),
        };
        if last.is_some_and(|id| id >= PHRASES.len()) {
            return None;
        }
        let mut remaining = Vec::with_capacity(PHRASES.len());
        for field in fields {
            let id = field.parse::<usize>().ok()?;
            if id >= PHRASES.len() || remaining.contains(&id) {
                return None;
            }
            remaining.push(id);
        }
        Some(Self { remaining, last })
    }

    pub fn next(&mut self, seed: u64) -> &'static str {
        if self.remaining.is_empty() {
            self.remaining = (0..PHRASES.len()).collect();
            for i in (1..self.remaining.len()).rev() {
                let j = (variation_unit(seed, i as u64) * (i + 1) as f32) as usize;
                self.remaining.swap(i, j.min(i));
            }
            let end = self.remaining.len() - 1;
            if self.last == Some(self.remaining[end]) {
                self.remaining.swap(0, end);
            }
        }
        let id = self.remaining.pop().unwrap();
        self.last = Some(id);
        PHRASES[id]
    }

    pub fn encode(&self) -> String {
        let mut saved = format!(
            "v1 {}",
            self.last.map_or_else(|| "-".into(), |id| id.to_string())
        );
        for id in &self.remaining {
            saved.push_str(&format!(" {id}"));
        }
        saved
    }
}
