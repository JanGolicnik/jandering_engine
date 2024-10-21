use grass_object::GrassObject;
use image::GenericImageView;
use jandering_engine::{
    bind_group::{
        camera::free::{FreeCameraController, MatrixCamera},
        texture::TextureBindGroup,
        BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutDescriptorEntry,
        BindGroupLayoutEntry,
    },
    engine::Engine,
    object::{Instance, Object, Vertex},
    renderer::{BufferHandle, Janderer, Renderer},
    shader::{ShaderDescriptor, ShaderSource},
    texture::{sampler::SamplerDescriptor, texture_usage, TextureDescriptor, TextureFormat},
    types::{UVec2, Vec3},
};
use je_windowing::{Key, WindowConfig, WindowEvent, WindowManagerTrait, WindowTrait};
use star_object::StarObject;

mod grass_object;
mod star_object;

const STAR_COUNT: u32 = 10;

const GRASS_DENSITY: f32 = 1.0;

const GRASS_LOD1_SIDE: f32 = 100.0;
const GRASS_LOD1_AREA: f32 = GRASS_LOD1_SIDE * GRASS_LOD1_SIDE;
const GRASS_LOD1_N: u32 = (GRASS_LOD1_AREA * GRASS_DENSITY) as u32;

const GRASS_LOD2_SIDE: f32 = 10.0;
const GRASS_LOD2_AREA: f32 = GRASS_LOD2_SIDE * GRASS_LOD2_SIDE;
const GRASS_LOD2_N: u32 = (GRASS_LOD2_AREA * GRASS_DENSITY) as u32;

const MULTISAMPLE: u32 = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

pub struct RenderDataData {
    ground_color: Vec3,
    time: f32,
    grass_top_color: Vec3,
    grass_height: f32,
    sky_color: Vec3,
    grass_height_variation: f32,
    wind_strength: f32,
    wind_scale: f32,
    wind_speed: f32,
    wind_direction: f32,
    wind_noise_scale: f32,
    wind_noise_strength: f32,
    sqrt_n_grass: u32,
    terrain_size: f32,
    render_square_size: f32,
    fov_x: f32,
    padding: [f32; 2],
}

pub struct RenderDataBindGroup {
    pub data: RenderDataData,

    pub buffer_handle: BufferHandle,
}

impl BindGroup for RenderDataBindGroup {
    fn get_layout(&self) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(self.buffer_handle)],
        }
    }

    fn get_layout_descriptor() -> jandering_engine::bind_group::BindGroupLayoutDescriptor
    where
        Self: Sized,
    {
        BindGroupLayoutDescriptor {
            entries: vec![BindGroupLayoutDescriptorEntry::Data { is_uniform: true }],
        }
    }
}

impl RenderDataBindGroup {
    fn new(renderer: &mut Renderer) -> Self {
        let data = RenderDataData {
            time: 0.0,
            ground_color: Vec3::new(0.1, 0.4, 0.2),
            grass_top_color: Vec3::new(0.5, 1.0, 0.6),
            sky_color: Vec3::new(0.02, 0.05, 0.1),
            grass_height: 1.0,
            grass_height_variation: 0.5,
            wind_strength: 0.2,
            wind_scale: 200.0,
            wind_speed: 5.0,
            wind_direction: 0.0,
            wind_noise_scale: 2.0,
            wind_noise_strength: 0.1,
            sqrt_n_grass: 0,
            terrain_size: 1000.0,
            render_square_size: 100.0,
            fov_x: 45.0,
            padding: [0.0; 2],
        };

        let buffer_handle = renderer.create_uniform_buffer(bytemuck::cast_slice(&[data]));

        Self {
            data,
            buffer_handle,
        }
    }
}

const CAMERA_FOV: f32 = 45.0;
const CAMEREA_NEAR: f32 = 0.01;
const CAMEREA_FAR: f32 = 100000.0;
fn main() {
    let mut engine = pollster::block_on(Engine::default());

    let mut window = engine.spawn_window(
        // also registers window by itself
        WindowConfig::default()
            .with_cursor(false)
            .with_title("Grass")
            .with_auto_resolution()
            .with_decorations(false)
            .with_transparency(true),
    );

    let renderer = &mut engine.renderer;

    let mut camera = MatrixCamera::with_controller(renderer, FreeCameraController::default());
    camera.make_perspective(CAMERA_FOV, 1.0, CAMEREA_NEAR, CAMEREA_FAR);
    let render_data = RenderDataBindGroup::new(renderer);
    let render_data_bind_group_layout = RenderDataBindGroup::get_layout_descriptor();

    let tex_sampler = renderer.create_sampler(SamplerDescriptor {
        address_mode: jandering_engine::texture::sampler::SamplerAddressMode::RepeatMirrored,
        ..Default::default()
    });

    let heightmap_image = image::load_from_memory(include_bytes!("heightmap.jpg")).unwrap();
    let heightmap_handle = renderer.create_texture(TextureDescriptor {
        data: Some(&heightmap_image.to_rgba8()),
        size: heightmap_image.dimensions().into(),
        ..Default::default()
    });
    let heightmap_texture = TextureBindGroup::new(renderer, heightmap_handle, tex_sampler);

    let noise_image = image::load_from_memory(include_bytes!("bignoise.png")).unwrap();
    let noise_handle = renderer.create_texture(TextureDescriptor {
        data: Some(&noise_image.to_rgba8()),
        size: noise_image.dimensions().into(),
        ..Default::default()
    });
    let noise_texture = TextureBindGroup::new(renderer, noise_handle, tex_sampler);

    let multisample_texture = renderer.create_texture(TextureDescriptor {
        size: (1920, 1080).into(),
        sample_count: MULTISAMPLE,
        usage: texture_usage::TARGET,
        ..Default::default()
    });

    let depth_texture = renderer.create_texture(TextureDescriptor {
        size: (1920, 1080).into(),
        sample_count: MULTISAMPLE,
        format: TextureFormat::Depth32F,
        usage: texture_usage::TARGET | texture_usage::BIND,
        ..Default::default()
    });

    let grass_shader = renderer.create_shader(
        ShaderDescriptor::default()
            .with_descriptors(vec![Vertex::desc()])
            .with_bind_group_layout_descriptors(vec![
                camera.get_layout_descriptor(),
                render_data_bind_group_layout.clone(),
                TextureBindGroup::get_layout_descriptor(),
                TextureBindGroup::get_layout_descriptor(),
            ])
            .with_depth(true)
            .with_backface_culling(false)
            .with_multisample(MULTISAMPLE)
            .with_source(ShaderSource::Code(
                include_str!("terrain_shader.wgsl").to_string(),
            ))
            .with_fs_entry("fs_grass")
            .with_vs_entry("vs_grass"),
    );

    let ground_shader = renderer.create_shader(
        ShaderDescriptor::default()
            .with_descriptors(vec![Vertex::desc(), Instance::desc()])
            .with_bind_group_layout_descriptors(vec![
                camera.get_layout_descriptor(),
                render_data_bind_group_layout.clone(),
                TextureBindGroup::get_layout_descriptor(),
            ])
            .with_backface_culling(false)
            .with_depth(true)
            .with_multisample(MULTISAMPLE)
            .with_source(ShaderSource::Code(
                include_str!("terrain_shader.wgsl").to_string(),
            ))
            .with_fs_entry("fs_ground")
            .with_vs_entry("vs_ground"),
    );

    let star_shader = renderer.create_shader(
        ShaderDescriptor::default()
            .with_descriptors(vec![Vertex::desc()])
            .with_bind_group_layout_descriptors(vec![
                camera.get_layout_descriptor(),
                render_data_bind_group_layout.clone(),
            ])
            .with_backface_culling(true)
            .with_depth(true)
            .with_multisample(MULTISAMPLE)
            .with_source(ShaderSource::Code(
                include_str!("star_shader.wgsl").to_string(),
            )),
    );

    let render_data_handle = renderer.create_typed_bind_group(render_data);

    let heightmap_texture = renderer.create_typed_bind_group(heightmap_texture);
    let noise_texture = renderer.create_typed_bind_group(noise_texture);

    let grass_lod1 = GrassObject::from_text(include_str!("grass_lod1.obj"), renderer, GRASS_LOD1_N);
    let grass_lod2 = GrassObject::from_text(include_str!("grass_lod2.obj"), renderer, GRASS_LOD2_N);

    let ground = Object::from_obj(
        include_str!("plane.obj"),
        // include_str!("grass_lod1.obj"),
        renderer,
        vec![Instance::default().scale(1000.0)],
    );

    let star_triangle = StarObject::new(renderer, STAR_COUNT);

    let mut fps_accumulator: f32 = Default::default();
    let mut fps_counter: u32 = Default::default();

    let mut is_in_fps: bool = Default::default();

    let mut last_time = web_time::Instant::now();
    engine.run(move |renderer, window_manager| {
        if window.should_close() {
            window_manager.end();
        }

        window.poll_events();
        let events = window.events().clone();

        for event in events.iter() {
            match event {
                WindowEvent::WindowInitialized => renderer.register_window(&window),
                _ => {}
            }
        }

        let current_time = web_time::Instant::now();
        let dt = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        fps_accumulator += dt;
        fps_counter += 1;
        if fps_accumulator > 1.0 {
            println!("fps: {}", 1.0 / (fps_accumulator / fps_counter as f32));
            fps_accumulator = 0.0;
            fps_counter = 0;
        }
        {
            let render_data = renderer
                .get_typed_bind_group_mut(render_data_handle)
                .unwrap();
            render_data.data.time += dt;
        }

        if is_in_fps {
            camera.update(renderer, &events, dt);

            if events.is_pressed(Key::Alt) {
                is_in_fps = false;
                window.set_cursor_visible(true);
            }
        } else if events.is_mouse_pressed(je_windowing::MouseButton::Left) {
            is_in_fps = true;
            window.set_cursor_visible(false);
        }

        if events.iter().any(|e| matches!(e, WindowEvent::Resized(_))) {
            let size: UVec2 = window.size().into();
            camera.make_perspective(
                CAMERA_FOV,
                size.x as f32 / size.y as f32,
                CAMEREA_NEAR,
                CAMEREA_FAR,
            );

            renderer.re_create_texture(
                TextureDescriptor {
                    size,
                    format: TextureFormat::Depth32F,
                    sample_count: MULTISAMPLE,
                    usage: texture_usage::TARGET | texture_usage::BIND,
                    ..Default::default()
                },
                depth_texture,
            );

            renderer.re_create_texture(
                TextureDescriptor {
                    size,
                    sample_count: MULTISAMPLE,
                    usage: texture_usage::TARGET,
                    ..Default::default()
                },
                multisample_texture,
            );
        }

        if events.is_pressed(Key::B) {
            renderer.re_create_shaders();
        }

        if window.is_initialized() {
            let render_data = renderer
                .get_typed_bind_group_mut(render_data_handle)
                .unwrap();
            let sky_color = render_data.data.sky_color;

            render_data.data.sqrt_n_grass = (GRASS_LOD2_N as f32).sqrt() as u32;
            render_data.data.render_square_size = GRASS_LOD2_SIDE;
            let render_data = renderer.get_typed_bind_group(render_data_handle).unwrap();
            renderer.write_buffer(
                render_data.buffer_handle,
                bytemuck::cast_slice(&[render_data.data]),
            );

            let mut pass = renderer.new_pass(&mut window);
            pass.set_shader(star_shader)
                .with_target_texture_resolve(
                    jandering_engine::renderer::TargetTexture::Handle(multisample_texture),
                    None,
                )
                .with_depth(depth_texture, Some(1.0))
                .with_clear_color(sky_color.x, sky_color.y, sky_color.z)
                .bind(0, camera.bind_group())
                .bind(1, render_data_handle.into())
                .render(&[&star_triangle])
                .set_shader(ground_shader)
                .bind(2, heightmap_texture.into())
                .render(&[&ground])
                .set_shader(grass_shader)
                .bind(3, noise_texture.into())
                .render(&[&grass_lod2]);
            pass.submit();

            let render_data = renderer
                .get_typed_bind_group_mut(render_data_handle)
                .unwrap();
            render_data.data.sqrt_n_grass = (GRASS_LOD1_N as f32).sqrt() as u32;
            render_data.data.render_square_size = GRASS_LOD1_SIDE;
            let render_data = renderer.get_typed_bind_group(render_data_handle).unwrap();
            renderer.write_buffer(
                render_data.buffer_handle,
                bytemuck::cast_slice(&[render_data.data]),
            );

            let mut pass = renderer.new_pass(&mut window);
            pass.set_shader(grass_shader)
                .with_depth(depth_texture, None)
                .with_target_texture_resolve(
                    jandering_engine::renderer::TargetTexture::Handle(multisample_texture),
                    None,
                )
                .bind(0, camera.bind_group())
                .bind(1, render_data_handle.into())
                .bind(2, heightmap_texture.into())
                .bind(3, noise_texture.into())
                .render(&[&grass_lod1]);
            pass.submit();
        }

        window.request_redraw();
    });
}
