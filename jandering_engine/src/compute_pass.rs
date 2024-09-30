use std::collections::HashMap;

use crate::renderer::{ComputeShaderHandle, UntypedBindGroupHandle};

pub struct ComputePassData {
    pub(crate) shader: ComputeShaderHandle,
    pub(crate) bind_groups: HashMap<u32, UntypedBindGroupHandle>,
}

impl Default for ComputePassData {
    fn default() -> Self {
        Self {
            shader: ComputeShaderHandle(0),
            bind_groups: Default::default(),
        }
    }
}

pub trait ComputePassTrait {
    fn get_data(&mut self) -> &mut ComputePassData;

    fn bind(mut self, slot: u32, bind_group: UntypedBindGroupHandle) -> Self
    where
        Self: Sized,
    {
        self.get_data()
            .bind_groups
            .entry(slot)
            .and_modify(|e| *e = bind_group)
            .or_insert(bind_group);
        self
    }

    fn unbind(mut self, slot: u32) -> Self
    where
        Self: Sized,
    {
        self.get_data().bind_groups.remove(&slot);
        self
    }

    fn dispatch(self, x: u32, y: u32, z: u32);

    fn set_shader(mut self, shader: ComputeShaderHandle) -> Self
    where
        Self: Sized,
    {
        self.get_data().shader = shader;
        self
    }
}
