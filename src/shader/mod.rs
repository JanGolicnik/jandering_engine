use crate::{bind_group::BindGroupLayoutDescriptor, texture::TextureFormat, utils::FilePath};

#[derive(Clone)]
pub enum ShaderSource {
    Code(String),
    #[cfg(not(target_arch = "wasm32"))]
    File(FilePath),
}

#[derive(Clone)]
pub enum BufferLayoutStepMode {
    Vertex,
    Instance,
}

#[derive(Clone, Copy)]
pub enum BufferLayoutEntryDataType {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,

    U32,
}

#[derive(Clone, Copy)]
pub struct BufferLayoutEntry {
    pub location: u32,
    pub data_type: BufferLayoutEntryDataType,
}

impl BufferLayoutEntryDataType {
    pub fn size_bytes(&self) -> u64 {
        match self {
            BufferLayoutEntryDataType::Float32 | BufferLayoutEntryDataType::U32 => 4,
            BufferLayoutEntryDataType::Float32x2 => 8,
            BufferLayoutEntryDataType::Float32x3 => 12,
            BufferLayoutEntryDataType::Float32x4 => 16,
        }
    }
}

#[derive(Clone)]
pub struct BufferLayout {
    pub step_mode: BufferLayoutStepMode,
    pub entries: &'static [BufferLayoutEntry],
}

#[derive(Clone)]
pub struct ShaderDescriptor {
    pub name: &'static str,
    pub source: ShaderSource,
    pub descriptors: Vec<BufferLayout>,
    pub bind_group_layout_descriptors: Vec<BindGroupLayoutDescriptor>,
    pub vs_entry: &'static str,
    pub fs_entry: &'static str,
    pub backface_culling: bool,
    pub depth: bool,
    pub stripped: bool,
    pub multisample: u32,
    pub target_texture_format: Option<TextureFormat>,
}

#[derive(Clone)]
pub struct ComputeShaderDescriptor {
    pub source: ShaderSource,
    pub bind_group_layout_descriptors: Vec<BindGroupLayoutDescriptor>,
    pub entry: &'static str,
}

impl Default for ShaderDescriptor {
    fn default() -> Self {
        Self {
            name: "Unnamed Shader",
            source: ShaderSource::Code(include_str!("default_shader.wgsl").to_string()),
            descriptors: Vec::new(),
            bind_group_layout_descriptors: Vec::new(),
            vs_entry: "vs_main",
            fs_entry: "fs_main",
            backface_culling: true,
            depth: false,
            stripped: false,
            multisample: 1,
            target_texture_format: Some(TextureFormat::Bgra8U),
        }
    }
}

impl ShaderDescriptor {
    pub fn flat() -> Self {
        Self {
            source: ShaderSource::Code(include_str!("flat_shader.wgsl").to_string()),
            ..Default::default()
        }
    }
}
