use std::any::Any;

pub mod camera;
pub mod resolution;
pub trait BindGroup: Any + BindGroupToAny {
    fn get_data(&self) -> Box<[u8]>;
}

pub trait BindGroupToAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> BindGroupToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub enum BindGroupLayoutEntry {
    Data,
    Texture,
    Sampler,
}

pub struct BindGroupLayout {
    pub entries: Vec<BindGroupLayoutEntry>,
}
