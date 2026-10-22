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
    StackV1,
    #[default]
    StackV2,
    FracturedHalo,
    ClickPulse,
    DragConnector,
}

#[derive(Clone, Copy, Debug)]
pub struct VisualStyle {
    pub intensity: Intensity,
    pub seed: u64,
}

impl Default for VisualStyle {
    fn default() -> Self {
        Self {
            intensity: Intensity::Experimental,
            seed: 1,
        }
    }
}
