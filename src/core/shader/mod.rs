use super::bind_group::BindGroupLayout;

pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,
}

pub struct ShaderDescriptor<'a> {
    pub code: &'a str,
    pub descriptors: &'a [wgpu::VertexBufferLayout<'a>], //TODO: abstract this away so there is no dependency on wgpu
    pub bind_group_layouts: Vec<BindGroupLayout>,
    pub vs_entry: &'a str,
    pub fs_entry: &'a str,
    pub backface_culling: bool,
    pub depth: bool,
}

impl<'a> Default for ShaderDescriptor<'a> {
    fn default() -> Self {
        Self {
            code: include_str!("default_shader.wgsl"),
            descriptors: &[],
            bind_group_layouts: Vec::new(),
            vs_entry: "vs_main",
            fs_entry: "fs_main",
            backface_culling: true,
            depth: false,
        }
    }
}

impl<'a> ShaderDescriptor<'a> {
    pub fn default_flat() -> Self {
        Self {
            code: include_str!("flat_shader.wgsl"),
            ..Default::default()
        }
    }
}

impl<'a> ShaderDescriptor<'a> {
    pub fn with_descriptors(mut self, descriptors: &'a [wgpu::VertexBufferLayout<'a>]) -> Self {
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
}
