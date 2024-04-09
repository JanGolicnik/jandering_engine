use crate::core::shader::{Shader, ShaderDescriptor};

use super::{Renderer, ShaderHandle};

impl Renderer {
    pub fn create_shader(&mut self, desc: ShaderDescriptor) -> ShaderHandle {
        //ugly ass code fuck off
        let bind_group_layouts = Self::get_layouts(&self.device, &desc.bind_group_layouts);
        let bind_group_ref = bind_group_layouts.iter().collect::<Vec<_>>();

        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shader Layout"),
                bind_group_layouts: &bind_group_ref,
                push_constant_ranges: &[],
            });

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(desc.code.into()),
        };

        let targets = &[Some(wgpu::ColorTargetState {
            format: self.config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let shader = self.device.create_shader_module(shader);
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: desc.vs_entry,
                    buffers: desc.descriptors,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: desc.fs_entry,
                    targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: if desc.stripped {
                        wgpu::PrimitiveTopology::TriangleStrip
                    } else {
                        wgpu::PrimitiveTopology::TriangleList
                    },
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: if desc.backface_culling {
                        Some(wgpu::Face::Back)
                    } else {
                        None
                    },
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: if desc.depth {
                    Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    })
                } else {
                    None
                },
                multisample: wgpu::MultisampleState {
                    count: desc.multisample,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
        self.shaders.push(Shader { pipeline });

        ShaderHandle(self.shaders.len() - 1)
    }
}
