use jandering_engine::{
    engine::Engine,
    object::{primitives, Instance},
    plugins::{resolution::ResolutionPlugin, time::TimePlugin},
};

fn main() {
    env_logger::init();

    let mut engine = Engine::new(vec![
        Box::<TimePlugin>::default(),
        Box::<ResolutionPlugin>::default(),
    ]);

    engine
        .window
        .set_inner_size(winit::dpi::PhysicalSize::new(1000, 1000));

    let mut quad = primitives::quad(&engine.renderer, vec![Instance::default()]);

    quad.shader = engine.add_shader({
        let bind_group_layouts = engine.get_bind_group_layouts();
        let layout =
            engine
                .renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Ray Marching Shader Layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Ray Marching Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };
        let shader = engine.renderer.device.create_shader_module(shader);
        engine
            .renderer
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        jandering_engine::object::VertexRaw::desc(),
                        jandering_engine::object::InstanceRaw::desc(),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: engine.renderer.config.format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::OVER,
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
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
            })
    });

    let mut objects = vec![quad];

    engine.run(move |renderer, encoder, plugins, surface, shaders, _, _| {
        renderer.render(&mut objects, encoder, plugins, surface, shaders);
    });
}
