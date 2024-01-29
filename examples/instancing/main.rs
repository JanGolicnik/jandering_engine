use std::f32::consts::PI;

use billboard::{Billboard, BillboardInstance};
use custom_camera::CustomCameraPlugin;
use jandering_engine::{engine::Engine, object::VertexRaw};
use winit::dpi::PhysicalSize;

mod billboard;
mod custom_camera;

const N_STARS_PER_ORBITAL: u32 = 1000;
const N_ORBITALS: u32 = 100;
const N_STARS: u32 = N_ORBITALS * N_STARS_PER_ORBITAL;

fn main() {
    env_logger::init();

    let engine_descriptor = EngineDescriptor {
        plugins: vec![Box::new(CustomCameraPlugin::new())],
        ..Default::default()
    };
    let mut engine = Engine::new(engine_descriptor);
    engine.window.set_cursor_visible(false);

    let instances = (0..N_STARS)
        .enumerate()
        .map(|(index, _)| BillboardInstance {
            size: 1.0 - index as f32 / N_STARS as f32,
            ..Default::default()
        })
        .collect();

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
    engine.window.set_inner_size(PhysicalSize::new(1000, 1000));

    engine.run(move |renderer, encoder, plugins, surface, shaders, _, dt| {
        time += dt;

        let star = stars.first_mut().unwrap();

        for (index, instance) in star.instances.iter_mut().enumerate() {
            let orbit = index as u32 / N_STARS_PER_ORBITAL + 1;
            let index_in_orbit = index as u32 % N_STARS_PER_ORBITAL;

            let radius = orbit as f32;

            // let speed = 0.0;
            let speed = radius.powf(0.1) * 0.2;

            let mut offset = (1.0 / N_STARS_PER_ORBITAL as f32) * index_in_orbit as f32;
            offset *= PI * 2.0;

            let x = (time as f32 * speed + offset).sin() * radius;
            let y = (time as f32 * speed + offset).cos() * 0.7 * radius;

            let deg = orbit as f32 * (2.0 + time as f32 * 1.5);
            let rad = deg * (PI / 180.0);
            let angle_sin = rad.sin();
            let angle_cos = rad.cos();

            let rotated_x = x * angle_cos - y * angle_sin;
            let rotated_y = x * angle_sin + y * angle_cos;

            instance.position = [rotated_x, 0.0, rotated_y]
        }

        renderer.render(&mut stars, encoder, plugins, surface, shaders);
    });
}
