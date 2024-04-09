use crate::core::texture::{
    sampler::{SamplerAddressMode, SamplerCompareFunction, SamplerDescriptor, SamplerFilterMode},
    texture_usage, Texture, TextureDescriptor, TextureFormat,
};

use super::{Renderer, SamplerHandle, TextureHandle};

impl Renderer {
    fn create_texture_at(&mut self, desc: TextureDescriptor, handle: TextureHandle) {
        let size = wgpu::Extent3d {
            width: desc.size.x,
            height: desc.size.y,
            depth_or_array_layers: 1,
        };

        let (format, channels) = match desc.format {
            TextureFormat::Rgba8U => (wgpu::TextureFormat::Rgba8UnormSrgb, 4),
            TextureFormat::Bgra8U => (wgpu::TextureFormat::Bgra8UnormSrgb, 4),
            TextureFormat::Depth32F => (wgpu::TextureFormat::Depth32Float, 1),
        };

        let mut usage = wgpu::TextureUsages::empty();
        if desc.usage & texture_usage::BIND != texture_usage::NONE {
            usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        }
        if desc.usage & texture_usage::COPY_SRC != texture_usage::NONE {
            usage |= wgpu::TextureUsages::COPY_SRC;
        }
        if desc.usage & texture_usage::COPY_TARGET != texture_usage::NONE {
            usage |= wgpu::TextureUsages::COPY_DST;
        }
        if desc.usage & texture_usage::TARGET != texture_usage::NONE {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count: 1,
            sample_count: desc.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        if let Some(data) = desc.data {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(channels * size.width),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture = Texture {
            texture,
            view,
            width: desc.size.x,
            height: desc.size.y,
        };

        if handle.0 >= self.textures.len() {
            self.textures.push(texture);
        } else {
            self.textures[handle.0] = texture;
        }
    }

    pub fn create_texture(&mut self, desc: TextureDescriptor) -> TextureHandle {
        self.create_texture_at(desc, TextureHandle(self.textures.len()));
        TextureHandle(self.textures.len() - 1)
    }

    pub fn re_create_texture(&mut self, desc: TextureDescriptor, handle: TextureHandle) {
        self.create_texture_at(desc, handle);
    }

    pub fn add_texture(&mut self, texture: Texture) -> TextureHandle {
        self.textures.push(texture);
        TextureHandle(self.textures.len() - 1)
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle.0)
    }

    pub fn create_sampler(&mut self, desc: SamplerDescriptor) -> SamplerHandle {
        let address_mode = match desc.address_mode {
            SamplerAddressMode::Repeat => wgpu::AddressMode::Repeat,
            SamplerAddressMode::RepeatMirrored => wgpu::AddressMode::MirrorRepeat,
            SamplerAddressMode::Clamp => wgpu::AddressMode::ClampToEdge,
        };
        let filter = match desc.filter {
            SamplerFilterMode::Linear => wgpu::FilterMode::Linear,
            SamplerFilterMode::Nearest => wgpu::FilterMode::Nearest,
        };
        let lod_min_clamp = desc.lod_min_clamp;
        let lod_max_clamp = desc.lod_max_clamp;
        let compare = desc.compare.map(|e| match e {
            SamplerCompareFunction::Less => wgpu::CompareFunction::Less,
            SamplerCompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
            SamplerCompareFunction::Equal => wgpu::CompareFunction::Equal,
            SamplerCompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            SamplerCompareFunction::Greater => wgpu::CompareFunction::Greater,
        });

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            lod_min_clamp,
            lod_max_clamp,
            compare,
            ..Default::default()
        });

        self.samplers.push(sampler);
        SamplerHandle(self.samplers.len() - 1)
    }
}
