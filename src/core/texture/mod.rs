use image::GenericImageView;

use crate::{
    types::UVec2,
    utils::{load_binary, FilePath},
};

use super::renderer::Renderer;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
}

pub struct TextureDescriptor {
    pub is_normal_map: bool,
    pub address_mode: wgpu::AddressMode,
    pub size: UVec2,
    pub sample_count: u32,
    pub format: wgpu::TextureFormat,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        Self {
            is_normal_map: false,
            address_mode: wgpu::AddressMode::ClampToEdge,
            size: UVec2::new(0, 0),
            sample_count: 1,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
        }
    }
}

impl Texture {
    pub async fn from_file(file_name: &str, renderer: &Renderer, desc: TextureDescriptor) -> Self {
        let data = load_binary(FilePath::FileName(file_name)).await.unwrap();
        Self::from_bytes(renderer, &data, desc)
    }

    pub fn from_bytes(renderer: &Renderer, bytes: &[u8], desc: TextureDescriptor) -> Self {
        let img = image::load_from_memory(bytes).unwrap();
        Self::from_image(renderer, &img, desc)
    }

    pub fn from_image(
        renderer: &Renderer,
        img: &image::DynamicImage,
        desc: TextureDescriptor,
    ) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let format = if desc.is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: desc.address_mode,
            address_mode_v: desc.address_mode,
            address_mode_w: desc.address_mode,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            width: dimensions.0,
            height: dimensions.1,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
