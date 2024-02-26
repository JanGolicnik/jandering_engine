use cgmath::InnerSpace;
use instance::D2ColorInstance;
use jandering_engine::{
    bind_group::camera::d2::D2CameraBindGroup,
    engine::EngineDescriptor,
    object::{Object, VertexRaw},
    shader::ShaderDescriptor,
    types::{UVec2, Vec2, Vec3},
};
use wasm_bindgen::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent};

mod instance;

const DOT_RADIUS: f32 = 20.0;
const LINE_WIDTH: f32 = DOT_RADIUS * 0.2;
const CONNECTED_DISTANCE: f32 = DOT_RADIUS * 3.0;
const MINIMUM_DISTANCE: f32 = DOT_RADIUS * 4.0;
const CIRCLE_COLOR: Vec3 = Vec3::new(1.0, 0.7, 0.6);
const SELECTED_COLOR: Vec3 = Vec3::new(0.7, 1.0, 0.6);
const SPRINGINESS: f32 = 3.0;

struct Node {
    position: Vec2,
    next: Option<usize>,
}

#[wasm_bindgen(start)]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Coultn init");
    let mut engine = jandering_engine::engine::Engine::new(EngineDescriptor::default());

    let mut camera_bind_group = D2CameraBindGroup::new(&engine.renderer, true);
    camera_bind_group.right_click_move = true;
    let camera_bg = engine.renderer.add_bind_group(camera_bind_group);

    let untyped_bind_groups = [camera_bg.into()];
    let shader = jandering_engine::shader::create_shader(
        &mut engine.renderer,
        ShaderDescriptor {
            code: include_str!("shader.wgsl"),
            descriptors: &[VertexRaw::desc(), D2ColorInstance::desc()],
            bind_groups: &untyped_bind_groups,
            backface_culling: false,
            ..Default::default()
        },
    );
    let mut nodes: Vec<Node> = Vec::new();

    let mut circles: Object<D2ColorInstance> =
        jandering_engine::object::primitives::circle(&engine.renderer, vec![], 16);

    let mut lines: Object<D2ColorInstance> =
        jandering_engine::object::primitives::quad(&engine.renderer, vec![]);

    let mut connecting_node: Option<usize> = None;
    let mut held_node = None;
    let mut mouse_pos: Option<Vec2> = None;

    engine.run(move |context, renderer| {
        for event in context.events {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    let camera_bind_group = renderer.get_bind_group_t(camera_bg).unwrap();
                    mouse_pos = Some(
                        camera_bind_group
                            .mouse_to_world(Vec2::new(position.x as f32, position.y as f32)),
                    );
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => {
                    if let Some(position) = mouse_pos {
                        if let Some((index, _)) = clicked_node(&mut nodes, position) {
                            match state {
                                ElementState::Pressed => held_node = Some(index),
                                ElementState::Released => held_node = None,
                            }
                        } else {
                            nodes.push(Node {
                                position,
                                next: None,
                            });
                        }
                    }
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Middle,
                    state,
                    ..
                } => {
                    if let Some(position) = mouse_pos {
                        if let Some((index, _)) = clicked_node(&mut nodes, position) {
                            match state {
                                ElementState::Pressed => connecting_node = Some(index),
                                ElementState::Released => {
                                    if connecting_node.is_some_and(|e| e != index) {
                                        nodes[connecting_node.unwrap()].next = Some(index);
                                    }
                                    connecting_node = None
                                }
                            }
                        } else if let Some(node) = connecting_node {
                            nodes[node].next = None;
                            connecting_node = None
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(mouse_pos) = mouse_pos {
            if let Some(node) = held_node {
                nodes[node].position = mouse_pos;
            }
        }

        update_nodes(&mut nodes, context.dt as f32);

        if let Some(mouse_pos) = mouse_pos {
            if let Some(node) = held_node {
                nodes[node].position = mouse_pos;
            }
        }

        let mut used_lines = 0;
        for (current_circle, node) in nodes.iter_mut().enumerate() {
            if let Some(circle) = circles.instances.get_mut(current_circle) {
                circle.position = node.position;
            } else {
                circles.instances.push(D2ColorInstance {
                    position: node.position,
                    scale: Vec2::new(DOT_RADIUS, DOT_RADIUS),
                    rotation: 0.0,
                    color: CIRCLE_COLOR,
                });
            }
            let connected_pos = if connecting_node.is_some_and(|val| val == current_circle) {
                mouse_pos
            } else {
                match node.next {
                    Some(i) => Some(circles.instances[i].position),
                    None => None,
                }
            };
            if let Some(target) = connected_pos {
                if let Some(line) = lines.instances.get_mut(used_lines) {
                    update_line_instance(line, node.position, target);
                } else {
                    lines
                        .instances
                        .push(make_line_instance(node.position, target, LINE_WIDTH))
                }
                used_lines += 1;
            }

            circles.instances[current_circle].color = SELECTED_COLOR;
        }

        if let Some(Some(circle)) = connecting_node.map(|val| circles.instances.get_mut(val)) {
            circle.color = CIRCLE_COLOR;
        }

        circles.instances.truncate(nodes.len());
        lines.instances.truncate(used_lines);

        circles.update(context, renderer);
        lines.update(context, renderer);

        let resolution = UVec2::new(renderer.config.width, renderer.config.height);
        let camera_bind_group = renderer.get_bind_group_t_mut(camera_bg).unwrap();
        camera_bind_group.update(context, resolution);

        renderer.render(&[&lines, &circles], context, &shader, &untyped_bind_groups);
    });
}

fn update_nodes(nodes: &mut [Node], dt: f32) {
    let mut i = 0;
    while i < nodes.len() {
        let mut j = 0;
        while j < nodes.len() {
            if i == j {
                j += 1;
                continue;
            }
            let other = nodes.get(j).unwrap();

            let is_connected = nodes[i].next.is_some_and(|val| j == val);

            let direction = other.position - nodes[i].position;
            let distance = direction.magnitude();
            let diff = if is_connected {
                CONNECTED_DISTANCE - distance
            } else if distance > MINIMUM_DISTANCE {
                0.0
            } else {
                MINIMUM_DISTANCE - distance
            } * 0.5;

            let move_vec = direction.normalize() * diff;

            nodes[i].position -= move_vec * dt * SPRINGINESS;
            nodes[j].position += move_vec * dt * SPRINGINESS;

            j += 1;
        }

        i += 1;
    }
}

fn make_line_instance(from: Vec2, to: Vec2, line_width: f32) -> D2ColorInstance {
    let diff: Vec2 = to - from;
    let unit_x: Vec2 = cgmath::Vector2::<f32>::unit_x();
    let position = from + diff * 0.5;
    let rotation = unit_x.angle(diff);
    D2ColorInstance {
        position,
        scale: Vec2::new(diff.magnitude(), line_width),
        rotation: rotation.0,
        color: Vec3::new(1.0, 0.7, 0.6),
    }
}

fn update_line_instance(instance: &mut D2ColorInstance, from: Vec2, to: Vec2) {
    let diff: Vec2 = to - from;
    let unit_x: Vec2 = cgmath::Vector2::<f32>::unit_x();
    let position = from + diff * 0.5;
    let rotation = unit_x.angle(diff);
    instance.position = position;
    instance.rotation = rotation.0;
    instance.scale.x = diff.magnitude();
}

fn clicked_node(nodes: &mut [Node], mouse_pos: Vec2) -> Option<(usize, &Node)> {
    nodes
        .iter()
        .enumerate()
        .find(|(_, node)| (node.position - mouse_pos).magnitude() < DOT_RADIUS)
}
