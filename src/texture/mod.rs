use image::GenericImageView;
use wgpu::TextureUsages;

use crate::{
    renderer::Renderer,
    types::UVec2,
    utils::{load_binary, FilePath},
};

pub struct Texture {
    pub texture: Option<wgpu::Texture>,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    width: u32,
    height: u32,
}

pub struct TextureDescriptor {
    pub is_normal_map: bool,
    pub address_mode: wgpu::AddressMode,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        Self {
            is_normal_map: false,
            address_mode: wgpu::AddressMode::ClampToEdge,
        }
    }
}

pub async fn load_texture(
    file_name: &str,
    renderer: &Renderer,
    desc: TextureDescriptor,
) -> anyhow::Result<Texture> {
    let data = load_binary(FilePath::FileName(file_name)).await?;
    Texture::from_bytes(renderer, &data, desc)
}

impl Texture {
    pub fn from_bytes(
        renderer: &Renderer,
        bytes: &[u8],
        desc: TextureDescriptor,
    ) -> anyhow::Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(renderer, &img, desc))
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
            texture: Some(texture),
            view,
            sampler,
            width: dimensions.0,
            height: dimensions.1,
        }
    }

    pub fn new_color(
        renderer: &Renderer,
        size: UVec2,
        usage: TextureUsages,
        format: Option<wgpu::TextureFormat>,
    ) -> Self {
        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("color_texture"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb),
            usage,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture: Some(texture),
            view,
            sampler,
            width: size.x,
            height: size.y,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
