use grass_object::GrassObject;
use image::GenericImageView;
use jandering_engine::{
    core::{
        bind_group::{
            camera::free::FreeCameraBindGroup, texture::TextureBindGroup, BindGroup,
            BindGroupLayout, BindGroupLayoutEntry,
        },
        engine::{Engine, EngineBuilder, EngineContext},
        event_handler::EventHandler,
        object::{Instance, Object, Vertex},
        renderer::{
            create_typed_bind_group, get_typed_bind_group, get_typed_bind_group_mut,
            BindGroupHandle, BufferHandle, Renderer, ShaderHandle, TextureHandle,
        },
        shader::ShaderDescriptor,
        texture::{sampler::SamplerDescriptor, texture_usage, TextureDescriptor, TextureFormat},
        window::{InputState, Key, MouseButton, WindowBuilder, WindowEvent},
    },
    types::Vec3,
};
use star_object::StarObject;

mod grass_object;
mod star_object;

struct Application {
    time: f32,
    last_time: web_time::Instant,
    fps_accumulator: f32,
    fps_counter: u32,

    is_in_fps: bool,

    grass_lod1: GrassObject,
    grass_lod2: GrassObject,
    grass_shader: ShaderHandle,
    ground: Object<Instance>,
    ground_shader: ShaderHandle,
    star_triangle: StarObject,
    star_shader: ShaderHandle,

    heightmap_texture: BindGroupHandle<TextureBindGroup>,
    noise_texture: BindGroupHandle<TextureBindGroup>,

    camera: BindGroupHandle<FreeCameraBindGroup>,
    render_data: BindGroupHandle<RenderDataBindGroup>,

    multisample_texture: TextureHandle,
    depth_texture: TextureHandle,
}

const STAR_COUNT: u32 = 1000;

const GRASS_DENSITY: f32 = 10.0;

const GRASS_LOD1_SIDE: f32 = 100.0;
const GRASS_LOD1_AREA: f32 = GRASS_LOD1_SIDE * GRASS_LOD1_SIDE;
const GRASS_LOD1_N: u32 = (GRASS_LOD1_AREA * GRASS_DENSITY) as u32;

const GRASS_LOD2_SIDE: f32 = 1000.0;
const GRASS_LOD2_AREA: f32 = GRASS_LOD2_SIDE * GRASS_LOD2_SIDE;
const GRASS_LOD2_N: u32 = (GRASS_LOD2_AREA * GRASS_DENSITY) as u32;

const MULTISAMPLE: u32 = 4;

impl Application {
    pub async fn new(engine: &mut Engine) -> Self {
        let camera =
            create_typed_bind_group(engine.renderer.as_mut(), FreeCameraBindGroup::default());
        let render_data = RenderDataBindGroup::new(engine.renderer.as_mut());
        let render_data_bind_group_layout = render_data.get_layout(engine.renderer.as_mut());

        let grass_shader = engine.renderer.create_shader(
            ShaderDescriptor::default()
                .with_descriptors(vec![Vertex::desc()])
                .with_bind_group_layouts(vec![
                    FreeCameraBindGroup::get_layout(),
                    render_data_bind_group_layout.clone(),
                    TextureBindGroup::get_layout(),
                    TextureBindGroup::get_layout(),
                ])
                .with_depth(true)
                .with_backface_culling(false)
                .with_multisample(MULTISAMPLE)
                .with_source(include_str!("terrain_shader.wgsl"))
                .with_fs_entry("fs_grass")
                .with_vs_entry("vs_grass"),
        );

        let ground_shader = engine.renderer.create_shader(
            ShaderDescriptor::default()
                .with_descriptors(vec![Vertex::desc(), Instance::desc()])
                .with_bind_group_layouts(vec![
                    FreeCameraBindGroup::get_layout(),
                    render_data_bind_group_layout.clone(),
                    TextureBindGroup::get_layout(),
                ])
                .with_backface_culling(false)
                .with_depth(true)
                .with_multisample(MULTISAMPLE)
                .with_source(include_str!("terrain_shader.wgsl"))
                .with_fs_entry("fs_ground")
                .with_vs_entry("vs_ground"),
        );

        let star_shader = engine.renderer.create_shader(
            ShaderDescriptor::default()
                .with_descriptors(vec![Vertex::desc()])
                .with_bind_group_layouts(vec![
                    FreeCameraBindGroup::get_layout(),
                    render_data_bind_group_layout.clone(),
                ])
                .with_backface_culling(true)
                .with_depth(true)
                .with_multisample(MULTISAMPLE)
                .with_source(include_str!("star_shader.wgsl")),
        );

        let render_data = create_typed_bind_group(engine.renderer.as_mut(), render_data);

        let grass_lod1 = GrassObject::from_text(
            include_str!("grass_lod1.obj"),
            engine.renderer.as_mut(),
            GRASS_LOD1_N,
        );
        let grass_lod2 = GrassObject::from_text(
            include_str!("grass_lod2.obj"),
            engine.renderer.as_mut(),
            GRASS_LOD2_N,
        );

        let ground = Object::from_obj(
            include_str!("plane.obj"),
            // include_str!("grass_lod1.obj"),
            engine.renderer.as_mut(),
            vec![Instance::default().scale(1000.0)],
        );

        let star_triangle = StarObject::new(engine.renderer.as_mut(), STAR_COUNT);

        let tex_sampler = engine.renderer.create_sampler(SamplerDescriptor {
            address_mode:
                jandering_engine::core::texture::sampler::SamplerAddressMode::RepeatMirrored,
            ..Default::default()
        });

        let heightmap_image = image::load_from_memory(include_bytes!("heightmap.jpg")).unwrap();
        let heightmap_handle = engine.renderer.create_texture(TextureDescriptor {
            data: Some(&heightmap_image.to_rgba8()),
            size: heightmap_image.dimensions().into(),
            ..Default::default()
        });
        let heightmap_texture =
            TextureBindGroup::new(engine.renderer.as_mut(), heightmap_handle, tex_sampler);
        let heightmap_texture =
            create_typed_bind_group(engine.renderer.as_mut(), heightmap_texture);

        let noise_image = image::load_from_memory(include_bytes!("bignoise.png")).unwrap();
        let noise_handle = engine.renderer.create_texture(TextureDescriptor {
            data: Some(&noise_image.to_rgba8()),
            size: noise_image.dimensions().into(),
            ..Default::default()
        });
        let noise_texture =
            TextureBindGroup::new(engine.renderer.as_mut(), noise_handle, tex_sampler);
        let noise_texture = create_typed_bind_group(engine.renderer.as_mut(), noise_texture);

        let multisample_texture = engine.renderer.create_texture(TextureDescriptor {
            size: engine.renderer.size(),
            sample_count: MULTISAMPLE,
            ..Default::default()
        });

        let depth_texture = engine.renderer.create_texture(TextureDescriptor {
            size: engine.renderer.size(),
            sample_count: MULTISAMPLE,
            format: TextureFormat::Depth32F,
            usage: texture_usage::TARGET | texture_usage::BIND,
            ..Default::default()
        });

        Self {
            last_time: web_time::Instant::now(),
            time: 0.0,
            fps_accumulator: 0.0,
            fps_counter: 0,
            grass_lod1,
            grass_lod2,
            grass_shader,
            ground,
            ground_shader,
            heightmap_texture,
            noise_texture,
            camera,
            render_data,
            is_in_fps: false,
            multisample_texture,
            depth_texture,
            star_triangle,
            star_shader,
        }
    }

    fn setup_grass_lod1(&mut self, renderer: &mut Box<dyn Renderer>) {
        let render_data = get_typed_bind_group_mut(renderer.as_mut(), self.render_data).unwrap();
        render_data.data.sqrt_n_grass = (GRASS_LOD1_N as f32).sqrt() as u32;
        render_data.data.render_square_size = GRASS_LOD1_SIDE;
        let render_data = get_typed_bind_group(renderer.as_ref(), self.render_data).unwrap();
        renderer.write_bind_group(self.render_data.into(), &render_data.get_data());
    }

    fn setup_grass_lod2(&mut self, renderer: &mut Box<dyn Renderer>) {
        let render_data = get_typed_bind_group_mut(renderer.as_mut(), self.render_data).unwrap();
        render_data.data.sqrt_n_grass = (GRASS_LOD2_N as f32).sqrt() as u32;
        render_data.data.render_square_size = GRASS_LOD2_SIDE;
        let render_data = get_typed_bind_group(renderer.as_ref(), self.render_data).unwrap();
        renderer.write_bind_group(self.render_data.into(), &render_data.get_data());
    }
}

impl EventHandler for Application {
    fn on_update(&mut self, context: &mut EngineContext) {
        let current_time = web_time::Instant::now();
        let dt = (current_time - self.last_time).as_secs_f32();
        self.last_time = current_time;
        self.time += dt;

        self.fps_accumulator += dt;
        self.fps_counter += 1;
        if self.fps_accumulator > 1.0 {
            println!(
                "fps: {}",
                1.0 / (self.fps_accumulator / self.fps_counter as f32)
            );
            self.fps_accumulator = 0.0;
            self.fps_counter = 0;
        }

        let render_data =
            get_typed_bind_group_mut(context.renderer.as_mut(), self.render_data).unwrap();
        render_data.data.time += dt;

        if self.is_in_fps {
            let resolution = context.renderer.size();
            let camera = get_typed_bind_group_mut(context.renderer.as_mut(), self.camera).unwrap();
            camera.update(context.events, context.window, resolution, dt);

            if context.events.iter().any(|e| {
                matches!(
                    e,
                    WindowEvent::KeyInput {
                        key: Key::Alt,
                        state: InputState::Pressed
                    }
                )
            }) {
                self.is_in_fps = false;
                context.window.set_cursor_visible(true);
            }
        } else if context.events.iter().any(|e| {
            matches!(
                e,
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: InputState::Pressed
                }
            )
        }) {
            self.is_in_fps = true;
            context.window.set_cursor_visible(false);
        }

        if context
            .events
            .iter()
            .any(|e| matches!(e, WindowEvent::Resized(_)))
        {
            context.renderer.re_create_texture(
                TextureDescriptor {
                    size: context.renderer.size(),
                    format: TextureFormat::Depth32F,
                    sample_count: MULTISAMPLE,
                    ..Default::default()
                },
                self.depth_texture,
            );

            context.renderer.re_create_texture(
                TextureDescriptor {
                    size: context.renderer.size(),
                    sample_count: MULTISAMPLE,
                    ..Default::default()
                },
                self.multisample_texture,
            );
        }

        if context.events.iter().any(|e| {
            matches!(
                e,
                WindowEvent::KeyInput {
                    key: Key::B,
                    state: InputState::Pressed
                }
            )
        }) {
            context.renderer.re_create_shaders();
        }
    }

    fn on_render(&mut self, renderer: &mut Box<dyn Renderer>) {
        let camera = get_typed_bind_group(renderer.as_ref(), self.camera).unwrap();
        renderer.write_bind_group(self.camera.into(), &camera.get_data());

        let sky_color = get_typed_bind_group(renderer.as_ref(), self.render_data)
            .unwrap()
            .data
            .sky_color;

        self.setup_grass_lod2(renderer);

        renderer
            .new_pass()
            .set_shader(self.star_shader)
            .with_target_texture_resolve(self.multisample_texture, None)
            .with_depth(self.depth_texture, Some(1.0))
            .with_clear_color(sky_color.x, sky_color.y, sky_color.z)
            .bind(0, self.camera.into())
            .bind(1, self.render_data.into())
            .render(&[&self.star_triangle])
            .set_shader(self.ground_shader)
            .bind(2, self.heightmap_texture.into())
            .render(&[&self.ground])
            .set_shader(self.grass_shader)
            .bind(3, self.noise_texture.into())
            .render(&[&self.grass_lod2])
            .submit();

        self.setup_grass_lod1(renderer);

        renderer
            .new_pass()
            .set_shader(self.grass_shader)
            .with_depth(self.depth_texture, None)
            .with_target_texture_resolve(self.multisample_texture, None)
            .bind(0, self.camera.into())
            .bind(1, self.render_data.into())
            .bind(2, self.heightmap_texture.into())
            .bind(3, self.noise_texture.into())
            .render(&[&self.grass_lod1])
            .submit();
    }
}

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

    buffer_handle: BufferHandle,
}

impl BindGroup for RenderDataBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        bytemuck::cast_slice(&[self.data]).into()
    }

    fn get_layout(&self, _renderer: &mut dyn Renderer) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(self.buffer_handle)],
        }
    }
}

impl RenderDataBindGroup {
    fn new(renderer: &mut dyn Renderer) -> Self {
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

fn main() {
    // let mut input = String::new();
    // std::io::stdin()
    //     .read_line(&mut input)
    //     .expect("error: unable to read user input");

    let mut engine = EngineBuilder::default()
        .with_window(
            WindowBuilder::default()
                .with_cursor(true)
                .with_resolution(1920, 1080)
                .with_title("Grass")
                .with_cursor(false),
        )
        .build();

    let app = pollster::block_on(Application::new(&mut engine));

    engine.run(app);
}
