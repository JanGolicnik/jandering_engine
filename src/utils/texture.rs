// use crate::renderer::{BindGroupHandle, Renderer, SamplerHandle, TextureHandle};

// use crate::bind_group::{
//     BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutDescriptorEntry,
//     BindGroupLayoutEntry, SamplerType, TextureSampleType,
// };

// pub struct TextureBindGroup {
//     pub texture_handle: TextureHandle,
//     pub bind_group_handle: BindGroupHandle,
// }

// impl TextureBindGroup {
//     fn get_layout(&self) -> BindGroupLayout {
//         BindGroupLayout {
//             entries: vec![BindGroupLayoutEntry::Texture {
//                 handle: self.texture_handle,
//                 sample_type: TextureSampleType::default(),
//             }],
//         }
//     }

//     fn get_layout_descriptor() -> BindGroupLayoutDescriptor
//     where
//         Self: Sized,
//     {
//         BindGroupLayoutDescriptor {
//             entries: vec![BindGroupLayoutDescriptorEntry::Texture {
//                 sample_type: Default::default(),
//             }],
//         }
//     }
// }

// impl TextureBindGroup {
//     pub fn new(_renderer: &mut Renderer, texture_handle: TextureHandle) -> Self {
//         Self { texture_handle }
//     }
// }

// pub struct UnfilteredTextureBindGroup {
//     pub texture_handle: TextureHandle,
// }

// impl BindGroup for UnfilteredTextureBindGroup {
//     fn get_layout(&self) -> BindGroupLayout {
//         BindGroupLayout {
//             entries: vec![BindGroupLayoutEntry::Texture {
//                 handle: self.texture_handle,
//                 sample_type: TextureSampleType::NonFilterable,
//             }],
//         }
//     }

//     fn get_layout_descriptor() -> BindGroupLayoutDescriptor
//     where
//         Self: Sized,
//     {
//         BindGroupLayoutDescriptor {
//             entries: vec![BindGroupLayoutDescriptorEntry::Texture {
//                 sample_type: TextureSampleType::NonFilterable,
//             }],
//         }
//     }
// }

// impl UnfilteredTextureBindGroup {
//     pub fn new(_renderer: &mut Renderer, texture_handle: TextureHandle) -> Self {
//         Self { texture_handle }
//     }
// }

use crate::{
    bind_group::{
        BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutDescriptorEntry,
        BindGroupLayoutEntry, SamplerType, TextureSampleType,
    },
    renderer::{BindGroupHandle, Janderer, Renderer, SamplerHandle, TextureHandle},
};

pub struct TextureSamplerBindGroup {
    pub texture_handle: TextureHandle,
    pub sampler_handle: SamplerHandle,
    pub bind_group: BindGroupHandle,
}

impl TextureSamplerBindGroup {
    pub fn new(
        renderer: &mut Renderer,
        texture_handle: TextureHandle,
        sampler_handle: SamplerHandle,
    ) -> Self {
        let bind_group = renderer.create_bind_group(BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture {
                    handle: texture_handle,
                    sample_type: TextureSampleType::default(),
                },
                BindGroupLayoutEntry::Sampler {
                    handle: sampler_handle,
                    sampler_type: SamplerType::Filtering,
                },
            ],
        });
        Self {
            texture_handle,
            sampler_handle,
            bind_group,
        }
    }

    pub fn re_create(
        &mut self,
        renderer: &mut Renderer,
        texture_handle: TextureHandle,
        sampler_handle: SamplerHandle,
    ) {
        self.texture_handle = texture_handle;
        self.sampler_handle = sampler_handle;
        renderer.create_bind_group_at(
            BindGroupLayout {
                entries: vec![
                    BindGroupLayoutEntry::Texture {
                        handle: self.texture_handle,
                        sample_type: TextureSampleType::default(),
                    },
                    BindGroupLayoutEntry::Sampler {
                        handle: self.sampler_handle,
                        sampler_type: SamplerType::Filtering,
                    },
                ],
            },
            self.bind_group,
        );
    }
    pub fn get_layout_descriptor() -> BindGroupLayoutDescriptor {
        BindGroupLayoutDescriptor {
            entries: vec![
                BindGroupLayoutDescriptorEntry::Texture {
                    sample_type: Default::default(),
                },
                BindGroupLayoutDescriptorEntry::Sampler {
                    sampler_type: SamplerType::Filtering,
                },
            ],
        }
    }
}

pub struct UnfilteredTextureSamplerBindGroup {
    pub texture_handle: TextureHandle,
    pub sampler_handle: SamplerHandle,
    pub bind_group: BindGroupHandle,
}

impl UnfilteredTextureSamplerBindGroup {
    pub fn new(
        renderer: &mut Renderer,
        texture_handle: TextureHandle,
        sampler_handle: SamplerHandle,
    ) -> Self {
        let bind_group = renderer.create_bind_group(BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture {
                    handle: texture_handle,
                    sample_type: TextureSampleType::NonFilterable,
                },
                BindGroupLayoutEntry::Sampler {
                    handle: sampler_handle,
                    sampler_type: SamplerType::NonFiltering,
                },
            ],
        });

        Self {
            texture_handle,
            sampler_handle,
            bind_group,
        }
    }

    pub fn get_layout_descriptor() -> BindGroupLayoutDescriptor {
        BindGroupLayoutDescriptor {
            entries: vec![
                BindGroupLayoutDescriptorEntry::Texture {
                    sample_type: TextureSampleType::NonFilterable,
                },
                BindGroupLayoutDescriptorEntry::Sampler {
                    sampler_type: SamplerType::NonFiltering,
                },
            ],
        }
    }
}
