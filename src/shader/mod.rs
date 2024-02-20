use wgpu::VertexBufferLayout;

use crate::renderer::{Renderer, UntypedBindGroupHandle};

pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,
}

pub struct ShaderDescriptor<'a> {
    pub code: &'a str,
    pub descriptors: &'a [VertexBufferLayout<'a>],
    pub bind_groups: &'a [UntypedBindGroupHandle],
    pub targets: Option<&'a [Option<wgpu::ColorTargetState>]>,
    pub vs_entry: &'a str,
    pub fs_entry: &'a str,
}

impl<'a> Default for ShaderDescriptor<'a> {
    fn default() -> Self {
        Self {
            code: include_str!("default_shader.wgsl"),
            descriptors: &[],
            bind_groups: &[],
            targets: None,
            vs_entry: "vs_main",
            fs_entry: "fs_main",
        }
    }
}

pub fn create_shader(renderer: &mut Renderer, desc: ShaderDescriptor) -> Shader {
    let bind_groups: Vec<_> = desc
        .bind_groups
        .iter()
        .flat_map(|e| renderer.get_bind_group(*e))
        .collect();

    let bind_group_layouts: Vec<_> = bind_groups
        .iter()
        .flat_map(|e| e.get_bind_group_layout())
        .collect();

    let layout = renderer
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader Layout"),
            bind_group_layouts: &bind_group_layouts[..],
            push_constant_ranges: &[],
        });
    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(desc.code.into()),
    };

    let default_targets = [Some(wgpu::ColorTargetState {
        format: renderer.config.format,
        blend: Some(wgpu::BlendState {
            alpha: wgpu::BlendComponent::OVER,
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    })];

    let targets = if let Some(targets) = desc.targets {
        targets
    } else {
        &default_targets
    };

    let shader = renderer.device.create_shader_module(shader);
    let pipeline = renderer
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: None,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

    Shader { pipeline }
}

pub fn default_flat_shader(renderer: &mut Renderer, mut desc: ShaderDescriptor) -> Shader {
    desc.code = include_str!("flat_shader.wgsl");
    create_shader(renderer, desc)
}
