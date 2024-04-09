use crate::core::bind_group::{BindGroup, BindGroupLayout, BindGroupLayoutEntry};

use super::{BindGroupHandle, BindGroupRenderData, BufferHandle, Renderer, UntypedBindGroupHandle};

impl Renderer {
    pub fn create_bind_group<T: BindGroup>(&mut self, bind_group: T) -> BindGroupHandle<T> {
        {
            let layout = bind_group.get_layout(self);
            let bind_group_layout = Self::get_layout(&self.device, &layout);
            let mut first_handle = BufferHandle(0); // TODO FIX THIS LMAO
            let entries = layout
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: match entry {
                        BindGroupLayoutEntry::Data(handle) => {
                            first_handle = *handle;
                            self.buffers[handle.0].as_entire_binding()
                        }
                        BindGroupLayoutEntry::Texture(handle) => {
                            wgpu::BindingResource::TextureView(&self.textures[handle.0].view)
                        }
                        BindGroupLayoutEntry::Sampler(handle) => {
                            wgpu::BindingResource::Sampler(&self.samplers[handle.0])
                        }
                    },
                })
                .collect::<Vec<_>>();

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &entries[..],
                label: Some("bind_group"),
            });

            self.bind_groups_render_data.push(BindGroupRenderData {
                bind_group_layout,
                bind_group,
                buffer_handle: first_handle,
            });
        }

        self.bind_groups.push(Box::new(bind_group));
        BindGroupHandle(self.bind_groups.len() - 1, std::marker::PhantomData::<T>)
    }

    pub fn get_bind_group(&self, handle: UntypedBindGroupHandle) -> Option<&dyn BindGroup> {
        if let Some(b) = self.bind_groups.get(handle.0) {
            Some(b.as_ref())
        } else {
            None
        }
    }

    pub fn get_bind_group_mut(
        &mut self,
        handle: UntypedBindGroupHandle,
    ) -> Option<&mut dyn BindGroup> {
        if let Some(b) = self.bind_groups.get_mut(handle.0) {
            Some(b.as_mut())
        } else {
            None
        }
    }

    pub fn get_bind_group_t<T: BindGroup>(&self, handle: BindGroupHandle<T>) -> Option<&T> {
        if let Some(b) = self.bind_groups.get(handle.0) {
            let b = b.as_ref();
            let any = b.as_any();
            if let Some(bind_group) = any.downcast_ref::<T>() {
                return Some(bind_group);
            }
        }
        None
    }

    pub fn get_bind_group_t_mut<T: BindGroup>(
        &mut self,
        handle: BindGroupHandle<T>,
    ) -> Option<&mut T> {
        if let Some(b) = self.bind_groups.get_mut(handle.0) {
            let b = b.as_mut();
            let any = b.as_any_mut();
            if let Some(bind_group) = any.downcast_mut::<T>() {
                return Some(bind_group);
            }
        }
        None
    }

    pub fn get_layout(device: &wgpu::Device, layout: &BindGroupLayout) -> wgpu::BindGroupLayout {
        let entries: Vec<_> = layout
            .entries
            .iter()
            .map(|e| match e {
                BindGroupLayoutEntry::Data(_) => wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry::Texture(_) => wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry::Sampler(_) => wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            })
            .collect();

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("bind_group_layout"),
        })
    }

    pub fn get_layouts(
        device: &wgpu::Device,
        layouts: &[BindGroupLayout],
    ) -> Vec<wgpu::BindGroupLayout> {
        layouts
            .iter()
            .map(|e| Self::get_layout(device, e))
            .collect()
    }

    pub fn write_bind_group(&mut self, handle: UntypedBindGroupHandle, data: &[u8]) {
        let render_data = &self.bind_groups_render_data[handle.0];
        self.queue
            .write_buffer(&self.buffers[render_data.buffer_handle.0], 0, data);
    }
}

impl<T> Clone for BindGroupHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for BindGroupHandle<T> {}

impl<T> From<BindGroupHandle<T>> for UntypedBindGroupHandle {
    fn from(value: BindGroupHandle<T>) -> Self {
        Self(value.0)
    }
}

impl<T> From<&BindGroupHandle<T>> for UntypedBindGroupHandle {
    fn from(value: &BindGroupHandle<T>) -> Self {
        Self(value.0)
    }
}

impl<T> std::fmt::Debug for BindGroupHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, std::any::type_name::<T>())
    }
}
