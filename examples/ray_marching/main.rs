// use jandering_engine::{
//     bind_group::{
//         definition::make_plugin, resolution::ResolutionBindGroup, time::TimeBindGroup, BindGroupVec,
//     },
//     engine::{Engine, EngineDescriptor},
//     object::{primitives, Instance, VertexRaw},
// };

// fn main() {
//     env_logger::init();

//     let engine_descriptor = EngineDescriptor {
//         resolution: (1000, 1000),
//     };
//     let mut engine = Engine::new(engine_descriptor);

//     let mut plugins: BindGroupVec = vec![
//         make_plugin(TimeBindGroup::new(&engine.renderer)),
//         make_plugin(ResolutionBindGroup::new(&engine.renderer)),
//     ];

//     let shader = jandering_engine::shader::create_shader(
//         &mut engine.renderer,
//         jandering_engine::shader::ShaderDescriptor {
//             code: include_str!("shader.wgsl"),
//             descriptors: &[VertexRaw::desc(), Instance::desc()],
//             plugins: &plugins,
//             targets: None,
//         },
//     );

//     let quad = primitives::quad::<Instance>(&engine.renderer, vec![Instance::default()]);

//     engine.run(move |context, renderer| {
//         plugins
//             .iter_mut()
//             .for_each(|e| e.borrow_mut().update(context, renderer));
//         renderer.render(&[&quad], context, &shader, &plugins);
//     });
// }
fn main() {}
