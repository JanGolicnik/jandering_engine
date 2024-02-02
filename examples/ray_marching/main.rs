use jandering_engine::{
    engine::{Engine, EngineDescriptor},
    object::{primitives, Instance, InstanceRaw, VertexRaw},
    plugins::{resolution::ResolutionPlugin, time::TimePlugin, Plugin},
};

fn main() {
    env_logger::init();

    let engine_descriptor = EngineDescriptor {
        resolution: (1000, 1000),
    };
    let mut engine = Engine::new(engine_descriptor);

    let mut plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(TimePlugin::new(&engine.renderer)),
        Box::new(ResolutionPlugin::new(&engine.renderer)),
    ];

    let shader = jandering_engine::shader::create_shader(
        &mut engine.renderer,
        jandering_engine::shader::ShaderDescriptor {
            code: include_str!("shader.wgsl"),
            descriptors: &[VertexRaw::desc(), InstanceRaw::desc()],
            plugins: &plugins,
        },
    );

    let quad = primitives::quad(&engine.renderer, vec![Instance::default()]);
    let mut objects = vec![quad];

    engine.run(move |context, renderer| {
        plugins.iter_mut().for_each(|e| e.update(context, renderer));
        renderer.render(&mut objects, context, &shader, &plugins);
    });
}
