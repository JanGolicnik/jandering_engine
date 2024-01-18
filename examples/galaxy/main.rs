use billboard::{Billboard, BillboardInstance};
use custom_camera::CustomCameraPlugin;
use jandering_engine::{engine::Engine, object::VertexRaw};

mod billboard;
mod custom_camera;

fn main() {
    env_logger::init();

    let mut engine = Engine::new(vec![Box::new(CustomCameraPlugin::new())]);

    let instances = (-10..10)
        .flat_map(|y| {
            (-10..10).map(move |x| BillboardInstance {
                position: [x as f32, y as f32, 0.0],
                size: 1.0,
            })
        })
        .collect::<Vec<_>>();

    let mut star = Billboard::new(&engine.renderer, instances);

    star.shader = engine.add_shader({
        let layout =
            engine
                .renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Default Shader Layout"),
                    bind_group_layouts: &[&engine.renderer.device.create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX
                                    | wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            }],
                            label: Some("model_bind_group_layout"),
                        },
                    )],
                    push_constant_ranges: &[],
                });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Star Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle_shader.wgsl").into()),
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
                    buffers: &[VertexRaw::desc(), BillboardInstance::desc()],
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
                    cull_mode: None,
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

    let mut objects = vec![star];

    engine.run(move |renderer, encoder, plugins, surface, shaders, _| {
        renderer.render(&mut objects, encoder, plugins, surface, shaders);
    });
}
