use std::f32::consts::PI;

use billboard::{Billboard, BillboardInstance};
use custom_camera::CustomCameraPlugin;
use jandering_engine::{engine::Engine, object::VertexRaw};

mod billboard;
mod custom_camera;

const N_STARS_PER_ORBITAL: u32 = 250;
const N_ORBITALS: u32 = 100;
const N_STARS: u32 = N_ORBITALS * N_STARS_PER_ORBITAL;

fn main() {
    env_logger::init();

    let mut engine = Engine::new(vec![Box::new(CustomCameraPlugin::new())]);

    let instances = (0..N_STARS).map(|_| BillboardInstance::default()).collect();

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
            source: wgpu::ShaderSource::Wgsl(include_str!("star_shader.wgsl").into()),
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

    let mut stars = vec![star];

    let mut time = 0.0;

    engine.run(move |renderer, encoder, plugins, surface, shaders, dt| {
        time += dt;

        let star = stars.first_mut().unwrap();

        for (mut index, instance) in star.instances.iter_mut().enumerate() {
            let orbit = index as u32 / N_STARS_PER_ORBITAL + 1;
            let index_in_orbit = index as u32 % N_STARS_PER_ORBITAL;

            let radius = orbit as f32;

            let speed = radius.powf(0.1) * 0.3;

            let mut offset = (1.0 / N_STARS_PER_ORBITAL as f32) * index_in_orbit as f32;
            offset *= PI * 2.0;

            let radius_x = 1.0 + ((orbit as f32 / (time as f32 * 0.02)).sin() + 1.0) / 5.0;
            let radius_y = 1.0 + ((orbit as f32 / (time as f32 * 0.02)).cos() + 1.0) / 5.0;

            instance.position = [
                (time as f32 * speed + offset).sin() * radius_x * radius,
                0.0,
                (time as f32 * speed + offset).cos() * radius_y * radius,
            ]
        }

        renderer.render(&mut stars, encoder, plugins, surface, shaders);
    });
}
