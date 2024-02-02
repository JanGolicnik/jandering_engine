use jandering_engine::{
    camera::DefaultCameraPlugin,
    engine::EngineDescriptor,
    object::{InstanceRaw, VertexRaw},
    plugins::Plugin,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Coultn init");
    let mut engine = jandering_engine::engine::Engine::new(EngineDescriptor::default());

    let mut plugins: Vec<Box<dyn Plugin>> =
        vec![Box::new(DefaultCameraPlugin::new(&engine.renderer))];

    let shader = jandering_engine::shader::default_shader(
        &mut engine.renderer,
        jandering_engine::shader::ShaderDescriptor {
            code: "",
            descriptors: &[VertexRaw::desc(), InstanceRaw::desc()],
            plugins: &plugins,
        },
    );

    let instances = (0..100)
        .flat_map(|x| {
            (0..100).map(move |y| jandering_engine::object::Instance {
                position: Some(cgmath::Point3::new(x as f32, 0.0, -y as f32)),
                ..Default::default()
            })
        })
        .collect();
    let triangle = jandering_engine::object::primitives::triangle(&engine.renderer, instances);
    let mut objects = vec![triangle];

    engine.run(move |context, renderer| {
        plugins.iter_mut().for_each(|e| e.update(context, renderer));
        renderer.render(&mut objects, context, &shader, &plugins);
    });
}
