use wgpu::VertexAttribute;

use crate::{
    bind_group::{BindGroupLayout, BindGroupLayoutEntry},
    renderer::{BindGroupHandle, UntypedBindGroupHandle},
    shader::BufferLayout,
};

use super::WGPURenderer;

impl WGPURenderer {
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
