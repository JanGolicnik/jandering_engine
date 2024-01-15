use jandering_engine::{engine::Engine, object::Instance};
fn main() {
    env_logger::init();

    let engine = Engine::default();

    let instances = (-100..10)
        .flat_map(|z| {
            (-20..20).map(move |x| Instance {
                position: Some(cgmath::Point3 {
                    x: x as f32,
                    y: 0.0,
                    z: z as f32,
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

    let mut objects = vec![triangle];

    engine.run(move |renderer, encoder, plugins, surface, shaders, _| {
        renderer.render(&mut objects, encoder, plugins, surface, shaders);
    });
}
