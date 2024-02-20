use cgmath::{num_traits::Signed, Zero};
use jandering_engine::{
    bind_group::{camera::d2::D2CameraBindGroup, texture::TextureBindGroup},
    engine::EngineContext,
    object::{primitives, D2Instance, Object, VertexRaw},
    renderer::{BindGroupHandle, Renderer, UntypedBindGroupHandle},
    shader::Shader,
    texture::{load_texture, TextureDescriptor},
    types::Vec2,
};
use winit::event::{ElementState, MouseButton, WindowEvent};

use super::{
    constants::{
        GRAVITY, HALF_TILE_SIZE, JUMP_DURATION, JUMP_HEIGHT, JUMP_HEIGHT_PIXELS, TILE_SIZE,
    },
    hue_plugin::HueBindGroup,
    map::{tiles::*, Map},
};

pub struct Player {
    pub position: Vec2,
    velocity: Vec2,
    quad: Object<D2Instance>,
    can_jump: bool,
    wants_to_jump: bool,
    rotation: cgmath::Deg<f32>,
    is_alive: bool,
    shader: Shader,
    bind_groups: [UntypedBindGroupHandle; 3],
    hue_bg: BindGroupHandle<HueBindGroup>,
    pub hue: f32,
}

impl Player {
    pub async fn new(
        renderer: &mut Renderer,
        camera_bg: BindGroupHandle<D2CameraBindGroup>,
    ) -> Self {
        let quad = primitives::quad::<D2Instance>(
            renderer,
            vec![D2Instance {
                position: Vec2::new(0.0, 0.0),
                scale: Vec2::new(TILE_SIZE, TILE_SIZE),
                rotation: 0.0,
            }],
        );

        let texture = load_texture("player.png", renderer, TextureDescriptor::default())
            .await
            .expect("kys");
        let texture_handle = renderer.add_texture(texture);
        let texture_bing_group = TextureBindGroup::new(renderer, texture_handle);

        let hue_bg = renderer.add_bind_group(HueBindGroup::new(renderer));
        let texture_bg = renderer.add_bind_group(texture_bing_group);

        let bind_groups: [UntypedBindGroupHandle; 3] =
            [camera_bg.into(), hue_bg.into(), texture_bg.into()];

        let shader = jandering_engine::shader::create_shader(
            renderer,
            jandering_engine::shader::ShaderDescriptor {
                code: include_str!("player_shader.wgsl"),
                descriptors: &[VertexRaw::desc(), D2Instance::desc()],
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
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
            can_jump: false,
            wants_to_jump: false,
            rotation: cgmath::Deg(0.0),
            is_alive: false,
            shader,
            bind_groups,
            hue_bg,
            hue: 0.0,
        }
    }

    pub fn update(&mut self, context: &mut EngineContext, map: &Map) {
        if !self.is_alive {
            return;
        }

        for event in context.events.iter() {
            if let WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } = event
            {
                self.wants_to_jump = matches!(state, ElementState::Pressed);
            }
        }

        let buffer = TILE_SIZE * if self.velocity.y.is_zero() { 0.5 } else { 0.2 };
        if let Some(tile) = map.get_tile((self.position.x + buffer) as u32, self.position.y as u32)
        {
            if *tile != AIR {
                self.is_alive = false;
                return;
            }
        }

        let dt = context.dt as f32;
        self.update_movement(map, dt);
    }

    fn update_movement(&mut self, map: &Map, dt: f32) {
        self.velocity.y += GRAVITY * dt;

        let jump_vel = (-2.0 * GRAVITY * JUMP_HEIGHT_PIXELS).sqrt();

        if self.can_jump && self.wants_to_jump {
            self.velocity.y = jump_vel;
            self.can_jump = false;
        }

        if self.velocity.y.is_negative() {
            let below = (self.position.y - HALF_TILE_SIZE + self.velocity.y * dt).max(0.0);
            let rounded_pos_y = (below / TILE_SIZE).round() * TILE_SIZE;
            match map.get_tile(self.position.x as u32, rounded_pos_y as u32) {
                Some(&GROUND) => {
                    self.velocity.y = 0.0;
                    self.position.y = rounded_pos_y + TILE_SIZE;
                    self.can_jump = true;
                    let rounded_rot = (self.rotation / 90.0).0.floor() * 90.0;
                    self.rotation +=
                        cgmath::Deg((rounded_rot - self.rotation.0) * (20.0 * dt).min(1.0));
                }
                Some(&SPIKE) => {
                    self.is_alive = false;
                    return;
                }
                _ => {}
            }
        }

        if !self.velocity.y.is_zero() {
            self.rotation -= cgmath::Deg(90.0 * JUMP_HEIGHT * 0.5 * (1.0 / JUMP_DURATION) * dt);
        }

        self.position += self.velocity * dt;
    }

    fn update_render_data(&mut self, renderer: &mut Renderer) {
        let instance = self.quad.instances.first_mut().unwrap();
        instance.position = self.position;
        instance.rotation = cgmath::Rad::from(self.rotation).0;

        let hue_bind_group = renderer.get_bind_group_t_mut(self.hue_bg).unwrap();
        hue_bind_group.uniform.hue = self.hue;
    }

    pub fn render(&mut self, context: &mut EngineContext, renderer: &mut Renderer) {
        if !self.is_alive {
            return;
        }
        self.update_render_data(renderer);
        self.quad.update(context, renderer);
        renderer.render(&[&self.quad], context, &self.shader, &self.bind_groups);
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive
    }

    pub fn reset(&mut self, position: Vec2) {
        self.position = position;
        self.velocity = Vec2::new(0.0, 0.0);
        self.can_jump = false;
        self.wants_to_jump = false;
        self.rotation = cgmath::Deg(0.0);
        self.is_alive = true;
    }
}
