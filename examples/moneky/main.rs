// use glam::Vec4Swizzles;
// use jandering_engine::{
//     core::{
//         bind_group::{
//             camera::free::{CameraController, FreeCameraController, MatrixCameraBindGroup},
//             BindGroup,
//         },
//         engine::{Engine, EngineBuilder, EngineContext},
//         event_handler::EventHandler,
//         object::{Instance, Object, Vertex},
//         renderer::{
//             create_typed_bind_group, get_typed_bind_group, get_typed_bind_group_mut,
//             BindGroupHandle, Renderer, ShaderHandle, TextureHandle,
//         },
//         shader::ShaderDescriptor,
//         texture::{TextureDescriptor, TextureFormat},
//         window::{InputState, Key, MouseButton, WindowEvent},
//     },
//     types::Vec3,
// };

// use rand::Rng;

// struct Application {
//     last_time: web_time::Instant,
//     time: f32,
//     susane: Object<Instance>,
//     ground: Object<Instance>,
//     shader: ShaderHandle,
//     camera: BindGroupHandle<MatrixCameraBindGroup>,
//     is_in_fps: bool,
//     depth_texture: TextureHandle,
// }

// const CAMERA_FOV: f32 = 45.0;
// const CAMEREA_NEAR: f32 = 0.01;
// const CAMEREA_FAR: f32 = 100000.0;

// impl Application {
//     pub async fn new(engine: &mut Engine) -> Self {
//         let resolution = engine.renderer.size();
//         let controller: Box<dyn CameraController> = Box::<FreeCameraController>::default();
//         let mut camera = MatrixCameraBindGroup::with_controller(controller);
//         camera.make_perspective(
//             CAMERA_FOV,
//             resolution.x as f32 / resolution.y as f32,
//             CAMEREA_NEAR,
//             CAMEREA_FAR,
//         );
//         let camera = create_typed_bind_group(engine.renderer.as_mut(), camera);

//         let shader = engine.renderer.create_shader(
//             ShaderDescriptor::default()
//                 .with_descriptors(vec![Vertex::desc(), Instance::desc()])
//                 .with_bind_group_layouts(vec![MatrixCameraBindGroup::get_layout()])
//                 .with_depth(true)
//                 .with_backface_culling(false),
//         );

//         let mut rand = rand::thread_rng();

//         let susane_instances = (0..10_000)
//             .map(|_| {
//                 let height = (rand.gen::<f32>() as f32).powf(30.0) * 1000.0;
//                 let pos = Vec3::new(
//                     rand.gen::<f32>() * 500.0 - 250.0,
//                     height,
//                     rand.gen::<f32>() * 500.0 - 250.0,
//                 );
//                 let axis = Vec3::new(
//                     rand.gen::<f32>() - 0.5,
//                     rand.gen::<f32>() - 0.5,
//                     rand.gen::<f32>() - 0.5,
//                 );
//                 let angle = rand.gen::<f32>() * 360.0;
//                 Instance::default()
//                     .rotate(angle * DEG_TO_RAD, axis)
//                     .scale(1.0 + rand.gen::<f32>() * 10.0)
//                     .translate(pos)
//             })
//             .collect::<Vec<_>>();

//         let susane = Object::from_obj(
//             include_str!("susane.obj"),
//             engine.renderer.as_mut(),
//             susane_instances,
//         );

//         let ground = Object::from_obj(
//             include_str!("ground.obj"),
//             engine.renderer.as_mut(),
//             vec![Instance::default()
//                 .translate(Vec3::new(0.0, -5.0, 0.0))
//                 .scale(0.05)],
//         );

//         let depth_texture = engine.renderer.create_texture(TextureDescriptor {
//             size: engine.renderer.size(),
//             format: TextureFormat::Depth32F,
//             ..Default::default()
//         });

//         Self {
//             last_time: web_time::Instant::now(),
//             time: 0.0,
//             susane,
//             ground,
//             shader,
//             camera,
//             is_in_fps: false,
//             depth_texture,
//         }
//     }
// }

// impl EventHandler for Application {
//     fn on_update(&mut self, context: &mut EngineContext) {
//         let current_time = web_time::Instant::now();
//         let dt = (current_time - self.last_time).as_secs_f32();
//         self.last_time = current_time;
//         self.time += dt;

//         if self.is_in_fps {
//             let camera = get_typed_bind_group_mut(context.renderer.as_mut(), self.camera).unwrap();
//             camera.update(context.events, dt);

//             if context.events.iter().any(|e| {
//                 matches!(
//                     e,
//                     WindowEvent::KeyInput {
//                         key: Key::Alt,
//                         state: InputState::Pressed
//                     }
//                 )
//             }) {
//                 self.is_in_fps = false;
//                 context.window.set_cursor_visible(true);
//             }
//         } else if context.events.iter().any(|e| {
//             matches!(
//                 e,
//                 WindowEvent::MouseInput {
//                     button: MouseButton::Left,
//                     state: InputState::Pressed
//                 }
//             )
//         }) {
//             self.is_in_fps = true;
//             context.window.set_cursor_visible(false);
//         }

//         if context
//             .events
//             .iter()
//             .any(|e| matches!(e, WindowEvent::Resized(_)))
//         {
//             let resolution = context.renderer.size();
//             let camera = get_typed_bind_group_mut(context.renderer.as_mut(), self.camera).unwrap();
//             camera.make_perspective(
//                 CAMERA_FOV,
//                 resolution.x as f32 / resolution.y as f32,
//                 CAMEREA_NEAR,
//                 CAMEREA_FAR,
//             );
//             context.renderer.re_create_texture(
//                 TextureDescriptor {
//                     size: context.renderer.size(),
//                     format: TextureFormat::Depth32F,
//                     ..Default::default()
//                 },
//                 self.depth_texture,
//             );
//         }

//         self.susane.instances.iter_mut().for_each(|e| {
//             let position = e.model.col(3).xyz();
//             let t = position.y / 1000.0;
//             *e = e.translate(Vec3::new(0.0, self.time.sin() * t * 100.0, 0.0));
//         });

//         self.susane.update(context.renderer.as_mut());
//     }

//     fn on_render(&mut self, renderer: &mut Box<dyn Renderer>) {
//         let camera = get_typed_bind_group(renderer.as_ref(), self.camera).unwrap();
//         renderer.write_bind_group(self.camera.into(), &camera.get_data());

//         renderer
//             .new_pass()
//             .with_depth(self.depth_texture, Some(1.0))
//             .with_clear_color(0.2, 0.5, 1.0)
//             .set_shader(self.shader)
//             .bind(0, self.camera.into())
//             .render(&[&self.ground, &self.susane])
//             .submit();
//     }
// }

// fn main() {
//     let builder = EngineBuilder::default().with_window(
//         WindowBuilder::default()
//             .with_cursor(true)
//             .with_resolution(1000, 1000)
//             .with_title("heyy")
//             .with_cursor(false),
//     );
//     let mut engine = pollster::block_on(builder.build());
//     let app = pollster::block_on(Application::new(&mut engine));

//     engine.run(app);
// }

use jandering_engine::core::{
    engine::{EngineBuilder, EngineContext, EngineNew},
    event_handler::EventHandler,
    renderer::Renderer,
    window::{WindowConfig, WindowEvent},
};

struct Application {}

impl EngineNew for Application {
    fn engine_new<T: EventHandler + 'static + EngineNew>(
        engine: &mut jandering_engine::core::engine::Engine<T>,
    ) -> Self {
        Self {}
    }
}

impl EventHandler for Application {
    fn on_update(&mut self, engine: &mut EngineContext<'_>) {
        println!("onupdate");
        if engine
            .events
            .matches(|e| matches!(e, WindowEvent::CloseRequested))
        {
            engine.window.close();
        }
    }

    fn on_render(&mut self, renderer: &mut Box<dyn Renderer>) {}
}
fn main() {
    pollster::block_on(
        EngineBuilder::default()
            .with_window(
                WindowConfig::default()
                    .with_cursor(true)
                    .with_auto_resolution()
                    .with_title("beast"),
            )
            .run::<Application>(),
    );
}
