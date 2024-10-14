use crate::utils::SQRT_3;

use super::{Vec2, Vec3, Vertex};

pub fn triangle_data() -> (Vec<Vertex>, Vec<u32>) {
    let height = SQRT_3 * 0.5;
    let center = 1.0 - SQRT_3 / 3.0;

    let vertices = vec![
        Vertex {
            position: Vec3::new(0.0, center + height, 0.0),
            normal: Vec3::NEG_Z,
            uv: Vec2::new(1.0, 1.0),
        },
        Vertex {
            position: Vec3::new(1.0, center - 1.0, 0.0),
            normal: Vec3::NEG_Z,
            uv: Vec2::new(1.0, 0.0),
        },
        Vertex {
            position: Vec3::new(-1.0, center - 1.0, 0.0),
            normal: Vec3::NEG_Z,
            uv: Vec2::new(0.0, 1.0),
        },
    ];

    let indices = vec![0, 1, 2];
    (vertices, indices)
}

pub fn quad_data() -> (Vec<Vertex>, Vec<u32>) {
    let vertices = vec![
        Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
            uv: Vec2::new(0.0, 1.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
        },
        Vertex {
            position: Vec3::new(0.0, 1.0, 0.0),
            uv: Vec2::new(0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
        },
        Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
            uv: Vec2::new(1.0, 1.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
        },
        Vertex {
            position: Vec3::new(1.0, 1.0, 0.0),
            uv: Vec2::new(1.0, 0.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
        },
    ];

    let indices = vec![0, 3, 2, 0, 1, 3];

    (vertices, indices)
}

// pub fn circle<T>(renderer: &Renderer, instances: Vec<T>, resolution: u32) -> Object<T>
// where
//     T: bytemuck::Pod,
// {
//     let anglestep = (2.0 * std::f32::consts::PI) / resolution as f32;

//     let mut vertices: Vec<VertexRaw> = vec![VertexRaw {
//         position: Vec3::new(0.0, 0.0, 0.0),
//         uv: Vec2::new(0.0, 0.0),
//     }];

//     let mut indices: Vec<u32> = Vec::new();

//     (0..resolution).for_each(|i| {
//         let vec = Vec3::X;

//         let sina = (anglestep * i as f32).sin();
//         let cosa = (anglestep * i as f32).cos();

//         let position = Vec3::new(
//             vec.x * cosa - vec.y * sina,
//             vec.x * sina + vec.y * cosa,
//             0.0,
//         );

//         let this_vertex = vertices.len() as u32;
//         let next_vertex = if this_vertex == resolution {
//             1
//         } else {
//             this_vertex + 1
//         };
//         let mut this_indices = vec![0, this_vertex, next_vertex];

//         let vertex = VertexRaw {
//             position,
//             uv: Vec2::new(0.0, 0.0),
//         };

//         indices.append(&mut this_indices);
//         vertices.push(vertex);
//     });

//     let vertex_buffer = renderer
//         .device
//         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Vertex Buffer"),
//             contents: bytemuck::cast_slice(&vertices),
//             usage: wgpu::BufferUsages::VERTEX,
//         });
//     let index_buffer = renderer
//         .device
//         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(&indices),
//             usage: wgpu::BufferUsages::INDEX,
//         });
//     let instance_buffer = renderer
//         .device
//         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Instance Buffer"),
//             contents: bytemuck::cast_slice(&instances),
//             usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//         });
//     let previous_instances_len = instances.len();
//     Object {
//         vertices,
//         indices,
//         instances,
//         render_data: Some(super::ObjectRenderData {
//             vertex_buffer,
//             index_buffer,
//             instance_buffer,
//         }),
//         previous_instances_len,
//     }
// }
