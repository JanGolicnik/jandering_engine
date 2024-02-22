use cgmath::VectorSpace;
use jandering_engine::{
    bind_group::{
        camera::d2::D2CameraBindGroup, resolution::ResolutionBindGroup, texture::TextureBindGroup,
    },
    engine::EngineContext,
    object::{primitives, D2Instance, Object, VertexRaw},
    renderer::{BindGroupHandle, Renderer, UntypedBindGroupHandle},
    shader::{Shader, ShaderDescriptor},
    texture::{load_texture, TextureDescriptor},
    types::Vec2,
};
use winit::event::{MouseButton, WindowEvent};

use crate::game::constants::{RESOLUTION_X, RESOLUTION_Y};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum UIAction {
    Play,
    Create,
    Exit,
    Pause,
    Resume,
}

enum UIElementState {
    Idle(f32),
    Pressed,
    Hovered(f32),
}

struct UIElement {
    on_press: UIAction,
    quad: Object<D2Instance>,
    texture_bg: UntypedBindGroupHandle,
    visible: bool,
    state: UIElementState,
    hovered_scale: f32,
    idle_size: Vec2,
    should_update: bool,
}

pub struct UserInterface {
    elements: Vec<UIElement>,
    shader: Shader,
    bind_groups: [UntypedBindGroupHandle; 2],
}

impl UserInterface {
    pub async fn new(
        renderer: &mut Renderer,
        camera_bg: BindGroupHandle<D2CameraBindGroup>,
        resolution_bg: BindGroupHandle<ResolutionBindGroup>,
    ) -> Self {
        let bind_groups = [camera_bg.into(), resolution_bg.into()];

        let elements = Self::create_elements(renderer).await;

        let shader = jandering_engine::shader::create_shader(
            renderer,
            jandering_engine::shader::ShaderDescriptor {
                descriptors: &[VertexRaw::desc(), D2Instance::desc()],
                bind_groups: &[
                    bind_groups[0],
                    bind_groups[1],
                    elements.first().unwrap().texture_bg,
                ],
                targets: Some(&[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })]),
                ..ShaderDescriptor::default_flat()
            },
        );

        Self {
            elements,
            shader,
            bind_groups,
        }
    }

    pub fn update(
        &mut self,
        context: &mut EngineContext,
        renderer: &mut Renderer,
        screen_mouse_pos: Option<Vec2>,
        dt: f32,
    ) -> Option<UIAction> {
        let mut action = None;
        if let Some(mouse_pos) = screen_mouse_pos {
            for event in context.events.iter() {
                match event {
                    WindowEvent::CursorMoved { .. } => {
                        self.handle_hover(mouse_pos, dt);
                    }
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: winit::event::ElementState::Pressed,
                        ..
                    } => {
                        if let Some(val) = self.handle_mouse_click(mouse_pos) {
                            action = Some(val);
                        }
                    }
                    _ => {}
                }
            }
        }

        self.elements
            .iter_mut()
            .filter(|e| e.visible)
            .for_each(|e| e.update(dt));

        self.elements
            .iter_mut()
            .filter(|e| e.should_update)
            .for_each(|e| e.quad.update(context, renderer));

        action
    }

    fn handle_mouse_click(&mut self, mouse_pos: Vec2) -> Option<UIAction> {
        for element in self.elements.iter_mut().filter(|e| e.visible) {
            for instance in element.quad.instances.iter_mut() {
                let hscale = instance.scale * 0.5;
                if mouse_pos.x > instance.position.x - hscale.x
                    && mouse_pos.x < instance.position.x + hscale.x
                    && mouse_pos.y > instance.position.y - hscale.y
                    && mouse_pos.y < instance.position.y + hscale.y
                {
                    element.state = UIElementState::Pressed;
                    return Some(element.on_press);
                }
            }
        }
        None
    }

    fn handle_hover(&mut self, mouse_pos: Vec2, dt: f32) -> Option<UIAction> {
        for element in self.elements.iter_mut().filter(|e| e.visible) {
            for instance in element.quad.instances.iter_mut() {
                let hscale = instance.scale * 0.5;
                if mouse_pos.x > instance.position.x - hscale.x
                    && mouse_pos.x < instance.position.x + hscale.x
                    && mouse_pos.y > instance.position.y - hscale.y
                    && mouse_pos.y < instance.position.y + hscale.y
                {
                    element.should_update = true;
                    if !matches!(element.state, UIElementState::Hovered(_)) {
                        element.state = UIElementState::Hovered(0.0);
                    }
                } else {
                    element.state = UIElementState::Idle(0.0);
                }
            }
        }
        None
    }

    pub fn render(&mut self, context: &mut EngineContext, renderer: &mut Renderer) {
        self.elements
            .iter_mut()
            .filter(|e| e.visible)
            .for_each(|element| {
                element.quad.update(context, renderer);
                renderer.render(
                    &[&element.quad],
                    context,
                    &self.shader,
                    &[self.bind_groups[0], self.bind_groups[1], element.texture_bg],
                );
            });
    }

    pub fn show_mainmenu(&mut self) {
        self.hide();
        self.elements[0].show();
        self.elements[1].show();
    }

    pub fn show_playing(&mut self) {
        self.hide();
        self.elements[2].show();
    }

    pub fn show_paused(&mut self) {
        self.hide();
        self.elements[3].show();
        self.elements[4].show()
    }

    pub fn show_creating(&mut self) {
        self.hide();
        self.elements[3].show()
    }

    pub fn hide(&mut self) {
        self.elements.iter_mut().for_each(|e| e.hide())
    }

    async fn create_elements(renderer: &mut Renderer) -> Vec<UIElement> {
        let play_btn = UIElement {
            quad: primitives::quad::<D2Instance>(
                renderer,
                vec![D2Instance {
                    position: Vec2::new(RESOLUTION_X * 0.5, RESOLUTION_Y * 0.5),
                    scale: Vec2::new(0.0, 0.0),
                    rotation: 0.0,
                }],
            ),
            on_press: UIAction::Play,
            texture_bg: {
                let texture = load_texture("play_btn.png", renderer, TextureDescriptor::default())
                    .await
                    .expect("kys");
                let texture = renderer.add_texture(texture);
                let texture = TextureBindGroup::new(renderer, texture);
                renderer.add_bind_group(texture).into()
            },
            visible: false,
            state: UIElementState::Idle(0.0),
            idle_size: Vec2::new(222.0, 220.0),
            hovered_scale: 1.2,
            should_update: false,
        };

        let edit_btn = UIElement {
            quad: primitives::quad::<D2Instance>(
                renderer,
                vec![D2Instance {
                    position: Vec2::new(RESOLUTION_X * 0.5, RESOLUTION_Y * 0.5 - 222.0),
                    scale: Vec2::new(0.0, 0.0),
                    rotation: 0.0,
                }],
            ),
            on_press: UIAction::Create,
            texture_bg: {
                let texture =
                    load_texture("edit_button.png", renderer, TextureDescriptor::default())
                        .await
                        .expect("kys");
                let texture = renderer.add_texture(texture);
                let texture = TextureBindGroup::new(renderer, texture);
                renderer.add_bind_group(texture).into()
            },
            visible: false,
            state: UIElementState::Idle(0.0),
            idle_size: Vec2::new(81.0, 83.0),
            hovered_scale: 1.2,
            should_update: false,
        };

        let pause_btn = UIElement {
            quad: primitives::quad::<D2Instance>(
                renderer,
                vec![D2Instance {
                    position: Vec2::new(39.0, RESOLUTION_Y - 40.0),
                    scale: Vec2::new(0.0, 0.0),
                    rotation: 0.0,
                }],
            ),
            on_press: UIAction::Pause,
            texture_bg: {
                let texture = load_texture("pause_btn.png", renderer, TextureDescriptor::default())
                    .await
                    .expect("kys");
                let texture = renderer.add_texture(texture);
                let texture = TextureBindGroup::new(renderer, texture);
                renderer.add_bind_group(texture).into()
            },
            visible: false,
            state: UIElementState::Idle(0.0),
            idle_size: Vec2::new(39.0, 40.0),
            hovered_scale: 1.2,
            should_update: false,
        };

        let exit_btn = UIElement {
            quad: primitives::quad::<D2Instance>(
                renderer,
                vec![D2Instance {
                    position: Vec2::new(64.0, RESOLUTION_Y - 64.0),
                    scale: Vec2::new(0.0, 0.0),
                    rotation: 0.0,
                }],
            ),
            on_press: UIAction::Exit,
            texture_bg: {
                let texture = load_texture("menu_btn.png", renderer, TextureDescriptor::default())
                    .await
                    .expect("kys");
                let texture = renderer.add_texture(texture);
                let texture = TextureBindGroup::new(renderer, texture);
                renderer.add_bind_group(texture).into()
            },
            visible: false,
            state: UIElementState::Idle(0.0),
            idle_size: Vec2::new(63.0, 64.0),
            hovered_scale: 1.2,
            should_update: false,
        };

        let resume_btn = UIElement {
            quad: primitives::quad::<D2Instance>(
                renderer,
                vec![D2Instance {
                    position: Vec2::new(RESOLUTION_X * 0.5, RESOLUTION_Y * 0.5),
                    scale: Vec2::new(0.0, 0.0),
                    rotation: 0.0,
                }],
            ),
            on_press: UIAction::Resume,
            texture_bg: {
                let texture = load_texture(
                    "playepaused_btn.png",
                    renderer,
                    TextureDescriptor::default(),
                )
                .await
                .expect("kys");
                let texture = renderer.add_texture(texture);
                let texture = TextureBindGroup::new(renderer, texture);
                renderer.add_bind_group(texture).into()
            },
            visible: false,
            state: UIElementState::Idle(0.0),
            idle_size: Vec2::new(81.0, 81.0),
            hovered_scale: 1.2,
            should_update: false,
        };

        vec![play_btn, edit_btn, pause_btn, exit_btn, resume_btn]
    }
}

impl UIElement {
    pub fn show(&mut self) {
        self.visible = true;
        self.state = UIElementState::Idle(0.0);
        self.should_update = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.quad
            .instances
            .iter_mut()
            .for_each(|instance| instance.scale = self.idle_size);
        self.should_update = true;
    }
    pub fn update(&mut self, dt: f32) {
        match &mut self.state {
            UIElementState::Hovered(time) => self.quad.instances.iter_mut().for_each(|instance| {
                *time += dt;
                instance.scale = instance
                    .scale
                    .lerp(self.idle_size * self.hovered_scale, *time);
            }),
            UIElementState::Idle(time) => self.quad.instances.iter_mut().for_each(|instance| {
                *time += dt;
                instance.scale = instance.scale.lerp(self.idle_size, *time);
            }),
            UIElementState::Pressed => {}
        }
    }
}
