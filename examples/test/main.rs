use jandering_engine::{
    bind_group::{
        camera::free::{CameraController, FreeCameraController, MatrixCameraBindGroup},
        BindGroup,
    },
    engine::{Engine, EngineContext},
    event_handler::EventHandler,
    object::{Instance, Object, Vertex},
    render_pass::RenderPassTrait,
    renderer::{BindGroupHandle, Janderer, Renderer, ShaderHandle, TextureHandle},
    shader::ShaderDescriptor,
    texture::{TextureDescriptor, TextureFormat},
    types::{UVec2, Vec3},
    window::{WindowConfig, WindowEvent, WindowHandle, WindowManagerTrait, WindowTrait},
};

struct Application {
    window_handle: WindowHandle,
    cube: Object<Instance>,
    shader: ShaderHandle,
    camera: BindGroupHandle<MatrixCameraBindGroup>,
    depth_texture: TextureHandle,

    last_time: web_time::Instant,
}

const CAMERA_FOV: f32 = 45.0;
const CAMEREA_NEAR: f32 = 0.01;
const CAMEREA_FAR: f32 = 100000.0;

impl Application {
    fn new(engine: &mut Engine<Application>) -> Self {
        let resolution = 250;
        let window_handle = {
            let window_manager = engine.window_manager();

            window_manager.create_window(
                WindowConfig::default()
                    .with_cursor(true)
                    .with_auto_resolution()
                    .with_decorations(false)
                    .with_transparency(true)
                    .with_title("beast"),
            )
        };
        let renderer = &mut engine.renderer;

        let controller: Box<dyn CameraController> = Box::<FreeCameraController>::default();
        let mut camera = MatrixCameraBindGroup::with_controller(controller);
        camera.make_perspective(CAMERA_FOV, 1.0, CAMEREA_NEAR, CAMEREA_FAR);
        let camera = renderer.create_typed_bind_group(camera);

        let shader = renderer.create_shader(
            ShaderDescriptor::default()
                .with_descriptors(vec![Vertex::desc(), Instance::desc()])
                .with_bind_group_layouts(vec![MatrixCameraBindGroup::get_layout()])
                .with_depth(true)
                .with_backface_culling(false),
        );

        let cube = Object::from_obj(
            include_str!("cube.obj"),
            renderer,
            vec![Instance::default()],
        );

        let depth_texture = renderer.create_texture(TextureDescriptor {
            size: UVec2::splat(resolution),
            format: TextureFormat::Depth32F,
            ..Default::default()
        });

        Self {
            window_handle,
            cube,
            shader,
            camera,
            depth_texture,

            last_time: web_time::Instant::now(),
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for Application {
    fn init(&mut self, context: &mut EngineContext<'_>) {
        context
            .renderer
            .register_window(self.window_handle, context.window_manager);

        let resolution = context
            .window_manager
            .get_window(self.window_handle)
            .unwrap()
            .size();

        context.renderer.re_create_texture(
            TextureDescriptor {
                size: resolution.into(),
                format: TextureFormat::Depth32F,
                ..Default::default()
            },
            self.depth_texture,
        );

        context
            .renderer
            .get_typed_bind_group_mut(self.camera)
            .unwrap()
            .make_perspective(
                CAMERA_FOV,
                resolution.0 as f32 / resolution.1 as f32,
                CAMEREA_NEAR,
                CAMEREA_FAR,
            );
    }

    fn on_update(&mut self, context: &mut EngineContext<'_>) {
        let window = context
            .window_manager
            .get_window(context.window_handle)
            .unwrap();

        let current_time = web_time::Instant::now();
        let dt = (current_time - self.last_time).as_secs_f32();
        self.last_time = current_time;

        let camera = context
            .renderer
            .get_typed_bind_group_mut(self.camera)
            .unwrap();
        camera.update(context.events, dt);

        if context
            .events
            .iter()
            .any(|e| matches!(e, WindowEvent::Resized(_)))
        {
            let resolution = window.size();
            let camera = context
                .renderer
                .get_typed_bind_group_mut(self.camera)
                .unwrap();
            camera.make_perspective(
                CAMERA_FOV,
                resolution.0 as f32 / resolution.1 as f32,
                CAMEREA_NEAR,
                CAMEREA_FAR,
            );
            context.renderer.re_create_texture(
                TextureDescriptor {
                    size: window.size().into(),
                    format: TextureFormat::Depth32F,
                    ..Default::default()
                },
                self.depth_texture,
            );
        }

        self.cube.instances.iter_mut().for_each(|e| {
            e.rotate(20.0f32.to_radians() * dt, Vec3::Y);
        });

        self.cube.update(context.renderer);
    }

    fn on_render(&mut self, renderer: &mut Renderer) {
        let camera = renderer.get_typed_bind_group_mut(self.camera).unwrap();
        let data = camera.get_data();
        renderer.write_bind_group(self.camera.into(), &data);

        renderer
            .new_pass(self.window_handle)
            .with_depth(self.depth_texture, Some(1.0))
            .with_alpha(0.001)
            .set_shader(self.shader)
            .bind(0, self.camera.into())
            .render(&[&self.cube])
            .submit();
    }
}
fn main() {
    let mut engine = pollster::block_on(Engine::new());

    let app = Application::new(&mut engine);

    pollster::block_on(engine.run(app));
}
