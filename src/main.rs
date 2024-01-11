use cgmath::ElementWise;
use jandering_engine::{camera, engine::Engine};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};
fn main() {
    env_logger::init();

    let mut engine = Engine::default();

    engine.add_object(jandering_engine::object::primitives::triangle());

    let mut frame = 0;

    engine.run(move |ref event, control_flow, camera| match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                },
            ..
        } => {
            camera.eye.x = (frame as f32 / 10.0).sin();
            frame += 1;
        }
        _ => {}
    });
}
