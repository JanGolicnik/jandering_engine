use jandering_engine::engine::Engine;
use winit::{
    event::{ElementState, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
};
fn main() {
    env_logger::init();

    let engine = Engine::default();

    engine.run(move |ref event, control_flow| match event {
        WindowEvent::CloseRequested
        | WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => *control_flow = ControlFlow::Exit,
        _ => {}
    });
}
