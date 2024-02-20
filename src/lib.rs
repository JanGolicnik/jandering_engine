use jandering_engine::{
    bind_group::camera::free::FreeCameraBindGroup,
    engine::EngineDescriptor,
    object::{Instance, VertexRaw},
    types::{Mat4, UVec2, Vec3},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Coultn init");
    let mut engine = jandering_engine::engine::Engine::new(EngineDescriptor::default());

    let camera_bg = engine
        .renderer
        .add_bind_group(FreeCameraBindGroup::new(&engine.renderer));

    let untyped_bind_groups = [camera_bg.into()];
    let shader = jandering_engine::shader::create_shader(
        &mut engine.renderer,
        jandering_engine::shader::ShaderDescriptor {
            descriptors: &[VertexRaw::desc(), Instance::desc()],
            bind_groups: &untyped_bind_groups,
            ..Default::default()
        },
    );

    let instances = (-10..11)
        .flat_map(|x| {
            (-10..11).map(move |y| {
                let model: Mat4 = Mat4::from_translation(Vec3::new(x as f32, 0.0, y as f32));
                Instance { model }
            })
        })
        .collect();

    let triangle = jandering_engine::object::primitives::triangle(&engine.renderer, instances);

    engine.run(move |context, renderer| {
        let resolution = UVec2::new(renderer.config.width, renderer.config.height);
        let camera_bind_group = renderer.get_bind_group_t_mut(camera_bg).unwrap();
        camera_bind_group.update(context, resolution);

        renderer.render(&[&triangle], context, &shader, &untyped_bind_groups);
    });
}
