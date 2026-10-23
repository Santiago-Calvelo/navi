//! Renderer-independent contracts for finite desktop manifestations.
use crate::reactions::Intensity;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    NearCursor,
    #[default]
    WindowAnchor,
    SafeRandom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Family {
    #[default]
    StackV2,
    FracturedHalo,
    ClickPulse,
    DragConnector,
    Sutura,
}

#[derive(Clone, Copy, Debug)]
pub struct VisualStyle {
    pub intensity: Intensity,
    pub seed: u64,
    pub temporal: Option<TemporalMotion>,
}

impl Default for VisualStyle {
    fn default() -> Self {
        Self {
            intensity: Intensity::Experimental,
            seed: 1,
            temporal: None,
        }
    }
}

/// Normalized time curves; none changes the manifestation's lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rhythm {
    Linear,
    ArriveEarly,
    Linger,
}

impl Rhythm {
    /// Remap to [0, 1], preserving endpoints. Invalid progress starts at zero.
    pub fn remap(self, p: f32) -> f32 {
        let p = if p.is_nan() { 0.0 } else { p.clamp(0.0, 1.0) };
        match self {
            Self::Linear => p,
            Self::ArriveEarly => 1.0 - (1.0 - p) * (1.0 - p),
            Self::Linger => p * p,
        }
    }
}

/// Stable material parameters sampled once by the host for a manifestation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedVariation {
    pub seed: u64,
    pub scale: f32,
    pub opacity_gain: f32,
    pub rotation_radians: f32,
    pub duration_ms: u64,
    pub rhythm: Rhythm,
    pub temporal: TemporalMotion,
}

impl ResolvedVariation {
    /// Streams 1..=5 independently control scale, opacity, rotation, time and rhythm.
    /// Animated duration varies by up to 10%, bounded to 200..=4000 ms.
    /// Without motion the base duration is exact; the host supplies a static frame.
    pub fn sample(seed: u64, intensity: Intensity, duration_ms: u64, motion: bool) -> Self {
        let (scale_min, scale_max, opacity_min, degrees): (f32, f32, f32, f32) = match intensity {
            Intensity::Subtle => (0.96, 1.04, 0.90, 3.0),
            Intensity::Experimental => (0.92, 1.07, 0.82, 6.0),
            Intensity::Anomalous => (0.88, 1.10, 0.75, 9.0),
        };
        Self {
            seed,
            scale: scale_min + (scale_max - scale_min) * variation_unit(seed, 1),
            opacity_gain: opacity_min + (1.0 - opacity_min) * variation_unit(seed, 2),
            rotation_radians: if motion {
                (2.0 * variation_unit(seed, 3) - 1.0) * degrees.to_radians()
            } else {
                0.0
            },
            duration_ms: if motion {
                let base = duration_ms.clamp(200, 4000) as f64;
                let factor = 0.9 + 0.2 * f64::from(variation_unit(seed, 4));
                ((base * factor).round() as u64).clamp(200, 4000)
            } else {
                duration_ms
            },
            rhythm: if !motion {
                Rhythm::Linear
            } else {
                let unit = variation_unit(seed, 5);
                if unit < 1.0 / 3.0 {
                    Rhythm::Linear
                } else if unit < 2.0 / 3.0 {
                    Rhythm::ArriveEarly
                } else {
                    Rhythm::Linger
                }
            },
            temporal: TemporalMotion::sample(seed, intensity, motion),
        }
    }

    /// Unmodified material and timing for callers outside the initial integration.
    pub fn identity(seed: u64, duration_ms: u64) -> Self {
        Self {
            seed,
            scale: 1.0,
            opacity_gain: 1.0,
            rotation_radians: 0.0,
            duration_ms,
            rhythm: Rhythm::Linear,
            temporal: TemporalMotion::default(),
        }
    }

    pub fn progress(self, p: f32) -> f32 {
        self.rhythm.remap(p)
    }
}

/// Stable per-manifestation layer motion, sampled once and evaluated by progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemporalMotion {
    pub enabled: bool,
    pub layers: [LayerTemporal; 5],
    pub entry: f32,
    pub exit: f32,
}

impl Default for TemporalMotion {
    fn default() -> Self {
        Self {
            enabled: false,
            layers: [LayerTemporal::default(); 5],
            entry: 0.07,
            exit: 0.94,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayerTemporal {
    pub phase: f32,
    pub frequency: f32,
    pub amplitude: f32,
    pub construct_shift: f32,
    pub dissolve_shift: f32,
    pub direction: f32,
    pub torsion: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayerMotion {
    pub rotation_radians: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub skew: f32,
}

impl TemporalMotion {
    pub fn sample(seed: u64, intensity: Intensity, motion: bool) -> Self {
        let strength = match intensity {
            Intensity::Subtle => 0.55,
            Intensity::Experimental => 0.78,
            Intensity::Anomalous => 1.0,
        };
        let layers = std::array::from_fn(|i| {
            let stream = 16 + i as u64 * 8;
            LayerTemporal {
                phase: variation_unit(seed, stream) * std::f32::consts::TAU,
                frequency: 0.7 + variation_unit(seed, stream + 1) * 1.0,
                amplitude: strength * (0.7 + variation_unit(seed, stream + 2) * 0.3),
                construct_shift: (variation_unit(seed, stream + 3) - 0.5) * 0.06,
                dissolve_shift: (variation_unit(seed, stream + 4) - 0.5) * 0.06,
                direction: if variation_unit(seed, stream + 5) < 0.5 {
                    -1.0
                } else {
                    1.0
                },
                torsion: variation_unit(seed, stream + 6) * 2.0 - 1.0,
            }
        });
        Self {
            enabled: motion,
            layers,
            entry: 0.04 + variation_unit(seed, 8) * 0.05,
            exit: 0.91 + variation_unit(seed, 9) * 0.05,
        }
    }

    fn progress(p: f32) -> f32 {
        if p.is_nan() {
            0.0
        } else {
            p.clamp(0.0, 1.0)
        }
    }

    pub fn layer(self, index: usize, p: f32) -> LayerMotion {
        if !self.enabled {
            return LayerMotion::default();
        }
        let p = Self::progress(p);
        if p == 0.0 || p == 1.0 {
            return LayerMotion::default();
        }
        let params = self.layers[index % self.layers.len()];
        let phase = std::f32::consts::TAU * params.frequency * p + params.phase;
        let wave = phase.sin() * (std::f32::consts::PI * p).sin().powi(2) * params.amplitude;
        LayerMotion {
            rotation_radians: wave * params.torsion * 2.5_f32.to_radians(),
            offset_x: wave * params.direction * 8.0,
            offset_y: wave * params.direction * 5.0,
            skew: wave * params.torsion * 6.0,
        }
    }

    pub fn envelope(self, p: f32) -> f32 {
        let p = Self::progress(p);
        smooth_motion(p / self.entry) * (1.0 - smooth_motion((p - self.exit) / (1.0 - self.exit)))
    }
    pub fn construct(self, index: usize, p: f32) -> f32 {
        let p = Self::progress(p);
        let shift = self.layers[index % self.layers.len()].construct_shift;
        smooth_motion(p / (0.27 + shift))
    }
    pub fn dissolve(self, index: usize, p: f32) -> f32 {
        let p = Self::progress(p);
        let shift = self.layers[index % self.layers.len()].dissolve_shift;
        let start = 0.70 + shift;
        smooth_motion((p - start) / (1.0 - start))
    }
}

fn smooth_motion(value: f32) -> f32 {
    let value = if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    };
    value * value * (3.0 - 2.0 * value)
}

/// Deterministic unit sample with wrapping 64-bit mixing and no mutable RNG state.
/// The upper 24 bits map exactly into f32's precision, including both endpoints.
pub fn variation_unit(seed: u64, stream: u64) -> f32 {
    let mut value = seed.wrapping_add(stream.wrapping_mul(0x9e3779b97f4a7c15));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((value ^ (value >> 31)) >> 40) as f32 / ((1_u32 << 24) - 1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_samples_are_repeatable_bounded_and_streams_are_separate() {
        for seed in [0, 1, u64::MAX, u64::MAX - 1, 1 << 63] {
            let samples = [0, 1, 2, 3, 4, 5, u64::MAX].map(|stream| {
                let value = variation_unit(seed, stream);
                assert_eq!(value, variation_unit(seed, stream));
                assert!((0.0..=1.0).contains(&value));
                value
            });
            for (index, sample) in samples.iter().enumerate() {
                assert!(!samples[..index].contains(sample));
            }
        }
        assert_eq!(variation_unit(0, 0), 0.0);
        assert_ne!(variation_unit(1, 1), variation_unit(2, 1));
    }

    #[test]
    fn samples_respect_intensity_bounds_and_extreme_inputs() {
        for (intensity, low, high, alpha, degrees) in [
            (Intensity::Subtle, 0.96, 1.04, 0.90, 3.0_f32),
            (Intensity::Experimental, 0.92, 1.07, 0.82, 6.0),
            (Intensity::Anomalous, 0.88, 1.10, 0.75, 9.0),
        ] {
            for seed in (0..1024).chain([u64::MAX, u64::MAX - 1, 1 << 63]) {
                for duration in [0, 1, 199, 200, 850, 1800, 4000, 4001, u64::MAX] {
                    let v = ResolvedVariation::sample(seed, intensity, duration, true);
                    assert_eq!(
                        v,
                        ResolvedVariation::sample(seed, intensity, duration, true)
                    );
                    assert_eq!(v.seed, seed);
                    assert!((low..=high).contains(&v.scale));
                    assert!((alpha..=1.0).contains(&v.opacity_gain));
                    assert!(v.rotation_radians.abs() <= degrees.to_radians());
                    assert!((200..=4000).contains(&v.duration_ms));
                    let base = duration.clamp(200, 4000) as f64;
                    assert!((v.duration_ms as f64 - base).abs() <= base * 0.1 + 0.5);
                }
            }
        }
    }

    #[test]
    fn rhythms_are_monotonic_bounded_and_preserve_endpoints() {
        for rhythm in [Rhythm::Linear, Rhythm::ArriveEarly, Rhythm::Linger] {
            let mut previous = 0.0;
            for step in 0..=10000 {
                let p = step as f32 / 10000.0;
                let mapped = rhythm.remap(p);
                assert!((0.0..=1.0).contains(&mapped));
                assert!(mapped >= previous);
                previous = mapped;
                let v = ResolvedVariation {
                    rhythm,
                    ..ResolvedVariation::identity(0, 850)
                };
                assert_eq!(v.progress(p), mapped);
            }
            assert_eq!(rhythm.remap(0.0), 0.0);
            assert_eq!(rhythm.remap(1.0), 1.0);
            assert_eq!(rhythm.remap(f32::NEG_INFINITY), 0.0);
            assert_eq!(rhythm.remap(f32::INFINITY), 1.0);
            assert_eq!(rhythm.remap(f32::NAN), 0.0);
        }
        assert!(Rhythm::ArriveEarly.remap(0.5) > 0.5);
        assert!(Rhythm::Linger.remap(0.5) < 0.5);
    }

    #[test]
    fn reduced_motion_keeps_base_time_and_static_material_samples() {
        for intensity in [
            Intensity::Subtle,
            Intensity::Experimental,
            Intensity::Anomalous,
        ] {
            for seed in [0, 1, 42, u64::MAX] {
                for duration in [0, 200, 850, 4000, u64::MAX] {
                    let still = ResolvedVariation::sample(seed, intensity, duration, false);
                    let moving = ResolvedVariation::sample(seed, intensity, duration, true);
                    assert_eq!(still.rotation_radians, 0.0);
                    assert_eq!(still.rhythm, Rhythm::Linear);
                    assert_eq!(still.duration_ms, duration);
                    assert_eq!(still.scale, moving.scale);
                    assert_eq!(still.opacity_gain, moving.opacity_gain);
                    assert_eq!(still.progress(0.5), 0.5);
                }
            }
        }
    }

    #[test]
    fn identity_preserves_seed_duration_and_neutral_parameters() {
        for duration in [0, 850, u64::MAX] {
            let v = ResolvedVariation::identity(u64::MAX, duration);
            assert_eq!(v.seed, u64::MAX);
            assert_eq!(v.duration_ms, duration);
            assert_eq!(v.scale, 1.0);
            assert_eq!(v.opacity_gain, 1.0);
            assert_eq!(v.rotation_radians, 0.0);
            assert_eq!(v.rhythm, Rhythm::Linear);
        }
    }
}
