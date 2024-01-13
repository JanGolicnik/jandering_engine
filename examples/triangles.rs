use jandering_engine::{engine::Engine, object::Instance};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};
fn main() {
    env_logger::init();

    let mut engine = Engine::default();

    let instances = (-10..10)
        .flat_map(|z| {
            (-10..10).map(move |x| Instance {
                position: Some(cgmath::Vector3 {
                    x: x as f32,
                    y: 0.0,
                    z: z as f32,
                }),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();

    engine.add_object(jandering_engine::object::primitives::triangle(), instances);
    {
        let cam = engine.get_camera();
        cam.eye.y = 5.0;
        cam.eye.z = 10.0;
    }

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
        move |_, _, _, _, _| {},
    );
}
