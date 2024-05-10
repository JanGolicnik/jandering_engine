use super::bind_group::BindGroupLayout;

#[derive(Clone)]
pub enum ShaderSource {
    Code(String),
}

#[derive(Clone)]
pub enum BufferLayoutStepMode {
    Vertex,
    Instance,
}

#[derive(Clone)]
pub struct BufferLayoutEntry {
    pub size: u64,
}

#[derive(Clone)]
pub struct BufferLayout {
    pub step_mode: BufferLayoutStepMode,
    pub entries: &'static [BufferLayoutEntry],
}

#[derive(Clone)]
pub struct ShaderDescriptor {
    pub source: ShaderSource,
    pub descriptors: Vec<BufferLayout>, //TODO: abstract this away so there is no dependency on wgpu
    pub bind_group_layouts: Vec<BindGroupLayout>,
    pub vs_entry: &'static str,
    pub fs_entry: &'static str,
    pub backface_culling: bool,
    pub depth: bool,
    pub stripped: bool,
    pub multisample: u32,
}

impl Default for ShaderDescriptor {
    fn default() -> Self {
        Self {
            source: ShaderSource::Code(include_str!("default_shader.wgsl").to_string()),
            descriptors: Vec::new(),
            bind_group_layouts: Vec::new(),
            vs_entry: "vs_main",
            fs_entry: "fs_main",
            backface_culling: true,
            depth: false,
            stripped: false,
            multisample: 1,
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

impl ShaderDescriptor {
    pub fn with_descriptors(mut self, descriptors: Vec<BufferLayout>) -> Self {
        self.descriptors = descriptors;
        self
    }

    pub fn with_backface_culling(mut self, value: bool) -> Self {
        self.backface_culling = value;
        self
    }

    pub fn with_bind_group_layouts(mut self, layouts: Vec<BindGroupLayout>) -> Self {
        self.bind_group_layouts = layouts;
        self
    }

    pub fn with_depth(mut self, value: bool) -> Self {
        self.depth = value;
        self
    }

    pub fn with_vs_entry(mut self, entry: &'static str) -> Self {
        self.vs_entry = entry;
        self
    }

    pub fn with_fs_entry(mut self, entry: &'static str) -> Self {
        self.fs_entry = entry;
        self
    }

    pub fn with_source(mut self, source: ShaderSource) -> Self {
        self.source = source;
        self
    }

    pub fn with_stripping(mut self, value: bool) -> Self {
        self.stripped = value;
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_multisample(mut self, value: u32) -> Self {
        self.multisample = value;
        self
    }
}
