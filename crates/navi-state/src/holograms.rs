use crate::Phase;

pub const SIGNAL: u8 = 1;
pub const CONTEXT: u8 = 2;
pub const STACK: u8 = 4;
pub const ALL: u8 = SIGNAL | CONTEXT | STACK;

#[derive(Clone, Copy, Debug)]
pub struct EffectFrame {
    pub layers: u8,
    pub mask: u8,
    pub phase: Phase,
    pub progress: f32,
    pub opacity: f32,
    pub motion: bool,
}

pub struct Holograms {
    pub layers: u8,
    pub enabled: u8,
    pub duration_ms: u64,
    pub motion: bool,
    playback: Option<(u64, Phase, u8)>,
    playback_duration: u64,
}

impl Default for Holograms {
    fn default() -> Self {
        Self {
            layers: 3,
            enabled: 0,
            duration_ms: 1800,
            motion: true,
            playback: None,
            playback_duration: 1800,
        }
    }
}

impl Holograms {
    pub fn preview(&mut self, mask: u8, phase: Phase, now: u64) {
        self.playback_duration = self.duration_ms.clamp(200, 4000);
        self.playback = Some((now, phase, mask));
    }
    pub fn preview_timed(&mut self, mask: u8, phase: Phase, now: u64, duration: u64) {
        self.preview(mask, phase, now);
        self.playback_duration = duration.clamp(200, 10_000);
    }
    pub fn phase_changed(&mut self, phase: Phase, now: u64) {
        if phase == Phase::Idle || self.enabled == 0 {
            self.clear();
        } else {
            self.preview(self.enabled, phase, now);
        }
    }
    pub fn clear(&mut self) {
        self.playback = None;
    }
    pub fn frame(&mut self, now: u64) -> Option<EffectFrame> {
        let (start, phase, mask) = self.playback?;
        let duration = self.playback_duration;
        let elapsed = now.saturating_sub(start);
        if elapsed >= duration || mask == 0 {
            self.clear();
            return None;
        }
        let progress = elapsed as f32 / duration as f32;
        let opacity = if self.motion && mask == CONTEXT {
            (elapsed as f32 / 200.0).min(1.0) * ((duration - elapsed) as f32 / 200.0).min(1.0)
        } else if self.motion {
            (progress * 8.0).min(1.0) * ((1.0 - progress) * 6.0).min(1.0)
        } else {
            1.0
        };
        Some(EffectFrame {
            layers: self.layers.clamp(2, 4),
            mask,
            phase,
            progress,
            opacity,
            motion: self.motion,
        })
    }
    pub fn remaining(&self, now: u64) -> u32 {
        self.playback.map_or(0, |(start, _, _)| {
            (start + self.playback_duration).saturating_sub(now).max(1) as u32
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn playback_is_bounded_and_idle_never_starts_it() {
        let mut effects = Holograms {
            enabled: SIGNAL,
            ..Default::default()
        };
        effects.phase_changed(Phase::Listening, 100);
        assert!(effects.frame(800).is_some());
        assert!(effects.frame(1900).is_none());
        effects.phase_changed(Phase::Idle, 2000);
        assert!(effects.frame(2000).is_none());
        effects.enabled = 0;
        effects.phase_changed(Phase::Processing, 2100);
        assert!(effects.frame(2100).is_none());
    }
}
