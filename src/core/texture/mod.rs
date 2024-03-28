pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
}

pub struct TextureDescriptor {
    pub is_normal_map: bool,
    pub address_mode: wgpu::AddressMode,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        Self {
            is_normal_map: false,
            address_mode: wgpu::AddressMode::ClampToEdge,
        }
    }
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
