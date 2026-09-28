#[derive(Debug, Clone, Copy)]
pub struct RenderSettings {
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub seed: u64,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 64,
            max_depth: 12,
            seed: 42,
        }
    }
}
