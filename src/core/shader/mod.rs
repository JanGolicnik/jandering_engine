use super::bind_group::BindGroupLayout;

#[derive(Clone)]
pub struct ShaderDescriptor {
    pub code: &'static str,
    pub descriptors: Vec<wgpu::VertexBufferLayout<'static>>, //TODO: abstract this away so there is no dependency on wgpu
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
            code: include_str!("default_shader.wgsl"),
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
    pub fn default_flat() -> Self {
        Self {
            code: include_str!("flat_shader.wgsl"),
            ..Default::default()
        }
    }
}

impl ShaderDescriptor {
    pub fn with_descriptors(mut self, descriptors: Vec<wgpu::VertexBufferLayout<'static>>) -> Self {
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

    pub fn with_source(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }

    pub fn with_stripping(mut self, value: bool) -> Self {
        self.stripped = value;
        self
    }

    pub fn with_multisample(mut self, value: u32) -> Self {
        self.multisample = value;
        self
    }
}
