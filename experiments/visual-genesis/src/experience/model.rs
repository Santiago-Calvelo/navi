//! Local screenplay and editor. No operating-system actions or provider.
use std::ops::Range;

pub const INPUT_LIMIT: usize = 4096;
pub const COLS: usize = 62;
pub const LINE_HEIGHT: f32 = 25.0;
pub const MESSAGE_GAP: f32 = 56.0;

#[derive(Default, Debug)]
pub struct Editor {
    pub chars: Vec<char>,
    pub caret: usize,
    pub anchor: usize,
}
impl Editor {
    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }
    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.caret)..self.anchor.max(self.caret)
    }
    pub fn insert(&mut self, text: &str) {
        let range = self.selection();
        let count = INPUT_LIMIT - (self.chars.len() - range.len());
        let chars: Vec<_> = text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .take(count)
            .collect();
        self.caret = range.start + chars.len();
        self.anchor = self.caret;
        self.chars.splice(range, chars);
    }
    pub fn erase(&mut self, back: bool) {
        let mut r = self.selection();
        if r.is_empty() {
            if back {
                r.start = r.start.saturating_sub(1);
            } else {
                r.end = (r.end + 1).min(self.chars.len());
            }
        }
        self.caret = r.start;
        self.anchor = self.caret;
        self.chars.drain(r);
    }
    pub fn move_to(&mut self, pos: usize, select: bool) {
        self.caret = pos.min(self.chars.len());
        if !select {
            self.anchor = self.caret;
        }
    }
    pub fn clear(&mut self) {
        self.chars.clear();
        self.caret = 0;
        self.anchor = 0;
    }
    pub fn set(&mut self, text: &str) {
        self.clear();
        self.insert(text);
    }
    pub fn selected(&self) -> String {
        self.chars[self.selection()].iter().collect()
    }
}

/// Scalar ranges preserve cursor positions, blank lines and explicit line breaks.
pub fn lines(chars: &[char], cols: usize) -> Vec<Range<usize>> {
    let mut result = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let mut end = start;
        while end < chars.len() && end - start < cols && chars[end] != '\n' {
            end += 1;
        }
        if end < chars.len() && chars[end] != '\n' {
            if let Some(space) = (start..end).rev().find(|&i| chars[i] == ' ') {
                if space > start {
                    end = space + 1;
                }
            }
        }
        result.push(start..end);
        start = if chars.get(end) == Some(&'\n') {
            end + 1
        } else {
            end
        };
    }
    if result.is_empty() || chars.last() == Some(&'\n') {
        result.push(chars.len()..chars.len());
    }
    result
}
pub fn caret_line(rows: &[Range<usize>], caret: usize) -> usize {
    rows.iter().rposition(|r| r.start <= caret).unwrap_or(0)
}

#[derive(Clone)]
pub struct Message {
    pub user: bool,
    pub text: String,
    pub note: bool,
}
pub fn message_height(message: &Message) -> f32 {
    lines(&message.text.chars().collect::<Vec<_>>(), COLS).len() as f32 * LINE_HEIGHT + MESSAGE_GAP
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Quiet,
    Thinking,
    Permission,
}
pub struct Model {
    pub editor: Editor,
    pub messages: Vec<Message>,
    pub phase: Phase,
    pub due: f32,
    pending: String,
    pub scroll: f32,
    pub unread: bool,
    pub notice: String,
    pub reveal: f32,
}
impl Model {
    pub fn new() -> Self {
        Self {editor:Editor::default(),messages:vec![Message{user:false,text:"Estoy acá.\nPodés dejar una pregunta, una idea a medio hacer o simplemente seguir el hilo.".into(),note:false}],phase:Phase::Quiet,due:0.0,pending:String::new(),scroll:0.0,unread:false,notice:String::new(),reveal:0.0}
    }
    pub fn append(&mut self, user: bool, text: impl Into<String>, note: bool) {
        let message = Message {
            user,
            text: text.into(),
            note,
        };
        if self.scroll > 1.0 {
            self.unread = true;
            // Scroll is distance from the tail: offset new content to keep the
            // same old paragraph under the reader's eyes.
            self.scroll += message_height(&message);
        }
        self.messages.push(message);
    }
    pub fn submit(&mut self, now: f32) {
        let text = self.editor.text().trim().to_string();
        if text.is_empty() {
            return;
        }
        if self.phase != Phase::Quiet {
            self.notice = "Tu borrador está a salvo. Terminá o cancelá el paso pendiente.".into();
            return;
        }
        self.append(true, &text, false);
        self.editor.clear();
        self.scroll = 0.0;
        self.unread = false;
        self.pending = text;
        self.phase = Phase::Thinking;
        self.due = now + 2.2;
        self.notice.clear();
    }
    pub fn tick(&mut self, now: f32) -> bool {
        if self.phase != Phase::Thinking || now < self.due {
            return false;
        }
        let input = self.pending.to_lowercase();
        self.pending.clear();
        if ["orden", "guardar", "organiz", "confirm", "cerrar"]
            .iter()
            .any(|s| input.contains(s))
        {
            self.phase = Phase::Permission;
            self.append(false,"Puedo reunir este hilo en una memoria y dejar las ideas abiertas a un lado. Antes de hacerlo, necesito tu decisión.",false);
            self.notice =
                "Sólo cambia la escena de demostración. No toca archivos ni ventanas.".into();
        } else {
            self.phase = Phase::Quiet;
            let response = if input.contains("historia")
                || input.contains("largo")
                || input.contains("leer")
            {
                "Podemos leer el escritorio como un lugar habitado. Una ventana abierta no es sólo una tarea: también puede ser una idea que todavía no encontró dónde quedarse.\n\nLa presencia no necesita ocupar el centro. Puede permanecer en el borde, sostener una continuidad y aparecer cuando la buscás. El silencio también forma parte del intercambio.\n\nAl escribir, el espacio se vuelve más preciso. Tus palabras quedan en un mismo recorrido: podés volver, leer lo anterior y continuar sin perder la posición. No hace falta resolver todo en una respuesta.\n\nCuando una decisión cambia algo, la conversación se detiene justo ahí. El permiso pertenece a esa decisión concreta; no es una autorización para hacer cualquier otra cosa.\n\nDespués, la superficie puede retirarse. La conversación y el borrador permanecen durante esta sesión. La computadora vuelve a ser el lugar donde estabas trabajando.\n\nEsto es un recorrido escrito para el prototipo. Si querés probar el siguiente estado, escribí: guardar este hilo."
            } else if input.contains("hola") {
                "Te leo. Este borde es un lugar para retomar lo que estabas pensando.\n\nPodés escribir «quiero leer algo largo» para explorar el recorrido, o «guardar este hilo» para probar una decisión contextual."
            } else {
                match self.messages.len()%3 {
                    0=>"Queda en este hilo. Podemos conservar la pregunta abierta y mirar una parte por vez.\n\n¿Qué sería lo primero que te gustaría ordenar?",
                    1=>"Podemos seguir desde acá. Lo anterior permanece en el recorrido; podés desplazarte hacia arriba sin perder lo que estás escribiendo.\n\nEsta respuesta está guionada para ensayar la interfaz.",
                    _=>"No hace falta terminar la idea ahora. Puedo quedarme en el margen mientras seguís trabajando.\n\nAl volver, el borrador y esta conversación van a seguir acá durante la sesión.",
                }
            };
            self.append(false, response, false);
            self.reveal = now;
        }
        true
    }
    pub fn decide(&mut self, allow: bool, now: f32) {
        if self.phase != Phase::Permission {
            return;
        }
        self.phase = Phase::Quiet;
        self.notice.clear();
        self.append(false,if allow{"MEMORIA 01 / HILO REUNIDO\nLa escena quedó marcada. Tus palabras siguen acá; no se guardó ningún archivo ni se modificó el escritorio."}else{"La decisión quedó sin efecto. Podemos seguir hablando; no cambió nada fuera de este prototipo."},true);
        self.reveal = now;
        self.scroll = 0.0;
        self.unread = false;
    }
    pub fn cancel(&mut self, now: f32) {
        if self.phase == Phase::Permission {
            self.decide(false, now);
        } else if self.phase == Phase::Thinking {
            self.phase = Phase::Quiet;
            self.pending.clear();
            self.append(
                false,
                "Interrumpido. Tu nuevo borrador permanece intacto.",
                true,
            );
        }
    }
    pub fn history(&mut self) {
        if self.messages.len() > 12 {
            return;
        }
        for (question,answer) in [
            ("¿Dónde empieza una conversación?","En el momento en que algo merece permanecer. No hace falta abrir un espacio aparte: podemos escribir en el margen de lo que ya está pasando."),
            ("¿Y si todavía no sé qué preguntar?","Una frase incompleta también sirve. El borrador es tuyo: podés retirarte al borde, cambiar de ventana y volver más tarde."),
            ("Quiero poder volver atrás.","La rueda recorre el hilo. Las pequeñas marcas del margen señalan cada intervención. Al recibir algo nuevo mientras leés arriba, aparece una indicación para volver al presente."),
            ("¿Qué pasa si te pido cambiar algo?","La intención aparece junto a su contexto. Podés continuar o dejarla sin efecto. En esta demostración no se ejecutan acciones reales."),
            ("Dejemos algo abierto.","Podemos dejarlo así. Una presencia no tiene que llenar cada silencio. El hilo no se cierra cuando retirás la superficie."),
        ] {self.append(true,question,false);self.append(false,answer,false);}
        self.scroll = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draft_survives_response_and_confirmation() {
        let mut m = Model::new();
        m.editor.insert("guardar este hilo");
        m.submit(1.0);
        m.editor.insert("mañana seguimos");
        m.tick(4.0);
        assert_eq!(m.phase, Phase::Permission);
        m.decide(true, 5.0);
        assert_eq!(m.editor.text(), "mañana seguimos");
        assert_eq!(m.phase, Phase::Quiet);
        let count = m.messages.len();
        m.decide(true, 6.0);
        assert_eq!(m.messages.len(), count);
    }
    #[test]
    fn unicode_selection_and_wrapping() {
        let mut e = Editor::default();
        e.insert("á🌒bc");
        e.move_to(1, false);
        e.move_to(3, true);
        e.insert("ñ");
        assert_eq!(e.text(), "áñc");
        e.erase(true);
        assert_eq!(e.text(), "ác");
        let chars: Vec<_> = "uno dos\n\nfin\n".chars().collect();
        let rows = lines(&chars, 5);
        assert_eq!(rows.last().unwrap(), &(chars.len()..chars.len()));
        assert!(rows.iter().all(|r| r.len() <= 5));
    }
    #[test]
    fn canceled_work_never_completes() {
        let mut m = Model::new();
        m.editor.insert("hola");
        m.submit(0.0);
        m.cancel(1.0);
        let n = m.messages.len();
        assert!(!m.tick(8.0));
        assert_eq!(m.messages.len(), n);
    }
    #[test]
    fn incoming_response_does_not_move_the_reading_anchor() {
        let mut m = Model::new();
        m.history();
        m.scroll = 100.0;
        let before = m.messages.iter().map(message_height).sum::<f32>() - m.scroll;
        m.append(
            false,
            "Una respuesta que llega mientras leés arriba.",
            false,
        );
        let after = m.messages.iter().map(message_height).sum::<f32>() - m.scroll;
        assert_eq!(before, after);
        assert!(m.unread);
    }
}
