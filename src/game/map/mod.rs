use cgmath::Zero;
use jandering_engine::{
    bind_group::{camera::d2::D2CameraBindGroup, texture::TextureBindGroup},
    engine::EngineContext,
    object::{Object, VertexRaw},
    renderer::{BindGroupHandle, Renderer, UntypedBindGroupHandle},
    shader::Shader,
    texture::{load_texture, TextureDescriptor},
    types::Vec2,
    utils::FilePath,
};

use tiles::*;

use super::constants::{HALF_TILE_SIZE, TILE_SIZE};

use map_bind_group::MapBindGroup;

mod map_bind_group;

pub mod tiles;

type Tile = u8;

struct Tiles {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Tile>,
}

#[allow(dead_code)]
pub struct Map {
    tiles: Tiles,
    quad: Object<TileInstance>,
    shader: Shader,
    pub hue: f32,
    bind_groups: [UntypedBindGroupHandle; 3],
    map_bg: BindGroupHandle<MapBindGroup>,
    texture_bg: BindGroupHandle<TextureBindGroup>,
    pub position: Vec2,
}

impl Map {
    pub async fn new(
        renderer: &mut Renderer,
        path: FilePath<'_>,
        camera_bg: BindGroupHandle<D2CameraBindGroup>,
    ) -> Self {
        let tiles = Self::create_tiles(path).await;

        let mut instances = Vec::with_capacity((tiles.width * tiles.height) as usize);
        (0..tiles.width).for_each(|x| {
            (0..tiles.height).for_each(|y| {
                let id = tiles.data.get(x * tiles.height + y).unwrap();
                instances.push(TileInstance {
                    position: Vec2::new(x as f32, y as f32),
                    id: *id as u32,
                });
            })
        });

        let tilemap_texture = load_texture("tilemap.png", renderer, TextureDescriptor::default())
            .await
            .expect("kys");
        let tilemap_texture_handle = renderer.add_texture(tilemap_texture);
        let tilemap_texture_bind_group = TextureBindGroup::new(renderer, tilemap_texture_handle);

        let map_bg = renderer.add_bind_group(MapBindGroup::new(renderer));
        let texture_bg = renderer.add_bind_group(tilemap_texture_bind_group);

        let bind_groups: [UntypedBindGroupHandle; 3] =
            [camera_bg.into(), map_bg.into(), texture_bg.into()];

        let quad = jandering_engine::object::primitives::quad(renderer, instances);
        let shader = jandering_engine::shader::create_shader(
            renderer,
            jandering_engine::shader::ShaderDescriptor {
                code: include_str!("map_shader.wgsl"),
                descriptors: &[VertexRaw::desc(), TileInstance::desc()],
                bind_groups: &bind_groups,
                targets: Some(&[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })]),
                ..Default::default()
            },
        );

        Self {
            quad,
            shader,
            hue: 0.0,
            tiles,
            position: Vec2::zero(),
            map_bg,
            texture_bg,
            bind_groups,
        }
    }

    async fn create_tiles(path: FilePath<'_>) -> Tiles {
        let height = 10;

        let map: String = jandering_engine::utils::load_text(path)
            .await
            .expect("failed to load map");

        let width = map.lines().map(|e| e.len()).max().unwrap();

        let rows: Vec<Vec<Tile>> = map
            .lines()
            .map(|e| e.chars().map(|e| e.to_digit(10).unwrap() as Tile).collect())
            .collect();

        let data: Vec<u8> = (0..width)
            .flat_map(|x| {
                rows.iter()
                    .rev()
                    .map(|row| {
                        if let Some(tile) = row.get(x) {
                            *tile
                        } else {
                            AIR
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Tiles {
            width,
            height,
            data,
        }
    }

    #[allow(dead_code)]
    pub async fn load(&mut self, renderer: &mut Renderer, path: FilePath<'_>) {
        let tiles = Self::create_tiles(path).await;

        let mut instances = Vec::with_capacity((tiles.width * tiles.height) as usize);

        (0..tiles.width).for_each(|x| {
            (0..tiles.height).for_each(|y| {
                let id = tiles.data.get(x * tiles.height + y).unwrap();
                instances.push(TileInstance {
                    position: Vec2::new(x as f32, y as f32),
                    id: *id as u32,
                });
            })
        });

        let quad = jandering_engine::object::primitives::quad::<TileInstance>(renderer, instances);

        self.quad = quad;
    }

    pub fn render(&mut self, context: &mut EngineContext, renderer: &mut Renderer) {
        self.update_render_data(renderer);
        self.quad.update(context, renderer);

        let x_tiles = renderer.config.width / TILE_SIZE as u32 + 3;
        let y_tiles = renderer.config.height / TILE_SIZE as u32;
        let start = (-self.position.x / TILE_SIZE) as u32;
        let end = (start + x_tiles) * y_tiles;

        renderer.render_with_range(
            &self.quad,
            context,
            &self.shader,
            &self.bind_groups,
            start..end,
        );
    }

    fn update_render_data(&mut self, renderer: &mut Renderer) {
        self.hue = self.position.x / (TILE_SIZE * 100.0);
        self.position.x = self.position.x.clamp(
            -(self.tiles.width as f32 * TILE_SIZE - renderer.config.width as f32 - TILE_SIZE),
            0.0,
        );

        let map_bind_group = renderer.get_bind_group_t_mut(self.map_bg).unwrap();
        map_bind_group.uniform.position = self.position;
        map_bind_group.uniform.hue = self.hue;
    }

    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        let pos = (Vec2::new(x as f32, y as f32 + HALF_TILE_SIZE) - self.position) / TILE_SIZE;
        (pos.x.round() * self.tiles.height as f32 + pos.y) as usize
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        self.tiles.data.get(self.coords_to_index(x, y))
    }

    pub fn set_tile(&mut self, x: u32, y: u32, val: Tile) {
        let index = self.coords_to_index(x, y);

        if let Some(tile) = self.tiles.data.get_mut(index) {
            *tile = val;
        }
        if let Some(instance) = self.quad.instances.get_mut(index) {
            instance.id = val as u32;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TileInstance {
    pub position: Vec2,
    pub id: u32,
}

impl TileInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TileInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
