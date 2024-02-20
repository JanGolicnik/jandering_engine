// use std::f32::consts::PI;

// use custom_camera::CustomCameraPlugin;
// use jandering_engine::{
//     bind_group::{definition::make_plugin, BindGroupVec},
//     engine::Engine,
//     object::VertexRaw,
//     shader::ShaderDescriptor,
//     types::Vec3,
// };
// use winit::dpi::PhysicalSize;

// mod custom_camera;

// const N_STARS_PER_ORBITAL: u32 = 1000;
// const N_ORBITALS: u32 = 100;
// const N_STARS: u32 = N_ORBITALS * N_STARS_PER_ORBITAL;

// fn main() {
//     env_logger::init();

//     let engine_descriptor = jandering_engine::engine::EngineDescriptor::default();
//     let mut engine = Engine::new(engine_descriptor);
//     engine.window.set_cursor_visible(false);

//     let mut plugins: BindGroupVec =
//         vec![make_plugin(CustomCameraPlugin::new(&mut engine.renderer))];

//     let shader = jandering_engine::shader::create_shader(
//         &mut engine.renderer,
//         ShaderDescriptor {
//             code: include_str!("star_shader.wgsl"),
//             descriptors: &[VertexRaw::desc(), BillboardInstance::desc()],
//             plugins: &plugins,
//             targets: None,
//         },
//     );

//     let instances = (0..N_STARS)
//         .enumerate()
//         .map(|(index, _)| BillboardInstance {
//             size: 1.0 - index as f32 / N_STARS as f32,
//             ..Default::default()
//         })
//         .collect();

//     let mut star = jandering_engine::object::primitives::triangle(&engine.renderer, instances);

//     let mut time = 0.0;
//     engine.window.set_inner_size(PhysicalSize::new(1000, 1000));

//     engine.run(move |context, renderer| {
//         time += context.dt;

//         for (index, instance) in star.instances.iter_mut().enumerate() {
//             let orbit = index as u32 / N_STARS_PER_ORBITAL + 1;
//             let index_in_orbit = index as u32 % N_STARS_PER_ORBITAL;

//             let radius = orbit as f32;

//             // let speed = 0.0;
//             let speed = radius.powf(0.1) * 0.2;

//             let mut offset = (1.0 / N_STARS_PER_ORBITAL as f32) * index_in_orbit as f32;
//             offset *= PI * 2.0;

//             let x = (time as f32 * speed + offset).sin() * radius;
//             let y = (time as f32 * speed + offset).cos() * 0.7 * radius;

//             let deg = orbit as f32 * (2.0 + time as f32 * 1.5);
//             let rad = deg * (PI / 180.0);
//             let angle_sin = rad.sin();
//             let angle_cos = rad.cos();

//             let rotated_x = x * angle_cos - y * angle_sin;
//             let rotated_y = x * angle_sin + y * angle_cos;

//             instance.position = Vec3::new(rotated_x, 0.0, rotated_y)
//         }

//         star.update(context, renderer);

//         plugins
//             .iter_mut()
//             .for_each(|e| e.borrow_mut().update(context, renderer));

//         renderer.render(&[&star], context, &shader, &plugins);
//     });
// }

// #[repr(C)]
// #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct BillboardInstance {
//     pub position: Vec3,
//     //
//     pub size: f32,
// }

// impl Default for BillboardInstance {
//     fn default() -> Self {
//         Self {
//             size: 1.0,
//             position: Vec3::new(0.0, 0.0, 0.0),
//         }
//     }
// }

// impl BillboardInstance {
//     pub fn desc() -> wgpu::VertexBufferLayout<'static> {
//         use std::mem;

//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<BillboardInstance>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Instance,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 5,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shader_location: 6,
//                     format: wgpu::VertexFormat::Float32,
//                 },
//             ],
//         }
//     }
// }
fn main() {}
