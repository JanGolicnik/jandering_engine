use super::{Object, Vertex};

pub fn triangle() -> Object {
    Object {
        vertices: vec![
            Vertex {
                position: [0.0, 1.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ],
        scale: cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        rotation: cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        position: cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        indices: vec![0, 1, 2],
        pipeline: None,
    }
}
