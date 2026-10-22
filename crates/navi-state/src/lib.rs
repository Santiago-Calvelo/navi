#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    Invoked,
    Listening,
    Processing,
    Responding,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Voice,
    Text,
}

pub enum Event {
    Select(Mode),
    SetPhase(Phase),
    Character(char),
    Backspace,
    Submit,
    Tick,
    TogglePin,
    Reset,
}

#[derive(Default)]
pub struct Session {
    pub phase: Phase,
    pub mode: Mode,
    pub input: String,
    pub pinned: bool,
    pub frame: u32,
    pub submitted: String,
}

impl Session {
    pub fn apply(&mut self, event: Event) {
        match event {
            Event::Select(mode) => {
                self.mode = mode;
                self.phase = Phase::Invoked;
            }
            Event::SetPhase(phase) => {
                self.phase = phase;
                self.frame = 0;
            }
            Event::Character(c) if !c.is_control() && self.input.chars().count() < 120 => {
                self.phase = Phase::Invoked;
                self.mode = Mode::Text;
                self.input.push(c);
            }
            Event::Backspace => {
                self.input.pop();
            }
            Event::Submit => {
                if self.phase == Phase::Listening || !self.input.trim().is_empty() {
                    self.submitted = if self.input.trim().is_empty() {
                        "Hola, NAVI".into()
                    } else {
                        self.input.clone()
                    };
                    self.input.clear();
                    self.phase = Phase::Processing;
                    self.frame = 0;
                }
            }
            Event::Tick => {
                self.frame += 1;
                if self.phase == Phase::Processing && self.frame >= 24 {
                    self.phase = Phase::Responding;
                }
            }
            Event::TogglePin => self.pinned = !self.pinned,
            Event::Reset => {
                let pinned = self.pinned;
                *self = Self::default();
                self.pinned = pinned;
            }
            _ => {}
        }
    }

    pub fn animated(&self) -> bool {
        matches!(self.phase, Phase::Listening | Phase::Processing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simulated_submission_completes_without_an_external_action() {
        let mut s = Session::default();
        s.apply(Event::Character('á'));
        s.apply(Event::Submit);
        assert_eq!(s.submitted, "á");
        assert_eq!(s.phase, Phase::Processing);
        for _ in 0..24 {
            s.apply(Event::Tick);
        }
        assert_eq!(s.phase, Phase::Responding);
        assert!(!s.animated());
    }
    #[test]
    fn editing_and_reset_preserve_pin_and_bound_input() {
        let mut s = Session::default();
        s.apply(Event::TogglePin);
        for _ in 0..140 {
            s.apply(Event::Character('ñ'));
        }
        assert_eq!(s.input.chars().count(), 120);
        s.apply(Event::Backspace);
        assert_eq!(s.input.chars().count(), 119);
        s.apply(Event::Reset);
        assert!(s.pinned);
        assert!(s.input.is_empty());
    }
}
pub mod cursor;
pub mod holograms;
pub mod reactions;
pub mod visual;
