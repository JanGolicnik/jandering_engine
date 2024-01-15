use jandering_engine::{engine::Engine, object::Instance};

mod custom_camera;

fn main() {
    env_logger::init();

    let engine = Engine::default();

    let instances = (0..1)
        .flat_map(|y| {
            (0..1).map(move |x| Instance {
                position: Some(cgmath::Point3 {
                    x: x as f32,
                    y: y as f32,
                    z: 0.0,
                }),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();

    let instance_data: Vec<_> = instances.iter().map(|e| e.to_raw()).collect();

    let mut triangle =
        jandering_engine::object::primitives::triangle(&engine.renderer, &instance_data);
    triangle.instances = instances;
    triangle.instance_data = instance_data;

    // triangle.shader = Some({
    //     let layout =
    //         engine
    //             .renderer
    //             .device
    //             .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    //                 label: Some("Default Shader Layout"),
    //                 bind_group_layouts: &[&engine.camera.1.bind_group_layout],
    //                 push_constant_ranges: &[],
    //             });
    //     let shader = wgpu::ShaderModuleDescriptor {
    //         label: Some("Default Shader"),
    //         source: wgpu::ShaderSource::Wgsl(include_str!("triangle_shader.wgsl").into()),
    //     };
    //     let shader = engine.renderer.device.create_shader_module(shader);
    //     engine
    //         .renderer
    //         .device
    //         .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    //             label: Some("Render Pipeline"),
    //             layout: Some(&layout),
    //             vertex: wgpu::VertexState {
    //                 module: &shader,
    //                 entry_point: "vs_main",
    //                 buffers: &[VertexRaw::desc(), InstanceRaw::desc()],
    //             },
    //             fragment: Some(wgpu::FragmentState {
    //                 module: &shader,
    //                 entry_point: "fs_main",
    //                 targets: &[Some(wgpu::ColorTargetState {
    //                     format: engine.renderer.config.format,
    //                     blend: Some(wgpu::BlendState {
    //                         alpha: wgpu::BlendComponent::OVER,
    //                         color: wgpu::BlendComponent {
    //                             src_factor: wgpu::BlendFactor::SrcAlpha,
    //                             dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    //                             operation: wgpu::BlendOperation::Add,
    //                         },
    //                     }),
    //                     write_mask: wgpu::ColorWrites::ALL,
    //                 })],
    //             }),
    //             primitive: wgpu::PrimitiveState {
    //                 topology: wgpu::PrimitiveTopology::TriangleList,
    //                 strip_index_format: None,
    //                 front_face: wgpu::FrontFace::Ccw,
    //                 // cull_mode: Some(wgpu::Face::Back),
    //                 cull_mode: None,
    //                 polygon_mode: wgpu::PolygonMode::Fill,
    //                 unclipped_depth: false,
    //                 conservative: false,
    //             },
    //             depth_stencil: None,
    //             multisample: wgpu::MultisampleState {
    //                 count: 1,
    //                 mask: !0,
    //                 alpha_to_coverage_enabled: false,
    //             },
    //             multiview: None,
    //         })
    // });

    let mut objects = vec![triangle];

    engine.run(move |renderer, encoder, plugins, surface, shaders, _| {
        renderer.render(&mut objects, encoder, plugins, surface, shaders);
    });
}
