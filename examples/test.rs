use jandering_engine::{
    core::{
        bind_group::{camera::free::FreeCameraBindGroup, BindGroup},
        engine::{Engine, EngineBuilder, EngineContext},
        event_handler::EventHandler,
        object::{Instance, Object, Vertex},
        renderer::{BindGroupHandle, Renderer, ShaderHandle},
        shader::ShaderDescriptor,
        window::WindowBuilder,
    },
    types::{UVec2, Vec3},
    utils::load_obj_from_text,
};

struct Application {
    last_time: web_time::Instant,
    time: f32,
    susane: Object<Instance>,
    ground: Object<Instance>,
    shader: ShaderHandle,
    camera: BindGroupHandle<FreeCameraBindGroup>,
}

impl Application {
    pub async fn new(engine: &mut Engine) -> Self {
        let camera = engine
            .renderer
            .create_bind_group(FreeCameraBindGroup::default());

        let shader = engine.renderer.create_shader(
            ShaderDescriptor::default()
                .with_descriptors(&[Vertex::desc(), Instance::desc()])
                .with_bind_group_layouts(vec![FreeCameraBindGroup::layout()])
                .with_depth(true)
                .with_backface_culling(false),
        );

        let susane = load_obj_from_text(
            include_str!("susane.obj"),
            &engine.renderer,
            vec![Instance::default()],
        );

        let ground = load_obj_from_text(
            include_str!("ground.obj"),
            &engine.renderer,
            vec![Instance::with_position(Vec3::new(0.0, -1.0, 0.0))],
        );

        Self {
            last_time: web_time::Instant::now(),
            time: 0.0,
            susane,
            ground,
            shader,
            camera,
        }
    }
}

impl EventHandler for Application {
    fn on_update(&mut self, context: &mut EngineContext) {
        let current_time = web_time::Instant::now();
        let dt = (current_time - self.last_time).as_secs_f32();
        self.last_time = current_time;
        self.time += dt;

        context.renderer.clear_color.0 = (self.time).sin() * 0.5 + 0.5;
        let resolution = UVec2::new(context.renderer.width(), context.renderer.height());
        context
            .renderer
            .get_bind_group_t_mut(self.camera)
            .unwrap()
            .update(context.events, context.window, resolution, dt);
    }

    fn on_render(&mut self, renderer: &mut Box<Renderer>) {
        let camera = renderer.get_bind_group_t(self.camera).unwrap();
        renderer.write_bind_group(self.camera.into(), &camera.get_data());

        renderer
            .new_pass(self.shader)
            .bind(self.camera.into())
            .render(&[&self.ground, &self.susane]);
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
