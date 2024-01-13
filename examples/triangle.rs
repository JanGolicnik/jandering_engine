use jandering_engine::{engine::Engine, object::Instance};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};
fn main() {
    env_logger::init();

    let mut engine = Engine::default();

    engine.add_object(
        jandering_engine::object::primitives::triangle(),
        vec![Instance::default()],
    );

    let mut time = 0.0f64;

    engine.run(
        move |ref event, control_flow, _| match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {}
            _ => {}
        },
        move |_, camera, _, _, dt| {
            time += dt;
            camera.eye.x = (time * 2.0).sin() as f32;
        },
    );
}
