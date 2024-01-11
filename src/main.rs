use jandering_engine::engine::Engine;
use winit::{
    event::{ElementState, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
};
fn main() {
    env_logger::init();

    let mut engine = Engine::default();

    engine.add_object(jandering_engine::object::primitives::triangle());

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
