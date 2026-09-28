use crate::math::Real;

#[derive(Debug, Clone, Copy)]
pub struct RenderSettings {
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub seed: u64,
    pub clamp_indirect: Real,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 64,
            max_depth: 12,
            seed: 42,
            // Should be 0.0 for high samples counts
            clamp_indirect: 10.0,
        }
    }
}
