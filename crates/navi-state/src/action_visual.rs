//! One finite action manifestation, independent of desktop focus and tool latency.
use crate::{
    holograms::{EffectFrame, STACK},
    Phase,
};

pub const DURATION_MS: u64 = 1800;

pub struct ActionVisual {
    pub enabled: bool,
    pub seed: u64,
    playback: Option<(u64, bool)>,
}

impl Default for ActionVisual {
    fn default() -> Self {
        Self {
            enabled: true,
            seed: 0,
            playback: None,
        }
    }
}

impl ActionVisual {
    pub fn start(&mut self, now: u64, failed: bool) {
        self.seed = self.seed.wrapping_add(0x9e3779b97f4a7c15);
        self.playback = self.enabled.then_some((now, failed));
    }
    pub fn clear(&mut self) {
        self.playback = None;
    }
    pub fn frame(&mut self, now: u64, motion: bool) -> Option<(EffectFrame, u32)> {
        let (start, failed) = self.playback?;
        let elapsed = now.saturating_sub(start);
        if !self.enabled || elapsed >= DURATION_MS {
            self.clear();
            return None;
        }
        let progress = elapsed as f32 / DURATION_MS as f32;
        Some((
            EffectFrame {
                layers: 4,
                mask: STACK,
                phase: if failed {
                    Phase::Error
                } else {
                    Phase::Responding
                },
                progress,
                opacity: if motion {
                    (progress * 9.0).min(1.0) * ((1.0 - progress) * 5.0).min(1.0)
                } else {
                    1.0
                },
                motion,
            },
            (DURATION_MS - elapsed) as u32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn action_has_finite_lifetime_and_can_be_disabled_or_cleared() {
        let mut v = ActionVisual::default();
        assert!(v.frame(0, true).is_none());
        v.start(10, false);
        assert!(v.frame(700, true).unwrap().0.opacity > 0.5);
        assert!(v.frame(1810, true).is_none());
        assert!(v.frame(2000, true).is_none());
        v.start(3000, true);
        assert_eq!(v.frame(3100, false).unwrap().0.phase, Phase::Error);
        v.enabled = false;
        assert!(v.frame(3200, true).is_none());
        v.enabled = true;
        v.start(4000, false);
        v.clear();
        assert!(v.frame(4100, true).is_none());
    }
}
