use std::{collections::HashMap, ops::Range};

use crate::{
    object::Renderable,
    renderer::{
        BufferHandle, Renderer, ShaderHandle, TargetTexture, TextureHandle, UntypedBindGroupHandle,
    },
    types::Vec3,
};

use je_windowing::Window;

#[derive(Clone, Default, Debug)]
pub enum RenderAction {
    Mesh {
        vertex_buffer_handle: BufferHandle,
        index_buffer_handle: BufferHandle,
        instance_buffer_handle: Option<BufferHandle>,
        range: Range<u32>,
        num_indices: u32,
    },
    #[default]
    Empty,
}

#[derive(Clone, Debug)]
pub struct RenderStep {
    pub(crate) action: RenderAction,

    pub(crate) shader: Option<ShaderHandle>,
    pub(crate) bind_groups: HashMap<u32, UntypedBindGroupHandle>,

    pub(crate) target: TargetTexture,
    pub(crate) depth_tex: Option<TextureHandle>,
    pub(crate) resolve_target: Option<TargetTexture>,

    pub(crate) alpha: f32,
    pub(crate) depth: Option<f32>,
    pub(crate) clear_color: Option<Vec3>,
}

impl Default for RenderStep {
    fn default() -> Self {
        Self {
            action: Default::default(),
            shader: Default::default(),
            bind_groups: Default::default(),
            target: Default::default(),
            depth_tex: Default::default(),
            resolve_target: Default::default(),
            alpha: 1.0,
            depth: Default::default(),
            clear_color: Default::default(),
        }
    }
}

pub struct RenderPass<'renderer> {
    pub(crate) renderer: &'renderer mut Renderer,
    pub(crate) window: &'renderer mut Window,
    pub(crate) steps: Vec<RenderStep>,
}

impl<'renderer> RenderPass<'renderer> {
    pub fn new(renderer: &'renderer mut Renderer, window: &'renderer mut Window) -> Self {
        Self {
            renderer,
            window,
            steps: vec![RenderStep::default()],
        }
    }

    pub fn render_empty(&mut self) -> &mut Self {
        let step = self.steps.last_mut().unwrap();
        step.action = RenderAction::Empty;
        let mut next_step = step.clone();
        next_step.depth = None;
        next_step.clear_color = None;
        next_step.alpha = 1.0;
        self.steps.push(next_step);
        self
    }

    pub fn render(&mut self, renderables: &[&impl Renderable]) -> &mut Self {
        for renderable in renderables {
            self.render_range(*renderable, 0..renderable.num_instances());
        }
        self
    }

    pub fn render_one(&mut self, renderable: &impl Renderable) -> &mut Self {
        let range = 0..renderable.num_instances();
        self.render_range(renderable, range)
    }

    pub fn render_range(&mut self, renderable: &impl Renderable, range: Range<u32>) -> &mut Self {
        let step = self.steps.last_mut().unwrap();
        let (vertex_buffer_handle, index_buffer_handle, instance_buffer_handle) =
            renderable.get_buffers();
        let num_indices = renderable.num_indices();
        step.action = RenderAction::Mesh {
            vertex_buffer_handle,
            index_buffer_handle,
            instance_buffer_handle,
            range,
            num_indices,
        };
        let mut next_step = step.clone();
        next_step.depth = None;
        next_step.clear_color = None;
        next_step.alpha = 1.0;
        self.steps.push(next_step);
        self
    }

    pub fn bind(&mut self, slot: u32, bind_group: UntypedBindGroupHandle) -> &mut Self {
        self.steps
            .last_mut()
            .unwrap()
            .bind_groups
            .entry(slot)
            .and_modify(|e| *e = bind_group)
            .or_insert(bind_group);
        self
    }

    pub fn unbind(&mut self, slot: u32) -> &mut Self {
        self.steps.last_mut().unwrap().bind_groups.remove(&slot);
        self
    }

    pub fn submit(self) {
        Renderer::submit_pass(self);
    }

    pub fn set_shader(&mut self, shader: ShaderHandle) -> &mut Self {
        self.steps.last_mut().unwrap().shader = Some(shader);
        self
    }

    pub fn with_depth(&mut self, texture: TextureHandle, value: Option<f32>) -> &mut Self {
        let data = self.steps.last_mut().unwrap();
        data.depth_tex = Some(texture);
        data.depth = value;
        self
    }

    pub fn without_depth(&mut self) -> &mut Self {
        let data = self.steps.last_mut().unwrap();
        data.depth_tex = None;
        data.depth = None;
        self
    }

    pub fn with_clear_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.steps.last_mut().unwrap().clear_color = Some(Vec3::new(r, g, b));
        self
    }

    pub fn with_alpha(&mut self, alpha: f32) -> &mut Self {
        let data = self.steps.last_mut().unwrap();
        data.alpha = alpha;
        if data.clear_color.is_none() {
            data.clear_color = Some(Vec3::splat(1.0));
        }
        self
    }

    //  None for resolve target means use canvas
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_target_texture_resolve(
        &mut self,
        target: TargetTexture,
        resolve: Option<TargetTexture>,
    ) -> &mut Self {
        let data = self.steps.last_mut().unwrap();
        data.target = target;
        data.resolve_target = resolve;
        self
    }
}
