use crate::types::UVec2;

pub mod sampler;

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub enum TextureFormat {
    Rgba8U,
    Bgra8U,
    F32,
    Depth32F,
    Depth16U,
}

pub mod texture_usage {
    pub type TextureUsage = u32;

    pub const ALL: TextureUsage = COPY_SRC | COPY_TARGET | BIND | TARGET;
    pub const NONE: TextureUsage = 0;

    pub const COPY_SRC: TextureUsage = 1 << 0;
    pub const COPY_TARGET: TextureUsage = 1 << 1;
    pub const BIND: TextureUsage = 1 << 2;
    pub const TARGET: TextureUsage = 1 << 3;
}

#[derive(Clone)]
pub struct TextureDescriptor<'data> {
    pub name: &'static str,
    pub size: UVec2,
    pub sample_count: u32,
    pub data: Option<&'data [u8]>,
    pub format: TextureFormat,
    pub usage: texture_usage::TextureUsage,
}

impl<'data> Default for TextureDescriptor<'data> {
    fn default() -> Self {
        Self {
            name: "texture",
            size: UVec2::new(8, 8),
            sample_count: 1,
            format: TextureFormat::Bgra8U,
            data: None,
            usage: texture_usage::BIND | texture_usage::COPY_TARGET,
        }
    }
}
