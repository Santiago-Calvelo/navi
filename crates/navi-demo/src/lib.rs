use navi_state::Phase;

pub struct ContextSample {
    pub application: &'static str,
    pub file: &'static str,
    pub action: &'static str,
}
impl ContextSample {
    pub fn for_phase(phase: Phase) -> Self {
        Self {
            application: "Visual Studio Code",
            file: "NAVI / README.md",
            action: match phase {
                Phase::Processing => "vinculando contexto…",
                Phase::Responding => "lectura preparada",
                Phase::Error => "contexto no disponible",
                _ => "en espera de una instrucción",
            },
        }
    }
}

pub fn phase_for_key(key: usize) -> Option<Phase> {
    match key {
        0x70 => Some(Phase::Invoked),
        0x71 => Some(Phase::Listening),
        0x72 => Some(Phase::Processing),
        0x73 => Some(Phase::Responding),
        0x74 => Some(Phase::Error),
        0x76 => Some(Phase::Idle),
        _ => None,
    }
}

pub fn next_phase(phase: Phase) -> Phase {
    match phase {
        Phase::Idle => Phase::Invoked,
        Phase::Invoked => Phase::Listening,
        Phase::Listening => Phase::Processing,
        Phase::Processing => Phase::Responding,
        Phase::Responding => Phase::Error,
        Phase::Error => Phase::Idle,
    }
}

// Deterministic signal, never reads the microphone.
pub fn amplitude(index: u32, frame: u32, active: bool) -> f32 {
    let x = index as f32;
    let envelope =
        (-((x - 38.0) / 13.0).powi(2)).exp() + 0.45 * (-((x - 91.0) / 8.0).powi(2)).exp();
    let wave = (x * 1.71 + frame as f32 * 0.21).sin().abs();
    0.8 + envelope * wave * if active { 31.0 } else { 17.0 }
}
