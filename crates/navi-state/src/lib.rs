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
    KeyboardInvoke(u64),
    KeyboardTick(u64),
    KeyboardCancel,
    Select(Mode),
    SetPhase(Phase),
    Character(char),
    Backspace,
    Submit,
    BeginRequest,
    CompleteRequest {
        text: String,
        confirmation: bool,
        failed: bool,
    },
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
    pub answer: String,
    pub executing: bool,
    pub awaiting_confirmation: bool,
    pub keyboard_entry: Option<KeyboardEntry>,
}

/// Text readiness only; it never starts microphone or waveform activity.
#[derive(Clone, Copy, Debug)]
pub struct KeyboardEntry {
    pub prompt_until: u64,
    pub now: u64,
}

impl KeyboardEntry {
    pub fn prompting(self) -> bool {
        self.now < self.prompt_until
    }

    pub fn caret_visible(self) -> bool {
        !self.prompting() && ((self.now - self.prompt_until) / 500).is_multiple_of(2)
    }

    pub fn next_tick(self) -> u32 {
        if self.prompting() {
            (self.prompt_until - self.now).min(u32::MAX as u64) as u32
        } else {
            (500 - (self.now - self.prompt_until) % 500) as u32
        }
    }
}

impl Session {
    pub fn apply(&mut self, event: Event) {
        match event {
            Event::KeyboardInvoke(now) => {
                self.mode = Mode::Text;
                if !self.executing && !self.awaiting_confirmation {
                    self.phase = Phase::Invoked;
                    self.answer.clear();
                    self.keyboard_entry = Some(KeyboardEntry {
                        prompt_until: now.saturating_add(2_000),
                        now,
                    });
                }
            }
            Event::KeyboardTick(now) => {
                if let Some(entry) = &mut self.keyboard_entry {
                    entry.now = now;
                }
            }
            Event::KeyboardCancel => self.keyboard_entry = None,
            Event::Select(mode) => {
                self.mode = mode;
                self.phase = Phase::Invoked;
            }
            Event::SetPhase(phase) => {
                self.keyboard_entry = None;
                self.phase = phase;
                self.frame = 0;
                if !self.executing {
                    self.answer.clear();
                    self.awaiting_confirmation = false;
                }
            }
            Event::Character(c) if !c.is_control() && self.input.chars().count() < 120 => {
                self.phase = Phase::Invoked;
                self.mode = Mode::Text;
                self.input.push(c);
                if let Some(entry) = &mut self.keyboard_entry {
                    entry.prompt_until = entry.now;
                }
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
            Event::BeginRequest if !self.executing && !self.input.trim().is_empty() => {
                self.keyboard_entry = None;
                self.submitted = std::mem::take(&mut self.input);
                self.answer.clear();
                self.awaiting_confirmation = false;
                self.executing = true;
                self.phase = Phase::Processing;
                self.frame = 0;
            }
            Event::CompleteRequest {
                text,
                confirmation,
                failed,
            } => {
                self.keyboard_entry = None;
                self.answer = text.chars().take(240).collect();
                self.executing = false;
                self.awaiting_confirmation = confirmation;
                self.phase = if !self.input.is_empty() {
                    Phase::Invoked
                } else if failed {
                    Phase::Error
                } else {
                    Phase::Responding
                };
                self.frame = 0;
            }
            Event::Tick => {
                self.frame += 1;
                if self.phase == Phase::Processing && self.frame >= 24 && !self.executing {
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
        self.executing || matches!(self.phase, Phase::Listening | Phase::Processing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyboard_prompt_becomes_blinking_text_readiness_without_listening() {
        let mut s = Session::default();
        s.apply(Event::KeyboardInvoke(100));
        assert_eq!(s.phase, Phase::Invoked);
        assert_eq!(s.mode, Mode::Text);
        assert!(!s.animated());
        assert!(s.keyboard_entry.unwrap().prompting());
        assert_eq!(s.keyboard_entry.unwrap().next_tick(), 2_000);
        s.apply(Event::KeyboardTick(2_100));
        assert!(s.keyboard_entry.unwrap().caret_visible());
        s.apply(Event::KeyboardTick(2_600));
        assert!(!s.keyboard_entry.unwrap().caret_visible());
        s.apply(Event::Character('ñ'));
        s.apply(Event::KeyboardCancel);
        assert_eq!(s.input, "ñ");
        assert!(s.keyboard_entry.is_none());
    }
    #[test]
    fn keyboard_invocation_preserves_draft_and_active_or_confirmation_results() {
        let mut s = Session::default();
        s.apply(Event::Character('a'));
        s.apply(Event::KeyboardInvoke(0));
        assert_eq!(s.input, "a");
        s.apply(Event::BeginRequest);
        assert!(s.keyboard_entry.is_none());
        s.apply(Event::KeyboardInvoke(100));
        assert_eq!(s.phase, Phase::Processing);
        assert!(s.keyboard_entry.is_none());
        s.apply(Event::CompleteRequest {
            text: "Confirmación pendiente".into(),
            confirmation: true,
            failed: false,
        });
        s.apply(Event::KeyboardInvoke(200));
        assert_eq!(s.answer, "Confirmación pendiente");
        assert!(s.awaiting_confirmation);
        assert!(s.keyboard_entry.is_none());
        s.apply(Event::Reset);
        assert!(s.keyboard_entry.is_none());
    }
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
    #[test]
    fn real_execution_cannot_be_finished_by_demo_ticks_and_preserves_a_new_draft() {
        let mut s = Session::default();
        s.apply(Event::Character('a'));
        s.apply(Event::BeginRequest);
        for _ in 0..30 {
            s.apply(Event::Tick);
        }
        assert!(s.executing);
        assert_eq!(s.phase, Phase::Processing);
        s.apply(Event::Character('b'));
        s.apply(Event::CompleteRequest {
            text: "Respuesta".into(),
            confirmation: false,
            failed: false,
        });
        assert_eq!(s.input, "b");
        assert_eq!(s.answer, "Respuesta");
        assert_eq!(s.phase, Phase::Invoked);
        assert!(!s.executing);
    }
    #[test]
    fn pending_confirmation_is_not_a_running_animation() {
        let mut s = Session::default();
        s.apply(Event::CompleteRequest {
            text: "confirmar 1".into(),
            confirmation: true,
            failed: false,
        });
        assert_eq!(s.phase, Phase::Responding);
        assert!(s.awaiting_confirmation);
        assert!(!s.animated());
    }
}
pub mod action_visual;
pub mod cursor;
pub mod holograms;
pub mod reactions;
pub mod visual;
