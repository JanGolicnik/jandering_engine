use grass_object::GrassObject;
use jandering_engine::{
    core::{
        bind_group::{
            camera::free::FreeCameraBindGroup, texture::TextureBindGroup, BindGroup,
            BindGroupLayout, BindGroupLayoutEntry,
        },
        engine::{Engine, EngineBuilder, EngineContext},
        event_handler::EventHandler,
        object::{Instance, Object, Vertex},
        renderer::{BindGroupHandle, BufferHandle, Renderer, ShaderHandle, TextureHandle},
        shader::ShaderDescriptor,
        texture::{Texture, TextureDescriptor},
        window::{InputState, Key, MouseButton, WindowBuilder, WindowEvent},
    },
    types::Vec3,
};

struct Application {
    last_time: web_time::Instant,
    time: f32,
    fps_accumulator: f32,
    fps_counter: u32,
    grass_lod1: GrassObject,
    grass_lod2: GrassObject,
    grass_shader: ShaderHandle,
    ground: Object<Instance>,
    ground_shader: ShaderHandle,
    heightmap_texture: BindGroupHandle<TextureBindGroup>,
    noise_texture: BindGroupHandle<TextureBindGroup>,
    camera: BindGroupHandle<FreeCameraBindGroup>,
    render_data: BindGroupHandle<RenderDataBindGroup>,
    is_in_fps: bool,

    multisample_texture: TextureHandle,
    depth_texture: TextureHandle,
}

mod grass_object;

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
        let camera = engine
            .renderer
            .create_bind_group(FreeCameraBindGroup::default());

        let render_data = RenderDataBindGroup::new(&mut engine.renderer);
        let render_data_bind_group_layout = render_data.get_layout(&mut engine.renderer);

        let grass_shader = engine.renderer.create_shader(
            ShaderDescriptor::default()
                .with_descriptors(&[Vertex::desc()])
                .with_bind_group_layouts(vec![
                    FreeCameraBindGroup::get_layout(),
                    render_data_bind_group_layout.clone(),
                    TextureBindGroup::get_layout(),
                    TextureBindGroup::get_layout(),
                ])
                .with_depth(true)
                .with_backface_culling(false)
                .with_multisample(MULTISAMPLE)
                .with_source(include_str!("grass_shader.wgsl")),
        );

        let ground_shader = engine.renderer.create_shader(
            ShaderDescriptor::default()
                .with_descriptors(&[Vertex::desc(), Instance::desc()])
                .with_bind_group_layouts(vec![
                    FreeCameraBindGroup::get_layout(),
                    render_data_bind_group_layout.clone(),
                    TextureBindGroup::get_layout(),
                ])
                .with_backface_culling(false)
                .with_depth(true)
                .with_multisample(MULTISAMPLE)
                .with_source(include_str!("ground_shader.wgsl")),
        );

        let render_data = engine.renderer.create_bind_group(render_data);

        // let grass =
        let grass_lod1 = GrassObject::from_text(
            include_str!("grass_lod1.obj"),
            &mut engine.renderer,
            GRASS_LOD1_N,
        );
        let grass_lod2 = GrassObject::from_text(
            include_str!("grass_lod2.obj"),
            &mut engine.renderer,
            GRASS_LOD2_N,
        );

        let ground = Object::from_obj(
            include_str!("plane.obj"),
            &mut engine.renderer,
            vec![Instance::default().scale(1000.0)],
        );

        let heightmap_handle = engine.renderer.add_texture(Texture::from_bytes(
            &engine.renderer,
            include_bytes!("heightmap.jpg"),
            TextureDescriptor::default(),
        ));
        let heightmap_texture = TextureBindGroup::new(&mut engine.renderer, heightmap_handle);
        let heightmap_texture = engine.renderer.create_bind_group(heightmap_texture);

        let noise_handle = engine.renderer.add_texture(Texture::from_bytes(
            &engine.renderer,
            include_bytes!("bignoise.png"),
            TextureDescriptor {
                address_mode: wgpu::AddressMode::Repeat,
                ..Default::default()
            },
        ));
        let noise_texture = TextureBindGroup::new(&mut engine.renderer, noise_handle);
        let noise_texture = engine.renderer.create_bind_group(noise_texture);

        let multisample_texture = engine.renderer.create_texture(TextureDescriptor {
            size: engine.renderer.size(),
            sample_count: MULTISAMPLE,
            ..Default::default()
        });

        let depth_texture = engine.renderer.create_texture(TextureDescriptor {
            size: engine.renderer.size(),
            sample_count: MULTISAMPLE,
            format: wgpu::TextureFormat::Depth32Float,
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
        }
    }

    fn setup_grass_lod1(&mut self, renderer: &mut Box<Renderer>) {
        let render_data = renderer.get_bind_group_t_mut(self.render_data).unwrap();
        render_data.data.sqrt_n_grass = (GRASS_LOD1_N as f32).sqrt() as u32;
        render_data.data.render_square_size = GRASS_LOD1_SIDE;
        let render_data = renderer.get_bind_group_t(self.render_data).unwrap();
        renderer.write_bind_group(self.render_data.into(), &render_data.get_data());
    }

    fn setup_grass_lod2(&mut self, renderer: &mut Box<Renderer>) {
        let render_data = renderer.get_bind_group_t_mut(self.render_data).unwrap();
        render_data.data.sqrt_n_grass = (GRASS_LOD2_N as f32).sqrt() as u32;
        render_data.data.render_square_size = GRASS_LOD2_SIDE;
        let render_data = renderer.get_bind_group_t(self.render_data).unwrap();
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

        let render_data = context
            .renderer
            .get_bind_group_t_mut(self.render_data)
            .unwrap();
        render_data.data.time += dt;

        if self.is_in_fps {
            let resolution = context.renderer.size();
            let camera = context.renderer.get_bind_group_t_mut(self.camera).unwrap();
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
                    format: wgpu::TextureFormat::Depth32Float,
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
    }

    fn on_render(&mut self, renderer: &mut Box<Renderer>) {
        let camera = renderer.get_bind_group_t(self.camera).unwrap();
        renderer.write_bind_group(self.camera.into(), &camera.get_data());

        self.setup_grass_lod2(renderer);

        renderer
            .new_pass()
            .set_shader(self.ground_shader)
            .with_target_texture_resolve(self.multisample_texture, None)
            .with_depth(self.depth_texture, Some(1.0))
            .with_clear_color(0.2, 0.5, 1.0)
            .bind(0, self.camera.into())
            .bind(1, self.render_data.into())
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

fn main() {
    let mut engine = EngineBuilder::default()
        .with_window(
            WindowBuilder::default()
                .with_cursor(true)
                .with_resolution(1000, 1000)
                .with_title("heyy")
                .with_cursor(false),
        )
        .with_clear_color(0.9, 0.8, 0.7)
        .build();

    let app = pollster::block_on(Application::new(&mut engine));

    engine.run(app);
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

pub struct RenderDataData {
    ground_color: Vec3,
    time: f32,
    grass_top_color: Vec3,
    grass_height: f32,
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
    padding: [f32; 1],
}

pub struct RenderDataBindGroup {
    pub data: RenderDataData,

    buffer_handle: BufferHandle,
}

impl BindGroup for RenderDataBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        bytemuck::cast_slice(&[self.data]).into()
    }

    fn get_layout(&self, _renderer: &mut Renderer) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(self.buffer_handle)],
        }
    }
}

impl RenderDataBindGroup {
    fn new(renderer: &mut Renderer) -> Self {
        let data = RenderDataData {
            time: 0.0,
            ground_color: Vec3::new(0.1, 0.4, 0.2),
            grass_top_color: Vec3::new(0.5, 1.0, 0.6),
            grass_height: 1.0,
            grass_height_variation: 0.5,
            wind_strength: 0.2,
            wind_scale: 200.0,
            wind_speed: 5.0,
            wind_direction: 0.0,
            wind_noise_scale: 2.0,
            wind_noise_strength: 0.1,
            // sqrt_n_grass: (10_000_000.0f32.sqrt()) as u32,
            sqrt_n_grass: 0,
            terrain_size: 1000.0,
            render_square_size: 100.0,
            fov_x: 45.0,
            padding: [0.0; 1],
        };

        let buffer_handle = renderer.create_uniform_buffer(bytemuck::cast_slice(&[data]));

        Self {
            data,
            buffer_handle,
        }
    }
}
