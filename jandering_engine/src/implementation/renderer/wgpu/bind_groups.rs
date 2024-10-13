use wgpu::VertexAttribute;

use crate::{
    bind_group::{
        BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutDescriptorEntry,
        BindGroupLayoutEntry,
    },
    renderer::{BindGroupHandle, UntypedBindGroupHandle},
    shader::BufferLayout,
};

use super::WGPURenderer;

impl WGPURenderer {
    pub fn get_layout_descriptor<'a, T: Into<BindGroupLayoutDescriptorEntry> + Clone>(
        device: &wgpu::Device,
        entries: &'a [T],
    ) -> wgpu::BindGroupLayout {
        let entries: Vec<_> = entries
            .iter()
            .map(
                |e| match Into::<BindGroupLayoutDescriptorEntry>::into(e.clone()) {
                    BindGroupLayoutDescriptorEntry::Data { is_uniform, .. } => {
                        let ty = wgpu::BindingType::Buffer {
                            ty: if is_uniform {
                                wgpu::BufferBindingType::Uniform
                            } else {
                                wgpu::BufferBindingType::Storage { read_only: false }
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        };

                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE
                                | wgpu::ShaderStages::VERTEX
                                | wgpu::ShaderStages::FRAGMENT,
                            ty,
                            count: None,
                        }
                    }
                    BindGroupLayoutDescriptorEntry::Texture { sample_type, .. } => {
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE
                                | wgpu::ShaderStages::VERTEX
                                | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: match sample_type {
                                    crate::bind_group::TextureSampleType::Filterable => {
                                        wgpu::TextureSampleType::Float { filterable: true }
                                    }
                                    crate::bind_group::TextureSampleType::NonFilterable => {
                                        wgpu::TextureSampleType::Float { filterable: false }
                                    }
                                    crate::bind_group::TextureSampleType::Depth => {
                                        wgpu::TextureSampleType::Depth
                                    }
                                },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        }
                    }
                    BindGroupLayoutDescriptorEntry::Sampler { sampler_type, .. } => {
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE
                                | wgpu::ShaderStages::VERTEX
                                | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(match sampler_type {
                                crate::bind_group::SamplerType::Filtering => {
                                    wgpu::SamplerBindingType::Filtering
                                }
                                crate::bind_group::SamplerType::NonFiltering => {
                                    wgpu::SamplerBindingType::NonFiltering
                                }
                                crate::bind_group::SamplerType::Comparison => {
                                    wgpu::SamplerBindingType::Comparison
                                }
                            }),
                            count: None,
                        }
                    }
                },
            )
            .collect();

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("bind_group_layout"),
        })
    }

    pub fn get_layout_descriptors(
        device: &wgpu::Device,
        layouts: &[BindGroupLayoutDescriptor],
    ) -> Vec<wgpu::BindGroupLayout> {
        layouts
            .iter()
            .map(|e| Self::get_layout_descriptor(device, &e.entries))
            .collect()
    }

    pub fn get_layout_and_entries<'a>(
        &'a self,
        device: &'a wgpu::Device,
        layout: &BindGroupLayout,
    ) -> (wgpu::BindGroupLayout, Vec<wgpu::BindGroupEntry<'a>>) {
        let bind_group_layout = Self::get_layout_descriptor(device, &layout.entries);
        let entries = layout
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: match entry {
                    BindGroupLayoutEntry::Data(handle) => {
                        self.buffers[handle.index].as_entire_binding()
                    }
                    BindGroupLayoutEntry::Texture { handle, .. } => {
                        wgpu::BindingResource::TextureView(&self.textures[handle.0].view)
                    }
                    BindGroupLayoutEntry::Sampler { handle, .. } => {
                        wgpu::BindingResource::Sampler(&self.samplers[handle.0])
                    }
                },
            })
            .collect::<Vec<_>>();

        (bind_group_layout, entries)
    }

    pub fn get_buffer_attributes(layout: &BufferLayout) -> Vec<VertexAttribute> {
        let mut entries = Vec::new();
        let mut offset = 0;
        for entry in layout.entries.iter() {
            entries.push(wgpu::VertexAttribute {
                format: match entry.data_type {
                    crate::shader::BufferLayoutEntryDataType::Float32 => {
                        wgpu::VertexFormat::Float32
                    }
                    crate::shader::BufferLayoutEntryDataType::Float32x2 => {
                        wgpu::VertexFormat::Float32x2
                    }
                    crate::shader::BufferLayoutEntryDataType::Float32x3 => {
                        wgpu::VertexFormat::Float32x3
                    }
                    crate::shader::BufferLayoutEntryDataType::Float32x4 => {
                        wgpu::VertexFormat::Float32x4
                    }
                    crate::shader::BufferLayoutEntryDataType::U32 => wgpu::VertexFormat::Uint32,
                },
                offset,
                shader_location: entry.location,
            });
            offset += entry.data_type.size_bytes();
        }
        entries
    }

    pub fn get_buffer_layouts<'a>(
        entries: &'a [Vec<VertexAttribute>],
        layouts: &[BufferLayout],
    ) -> Vec<wgpu::VertexBufferLayout<'a>> {
        layouts
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if let Some(last) = entries[i].last() {
                    let offset = last.offset
                        + 4 * match last.format {
                            wgpu::VertexFormat::Float32 => 1,
                            wgpu::VertexFormat::Float32x2 => 2,
                            wgpu::VertexFormat::Float32x3 => 3,
                            wgpu::VertexFormat::Float32x4 => 4,
                            _ => panic!(),
                        };

                    wgpu::VertexBufferLayout {
                        array_stride: offset as wgpu::BufferAddress,
                        step_mode: match e.step_mode {
                            crate::shader::BufferLayoutStepMode::Vertex => {
                                wgpu::VertexStepMode::Vertex
                            }
                            crate::shader::BufferLayoutStepMode::Instance => {
                                wgpu::VertexStepMode::Instance
                            }
                        },
                        attributes: &entries[i],
                    }
                } else {
                    panic!()
                }
            })
            .collect()
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
