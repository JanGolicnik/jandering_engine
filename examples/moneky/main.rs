// use glam::Vec4Swizzles;
// use jandering_engine::{
//     bind_group::{
//         camera::free::{CameraController, FreeCameraController, MatrixCameraBindGroup},
//         BindGroup,
//     },
//     engine::{Engine, EngineConfig, EngineContext},
//     event_handler::EventHandler,
//     object::{Instance, Object, Vertex},
//     renderer::{BindGroupHandle, Janderer, Renderer, ShaderHandle, TextureHandle},
//     shader::ShaderDescriptor,
//     texture::{TextureDescriptor, TextureFormat},
//     types::Vec3,
//     window::{
//         Key, MouseButton, WindowConfig, WindowEvent, WindowHandle, WindowManager,
//         WindowManagerTrait, WindowTrait,
//     },
// };

// use rand::Rng;

// struct Application {
//     window_handle: WindowHandle,
//     last_time: web_time::Instant,
//     time: f32,
//     susane: Object<Instance>,
//     ground: Object<Instance>,
//     shader: ShaderHandle,
//     camera: BindGroupHandle<MatrixCameraBindGroup>,
//     is_in_fps: bool,
//     depth_texture: TextureHandle,

//     extra_windows: Vec<WindowHandle>,
// }

// const CAMERA_FOV: f32 = 45.0;
// const CAMEREA_NEAR: f32 = 0.01;
// const CAMEREA_FAR: f32 = 100000.0;

// impl Application {
//     fn new(engine: &mut Engine<Application>) -> Self {
//         let window_handle = {
//             let window_manager = engine.window_manager();

//             window_manager.create_window(
//                 WindowConfig::default()
//                     .with_cursor(true)
//                     .with_auto_resolution()
//                     .with_title("beast"),
//             )
//         };
//         let resolution = (1920, 1080);
//         let renderer = &mut engine.renderer;

//         let controller: Box<dyn CameraController> = Box::<FreeCameraController>::default();
//         let mut camera = MatrixCameraBindGroup::with_controller(renderer, controller);
//         camera.make_perspective(
//             CAMERA_FOV,
//             resolution.0 as f32 / resolution.1 as f32,
//             CAMEREA_NEAR,
//             CAMEREA_FAR,
//         );
//         let shader = renderer.create_shader(
//             ShaderDescriptor::default()
//                 .with_descriptors(vec![Vertex::desc(), Instance::desc()])
//                 .with_bind_group_layouts(vec![camera.get_layout()])
//                 .with_depth(true)
//                 .with_backface_culling(false),
//         );

//         let camera = renderer.create_typed_bind_group(camera);

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
//                     .rotate(angle.to_radians(), axis)
//                     .resize(Vec3::splat(1.0 + rand.gen::<f32>() * 10.0))
//                     .translate(pos)
//             })
//             .collect::<Vec<_>>();

//         let susane = Object::from_obj(include_str!("susane.obj"), renderer, susane_instances);

//         let ground = Object::from_obj(
//             include_str!("ground.obj"),
//             renderer,
//             vec![Instance::default()
//                 .translate(Vec3::new(0.0, -5.0, 0.0))
//                 .resize(Vec3::splat(0.05))],
//         );

//         let depth_texture = renderer.create_texture(TextureDescriptor {
//             size: resolution.into(),
//             format: TextureFormat::Depth32F,
//             ..Default::default()
//         });

//         Self {
//             window_handle,
//             last_time: web_time::Instant::now(),
//             time: 0.0,
//             susane,
//             ground,
//             shader,
//             camera,
//             is_in_fps: false,
//             depth_texture,

//             extra_windows: Vec::new(),
//         }
//     }
// }

// #[async_trait::async_trait]
// impl EventHandler for Application {
//     fn init(&mut self, renderer: &mut Renderer, window_manager: &mut WindowManager) {
//         renderer.register_window(self.window_handle, window_manager);

//         let resolution = window_manager
//             .get_window(self.window_handle)
//             .unwrap()
//             .size();

//         renderer.re_create_texture(
//             TextureDescriptor {
//                 size: resolution.into(),
//                 format: TextureFormat::Depth32F,
//                 ..Default::default()
//             },
//             self.depth_texture,
//         );

//         renderer
//             .get_typed_bind_group_mut(self.camera)
//             .unwrap()
//             .make_perspective(
//                 CAMERA_FOV,
//                 resolution.0 as f32 / resolution.1 as f32,
//                 CAMEREA_NEAR,
//                 CAMEREA_FAR,
//             );
//     }

//     fn on_update(&mut self, context: &mut EngineContext<'_>) {
//         let window = context
//             .window_manager
//             .get_window(context.window_handle)
//             .unwrap();

//         let current_time = web_time::Instant::now();
//         let dt = (current_time - self.last_time).as_secs_f32();
//         self.last_time = current_time;
//         self.time += dt;

//         if self.is_in_fps {
//             let camera = context
//                 .renderer
//                 .get_typed_bind_group_mut(self.camera)
//                 .unwrap();
//             camera.update(window.events(), dt);

//             if window.events().is_pressed(Key::Alt) {
//                 self.is_in_fps = false;
//                 window.set_cursor_visible(true);
//             }
//         } else if window.events().is_mouse_pressed(MouseButton::Left) {
//             self.is_in_fps = true;
//             window.set_cursor_visible(false);
//         }

//         if window
//             .events()
//             .iter()
//             .any(|e| matches!(e, WindowEvent::Resized(_)))
//         {
//             let resolution = window.size();
//             let camera = context
//                 .renderer
//                 .get_typed_bind_group_mut(self.camera)
//                 .unwrap();
//             camera.make_perspective(
//                 CAMERA_FOV,
//                 resolution.0 as f32 / resolution.1 as f32,
//                 CAMEREA_NEAR,
//                 CAMEREA_FAR,
//             );
//             context.renderer.re_create_texture(
//                 TextureDescriptor {
//                     size: window.size().into(),
//                     format: TextureFormat::Depth32F,
//                     ..Default::default()
//                 },
//                 self.depth_texture,
//             );
//         }

//         if window.events().is_pressed(Key::R) {
//             let handle = context.window_manager.create_window(
//                 WindowConfig::default()
//                     .with_cursor(true)
//                     .with_auto_resolution()
//                     .with_title("beast"),
//             );
//             context
//                 .renderer
//                 .register_window(handle, context.window_manager);
//             self.extra_windows.push(handle);
//         }

//         self.susane.instances.iter_mut().for_each(|e| {
//             let position = e.model.col(3).xyz();
//             let t = position.y / 1000.0;
//             *e = e.translate(Vec3::new(0.0, self.time.sin() * t * 100.0, 0.0));
//         });

//         self.susane.update(context.renderer);
//     }

//     fn on_render(&mut self, renderer: &mut Renderer, _: WindowHandle, _: &mut WindowManager) {
//         let camera = renderer.get_typed_bind_group_mut(self.camera).unwrap();
//         let data = camera.get_data();
//         renderer.write_bind_group(self.camera.into(), &data);

//         let mut pass = renderer.new_pass(self.window_handle);
//         pass.with_depth(self.depth_texture, Some(1.0))
//             .with_clear_color(0.2, 0.5, 1.0)
//             .set_shader(self.shader)
//             .bind(0, self.camera.into())
//             .render(&[&self.ground, &self.susane]);
//         pass.submit();

//         for handle in self.extra_windows.iter() {
//             let mut pass = renderer.new_pass(*handle);
//             pass.with_depth(self.depth_texture, Some(1.0))
//                 .with_clear_color(0.2, 0.5, 1.0)
//                 .set_shader(self.shader)
//                 .bind(0, self.camera.into())
//                 .render(&[&self.ground, &self.susane]);
//             pass.submit();
//         }
//     }
// }
// fn main() {
//     let mut engine = pollster::block_on(Engine::new(EngineConfig::default()));

//     let app = Application::new(&mut engine);

//     pollster::block_on(engine.run(app));
// }

fn main() {
    todo!()
}
