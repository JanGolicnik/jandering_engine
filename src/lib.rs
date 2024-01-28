use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Coultn init");

    let engine = jandering_engine::engine::Engine::default();

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

    engine.run(move |renderer, encoder, plugins, surface, shaders, _, _| {
        renderer.render(&mut objects, encoder, plugins, surface, shaders);
    });
}
