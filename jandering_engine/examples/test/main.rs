
use jandering_engine::{
    bind_group::{
        camera::free::{CameraController, FreeCameraController, MatrixCameraBindGroup},
        BindGroup,
    },
    engine::Engine,
    object::{Instance, Object, Vertex},
    renderer::Janderer,
    shader::ShaderDescriptor,
    texture::{texture_usage::{self}, TextureDescriptor, TextureFormat},
    types::{UVec2, Vec3},
    window::{WindowConfig, WindowEvent, WindowManagerTrait, WindowTrait},
};

const CAMERA_FOV: f32 = 45.0;
const CAMEREA_NEAR: f32 = 0.01;
const CAMEREA_FAR: f32 = 100000.0;

fn main() {
    let mut engine = pollster::block_on(Engine::default());

    let resolution = 512;

    let mut window = engine.spawn_window(
        // also registers window by itself
        WindowConfig::default()
            .with_cursor(true)
            // .with_resolution(resolution, resolution)
            .with_auto_resolution()
            .with_decorations(false)
            .with_transparency(true)
            .with_title("beast"),
    );


    let renderer = &mut engine.renderer;

    let mut camera = MatrixCameraBindGroup::with_controller(renderer, FreeCameraController::default());
    camera.make_perspective(CAMERA_FOV, 1.0, CAMEREA_NEAR, CAMEREA_FAR);

    let shader = renderer.create_shader(
        ShaderDescriptor::default()
            .with_descriptors(vec![Vertex::desc(), Instance::desc()])
            .with_bind_group_layouts(vec![camera.get_layout()])
            .with_depth(true)
            .with_backface_culling(false),
    );

    let camera_handle = renderer.create_typed_bind_group(camera);

    const COUNT:i32 = 1;
    let mut cube_instance_grid = Vec::new();
    for x in -COUNT..=COUNT{
        for y in -COUNT..=COUNT{
            for z in -COUNT..=COUNT{
                cube_instance_grid.push(Instance::default().translate(Vec3::new(x as f32, y as f32, z as f32) * 10.0));
            }
        }
    }
    
    let mut cube = Object::from_obj(
        include_str!("cube.obj"),
        renderer,
        cube_instance_grid,
    );

    let depth_texture = renderer.create_texture(TextureDescriptor {
        size: UVec2::splat(resolution),
        format: TextureFormat::Depth32F,
        usage: texture_usage::ALL,
        ..Default::default()
    });

    let mut last_time = web_time::Instant::now();
    engine.run(move |renderer, window_manager|{
        if window.should_close(){
            window_manager.end();
        }

        window.poll_events();

        for event in window.events().iter() {
            match event{
                WindowEvent::WindowInitialized => renderer.register_window(&window),
                _=>{}
            }
        }

        let current_time = web_time::Instant::now();
        let dt = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        if window
            .events().matches(|e| matches!(e, WindowEvent::Resized(_)))
        {
            let resolution = window.size();

            let camera = renderer.get_typed_bind_group_mut(camera_handle).unwrap();
            camera.make_perspective(
                CAMERA_FOV,
                resolution.0 as f32 / resolution.1 as f32,
                CAMEREA_NEAR,
                CAMEREA_FAR,
            );
            renderer.re_create_texture(
                TextureDescriptor {
                    size: window.size().into(),
                    format: TextureFormat::Depth32F,
                    usage: texture_usage::ALL,
                    ..Default::default()
                },
                depth_texture,
            );
        }

        cube.instances.iter_mut().for_each(|e| {
            e.rotate(20.0f32.to_radians() * dt, Vec3::Y);
        });

        cube.update(renderer);

        let camera = renderer.get_typed_bind_group_mut(camera_handle).unwrap();
        camera.update(window.events(), dt);

        if window.is_initialized() {
            // dbg!("rendering");
            // let mut pass = renderer.new_pass(&mut window);
            // pass.with_clear_color(0.4, 0.7, 1.0).render_empty();
            // pass.submit();

            let data = camera.get_data();
            renderer.write_bind_group(camera_handle.into(), &data);

            let mut pass = renderer.new_pass(&mut window);
            pass.with_depth(depth_texture, Some(1.0))
                .with_alpha(0.0)
                .set_shader(shader)
                .bind(0, camera_handle.into())
                .render(&[&cube]);
            pass.submit();
        }

        window.request_redraw();
    });
}
