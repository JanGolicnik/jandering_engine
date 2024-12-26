use primitives::plane_data;

use crate::{
    renderer::{BufferHandle, Janderer},
    shader::BufferLayoutEntryDataType,
    types::*,
    utils::load_obj,
};

use self::primitives::{quad_data, triangle_data};

use super::{
    renderer::Renderer,
    shader::{BufferLayout, BufferLayoutEntry},
};

pub mod primitives;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Debug)]
pub struct ObjectRenderData {
    pub vertex_buffer: BufferHandle,
    //
    pub index_buffer: BufferHandle,
    //
    pub instance_buffer: BufferHandle,
}

pub struct Object<T> {
    pub vertices: Vec<Vertex>,
    //
    pub indices: Vec<u32>,
    //
    pub instances: Vec<T>,
    //
    pub render_data: ObjectRenderData,

    previous_instances_len: usize,
}

impl Vertex {
    pub fn desc() -> BufferLayout {
        BufferLayout {
            step_mode: crate::shader::BufferLayoutStepMode::Vertex,
            stride: std::mem::size_of::<Vertex>(),
            entries: &[
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Vertex, position) as u64,
                    location: 0,
                    data_type: BufferLayoutEntryDataType::Float32x3,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Vertex, normal) as u64,
                    location: 1,
                    data_type: BufferLayoutEntryDataType::Float32x3,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Vertex, uv) as u64,
                    location: 2,
                    data_type: BufferLayoutEntryDataType::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Debug, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model: Mat4,
    pub inv_model: Mat4,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            model: Mat4::IDENTITY,
            inv_model: Mat4::IDENTITY,
        }
    }
}

impl Instance {
    pub fn desc() -> BufferLayout {
        BufferLayout {
            step_mode: crate::shader::BufferLayoutStepMode::Instance,
            stride: std::mem::size_of::<Instance>(),
            entries: &[
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, model) as u64,
                    location: 5,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, model) as u64 + 16,
                    location: 6,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, model) as u64 + 32,
                    location: 7,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, model) as u64 + 48,
                    location: 8,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, inv_model) as u64,
                    location: 9,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, inv_model) as u64 + 16,
                    location: 10,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, inv_model) as u64 + 32,
                    location: 11,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
                BufferLayoutEntry {
                    offset: std::mem::offset_of!(Instance, inv_model) as u64 + 48,
                    location: 12,
                    data_type: BufferLayoutEntryDataType::Float32x4,
                },
            ],
        }
    }

    pub fn from_mat(model: Mat4) -> Self {
        Self {
            model,
            inv_model: model.inverse(),
        }
    }

    pub fn translated(&mut self, pos: Vec3) {
        let (scale, rotation, _) = self.model.to_scale_rotation_translation();
        self.model = Mat4::from_scale_rotation_translation(scale, rotation, pos);
        self.inv_model = self.model.inverse();
    }

    pub fn translate(mut self, pos: Vec3) -> Self {
        let (scale, rotation, translation) = self.model.to_scale_rotation_translation();
        self.model = Mat4::from_scale_rotation_translation(scale, rotation, translation + pos);
        self.inv_model = self.model.inverse();
        self
    }

    pub fn rotate(&mut self, angle_rad: f32, axis: Vec3) {
        let (scale, rotation, translation) = self.model.to_scale_rotation_translation();
        let new_rot = Qua::from_axis_angle(axis, angle_rad);
        self.model = Mat4::from_scale_rotation_translation(scale, new_rot * rotation, translation);
        self.inv_model = self.model.inverse();
    }

    pub fn rotate_qua(&mut self, qua: Qua) {
        let (scale, _, translation) = self.model.to_scale_rotation_translation();
        self.model = Mat4::from_scale_rotation_translation(scale, qua, translation);
        self.inv_model = self.model.inverse();
    }

    pub fn rotated(mut self, angle_rad: f32, axis: Vec3) -> Self {
        let (scale, rotation, translation) = self.model.to_scale_rotation_translation();
        let new_rot = Qua::from_axis_angle(axis, angle_rad);
        self.model = Mat4::from_scale_rotation_translation(scale, new_rot * rotation, translation);
        self.inv_model = self.model.inverse();
        self
    }

    pub fn rotated_qua(mut self, qua: Qua) -> Self {
        let (scale, rotation, translation) = self.model.to_scale_rotation_translation();
        self.model = Mat4::from_scale_rotation_translation(scale, rotation * qua, translation);
        self.inv_model = self.model.inverse();
        self
    }

    pub fn look_in_dir(&mut self, mut dir: Vec3, up: Vec3) {
        let rotation = if dir == up {
            Qua::default()
        } else {
            dir = dir.normalize();
            let right = up.cross(dir).normalize();
            let recalculated_up = dir.cross(right);
            let rotation_matrix = Mat3::from_cols(right, recalculated_up, dir);
            Qua::from_mat3(&rotation_matrix)
        };
        let (scale, _, translation) = self.model.to_scale_rotation_translation();
        self.model = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.inv_model = self.model.inverse();
    }

    pub fn scale(&mut self, size: Vec3) {
        let (_, rotation, translation) = self.model.to_scale_rotation_translation();
        self.model = Mat4::from_scale_rotation_translation(size, rotation, translation);
        self.inv_model = self.model.inverse();
    }

    pub fn scaled(mut self, size: Vec3) -> Self {
        let (scale, rotation, translation) = self.model.to_scale_rotation_translation();
        self.model = Mat4::from_scale_rotation_translation(scale * size, rotation, translation);
        self.inv_model = self.model.inverse();
        self
    }

    pub fn position(&self) -> Vec3 {
        self.model.to_scale_rotation_translation().2
    }

    pub fn size(&self) -> Vec3 {
        self.model.to_scale_rotation_translation().0
    }

    pub fn rotation(&self) -> Qua {
        self.model.to_scale_rotation_translation().1
    }

    pub fn mat(&self) -> Mat4 {
        self.model
    }

    pub fn set_mat(&mut self, mat: Mat4) {
        self.model = mat;
        self.inv_model = self.model.inverse();
    }
}

impl<T: bytemuck::Pod> Object<T> {
    pub fn new(
        renderer: &mut Renderer,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        instances: Vec<T>,
    ) -> Object<T>
    where
        T: bytemuck::Pod,
    {
        let render_data = {
            let vertex_buffer = renderer.create_vertex_buffer(bytemuck::cast_slice(&vertices));
            let instance_buffer = renderer.create_vertex_buffer(bytemuck::cast_slice(&instances));
            let index_buffer = renderer.create_index_buffer(bytemuck::cast_slice(&indices));
            ObjectRenderData {
                vertex_buffer,
                instance_buffer,
                index_buffer,
            }
        };

        let previous_instances_len = instances.len();

        Object {
            vertices,
            indices,
            instances,
            render_data,
            previous_instances_len,
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        if self.previous_instances_len != self.instances.len() {
            self.render_data.instance_buffer =
                renderer.create_vertex_buffer(bytemuck::cast_slice(&self.instances));
            self.previous_instances_len = self.instances.len();
        } else {
            renderer.write_buffer(
                self.render_data.instance_buffer,
                bytemuck::cast_slice(&self.instances),
            );
        }
    }

    pub fn from_obj(data: &str, renderer: &mut Renderer, instances: Vec<T>) -> Object<T> {
        let (vertices, indices) = load_obj(data);
        Self::new(renderer, vertices, indices, instances)
    }

    pub fn triangle(renderer: &mut Renderer, instances: Vec<T>) -> Self
    where
        T: bytemuck::Pod,
    {
        let (vertices, indices) = triangle_data();
        Self::new(renderer, vertices, indices, instances)
    }

    pub fn quad(renderer: &mut Renderer, instances: Vec<T>) -> Self
    where
        T: bytemuck::Pod,
    {
        let (vertices, indices) = quad_data();
        Self::new(renderer, vertices, indices, instances)
    }

    pub fn plane(
        renderer: &mut Renderer,
        subdivisions: u32,
        centered: bool,
        instances: Vec<T>,
    ) -> Self
    where
        T: bytemuck::Pod,
    {
        let (vertices, indices) = plane_data(subdivisions, centered);
        Self::new(renderer, vertices, indices, instances)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D2Instance {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub color: Vec3,
}

impl Default for D2Instance {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
            color: Vec3::ONE,
        }
    }
}

impl D2Instance {
    pub fn desc() -> BufferLayout {
        BufferLayout {
            step_mode: crate::shader::BufferLayoutStepMode::Instance,
            stride: std::mem::size_of::<Self>(),
            entries: &[
                BufferLayoutEntry {
                    location: 3,
                    offset: std::mem::offset_of!(Self, position) as u64,
                    data_type: BufferLayoutEntryDataType::Float32x2,
                },
                BufferLayoutEntry {
                    location: 4,
                    offset: std::mem::offset_of!(Self, scale) as u64,
                    data_type: BufferLayoutEntryDataType::Float32x2,
                },
                BufferLayoutEntry {
                    location: 5,
                    offset: std::mem::offset_of!(Self, rotation) as u64,
                    data_type: BufferLayoutEntryDataType::Float32,
                },
                BufferLayoutEntry {
                    location: 6,
                    offset: std::mem::offset_of!(Self, color) as u64,
                    data_type: BufferLayoutEntryDataType::Float32x3,
                },
            ],
        }
    }
    pub fn set_color(mut self, color: Vec3) -> Self {
        self.color = color;
        self
    }

    pub fn set_position(mut self, pos: Vec2) -> Self {
        self.position = pos;
        self
    }

    pub fn translate(mut self, pos: Vec2) -> Self {
        self.position += pos;
        self
    }

    pub fn rotate(&mut self, angle_rad: f32) -> &mut Self {
        self.rotation += angle_rad;
        self
    }

    pub fn scale(mut self, size: f32) -> Self {
        self.scale *= size;
        self
    }

    pub fn set_size(mut self, size: Vec2) -> Self {
        self.scale = size;
        self
    }
}

pub trait Renderable {
    fn get_buffers(&self) -> (BufferHandle, BufferHandle, Option<BufferHandle>);

    fn num_indices(&self) -> u32;

    fn num_instances(&self) -> u32;
}

impl<T: std::any::Any> Renderable for Object<T> {
    fn num_instances(&self) -> u32 {
        self.previous_instances_len as u32
    }

    fn num_indices(&self) -> u32 {
        self.indices.len() as u32
    }

    fn get_buffers(&self) -> (BufferHandle, BufferHandle, Option<BufferHandle>) {
        (
            self.render_data.vertex_buffer,
            self.render_data.index_buffer,
            Some(self.render_data.instance_buffer),
        )
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::ZERO,
            uv: Vec2::ZERO,
        }
    }
}
