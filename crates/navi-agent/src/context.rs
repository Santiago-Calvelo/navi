use crate::model::{Application, MAX_MESSAGE_CHARS};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Turn {
    pub user: String,
    pub assistant: String,
}

#[derive(Default)]
pub struct Context {
    turns: VecDeque<Turn>,
    pub(crate) application: Option<Application>,
}

impl Context {
    pub fn turns(&self) -> impl Iterator<Item = &Turn> {
        self.turns.iter()
    }
    pub(crate) fn record(&mut self, user: &str, assistant: &str) {
        if self.turns.len() == 8 {
            self.turns.pop_front();
        }
        self.turns.push_back(Turn {
            user: user.chars().take(MAX_MESSAGE_CHARS).collect(),
            assistant: assistant.chars().take(MAX_MESSAGE_CHARS).collect(),
        });
    }
}
