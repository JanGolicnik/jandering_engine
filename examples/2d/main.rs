// use jandering_engine::{
//     engine::{Engine, EngineDescriptor},
//     object::{primitives, D2Instance, VertexRaw},
//     plugins::{
//         camera::d2::D2CameraPlugin, definition::make_plugin, resolution::ResolutionPlugin,
//         texture::TexturePlugin, PluginVec,
//     },
//     texture::Texture,
//     types::Vec2,
// };

// fn main() {
//     env_logger::init();

//     let engine_descriptor = EngineDescriptor {
//         resolution: (1000, 1000),
//     };
//     let mut engine = Engine::new(engine_descriptor);

//     let mut plugins: PluginVec = vec![
//         make_plugin(D2CameraPlugin::new(&engine.renderer, true)),
//         make_plugin(ResolutionPlugin::new(&engine.renderer)),
//         make_plugin(pollster::block_on(TexturePlugin::new(
//             &engine.renderer,
//             Texture::from_bytes(&engine.renderer, include_bytes!("tree.png"), false),
//         ))),
//     ];

//     let shader = jandering_engine::shader::default_flat_shader(
//         &mut engine.renderer,
//         jandering_engine::shader::ShaderDescriptor {
//             descriptors: &[VertexRaw::desc(), D2Instance::desc()],
//             plugins: &plugins,
//             code: "",
//         },
//     );

//     let mut quad = primitives::quad::<D2Instance>(
//         &engine.renderer,
//         vec![D2Instance {
//             position: Vec2::new(0.0, 0.0),
//             scale: Vec2::new(100.0, 100.0),
//             rotation: 0.0,
//         }],
//     );

//     let mut time = 0.0;

//     engine.run(move |context, renderer| {
//         time += context.dt;

//         for instance in quad.instances.iter_mut() {
//             instance.position.x = 190.0 * (time as f32 * 5.0).sin();
//             instance.position.y = 190.0 * (time as f32 * 5.0).cos();
//         }
//         quad.update(context, renderer);
//         plugins
//             .iter_mut()
//             .for_each(|e| e.borrow_mut().update(context, renderer));
//         renderer.render(&[&quad], context, &shader, &plugins);
//     });
// }

fn main() {}
