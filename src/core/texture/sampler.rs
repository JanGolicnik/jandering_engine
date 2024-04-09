pub enum SamplerAddressMode {
    Repeat,
    RepeatMirrored,
    Clamp,
}

pub enum SamplerFilterMode {
    Linear,
    Nearest,
}

pub enum SamplerCompareFunction {
    Less,
    LessEqual,
    Equal,
    GreaterEqual,
    Greater,
}

pub struct SamplerDescriptor {
    pub address_mode: SamplerAddressMode,
    pub filter: SamplerFilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<SamplerCompareFunction>,
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        Self {
            address_mode: SamplerAddressMode::Clamp,
            filter: SamplerFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
        }
    }
}
