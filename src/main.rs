use std::f32::consts::PI;

use billboard::{Billboard, BillboardInstance};
use custom_camera::CustomCameraPlugin;
use jandering_engine::{
    engine::Engine, object::VertexRaw, plugins::Plugin, shader::ShaderDescriptor,
};
use winit::dpi::PhysicalSize;

mod billboard;
mod custom_camera;

const N_STARS_PER_ORBITAL: u32 = 1000;
const N_ORBITALS: u32 = 100;
const N_STARS: u32 = N_ORBITALS * N_STARS_PER_ORBITAL;

fn main() {
    env_logger::init();

    let engine_descriptor = jandering_engine::engine::EngineDescriptor::default();
    let mut engine = Engine::new(engine_descriptor);
    engine.window.set_cursor_visible(false);

    let mut plugins: Vec<Box<dyn Plugin>> =
        vec![Box::new(CustomCameraPlugin::new(&mut engine.renderer))];

    let shader = jandering_engine::shader::create_shader(
        &mut engine.renderer,
        ShaderDescriptor {
            code: include_str!("star_shader.wgsl"),
            descriptors: &[VertexRaw::desc(), BillboardInstance::desc()],
            plugins: &plugins,
        },
    );

    let instances = (0..N_STARS)
        .enumerate()
        .map(|(index, _)| BillboardInstance {
            size: 1.0 - index as f32 / N_STARS as f32,
            ..Default::default()
        })
        .collect();

    let star = Billboard::new(&engine.renderer, instances);
    let mut stars = vec![star];

    let mut time = 0.0;
    engine.window.set_inner_size(PhysicalSize::new(1000, 1000));

    engine.run(move |context, renderer| {
        time += context.dt;

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
        plugins.iter_mut().for_each(|e| e.update(context, renderer));

        renderer.render(&mut stars, context, &shader, &plugins);
    });
}
