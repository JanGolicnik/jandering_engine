use std::{collections::HashMap, ops::Range};

use crate::{
    object::Renderable,
    renderer::{ShaderHandle, TextureHandle, UntypedBindGroupHandle},
    types::Vec3,
};

pub struct RenderPassData {
    pub(crate) shader: ShaderHandle,
    pub(crate) bind_groups: HashMap<u32, UntypedBindGroupHandle>,
    pub(crate) clear_color: Option<Vec3>,
    pub(crate) depth: Option<f32>,
    pub(crate) depth_tex: Option<TextureHandle>,
    pub(crate) target: Option<TextureHandle>,
    pub(crate) resolve_target: Option<TextureHandle>,
    pub(crate) alpha: f32,
}

impl Default for RenderPassData {
    fn default() -> Self {
        Self {
            shader: ShaderHandle(0),
            bind_groups: Default::default(),
            clear_color: Default::default(),
            depth: Default::default(),
            depth_tex: Default::default(),
            target: Default::default(),
            resolve_target: Default::default(),
            alpha: Default::default(),
        }
    }
}

pub trait RenderPassTrait {
    fn get_data(&mut self) -> &mut RenderPassData;

    fn render(self, renderables: &[&dyn Renderable]) -> Self
    where
        Self: Sized,
    {
        let mut ret = self;
        for renderable in renderables {
            ret = ret.render_range(*renderable, 0..renderable.num_instances());
        }
        ret
    }

    fn render_one(self, renderable: &dyn Renderable) -> Self
    where
        Self: Sized,
    {
        self.render_range(renderable, 0..renderable.num_instances())
    }

    fn render_range(self, renderables: &dyn Renderable, range: Range<u32>) -> Self
    where
        Self: Sized;

    fn bind(mut self, slot: u32, bind_group: UntypedBindGroupHandle) -> Self
    where
        Self: Sized,
    {
        self.get_data()
            .bind_groups
            .entry(slot)
            .and_modify(|e| *e = bind_group)
            .or_insert(bind_group);
        self
    }

    fn unbind(mut self, slot: u32) -> Self
    where
        Self: Sized,
    {
        self.get_data().bind_groups.remove(&slot);
        self
    }

    fn submit(self);

    fn set_shader(mut self, shader: ShaderHandle) -> Self
    where
        Self: Sized,
    {
        self.get_data().shader = shader;
        self
    }

    fn with_depth(mut self, texture: TextureHandle, value: Option<f32>) -> Self
    where
        Self: Sized,
    {
        let data = self.get_data();
        data.depth_tex = Some(texture);
        data.depth = value;
        self
    }

    fn with_clear_color(mut self, r: f32, g: f32, b: f32) -> Self
    where
        Self: Sized,
    {
        self.get_data().clear_color = Some(Vec3::new(r, g, b));
        self
    }

    fn with_alpha(mut self, alpha: f32) -> Self
    where
        Self: Sized,
    {
        let data = self.get_data();
        data.alpha = alpha;
        if data.clear_color.is_none() {
            data.clear_color = Some(Vec3::splat(1.0));
        }
        self
    }

    //  None for resolve target means use canvas
    #[cfg(not(target_arch = "wasm32"))]
    fn with_target_texture_resolve(
        mut self,
        target: TextureHandle,
        resolve: Option<TextureHandle>,
    ) -> Self
    where
        Self: Sized,
    {
        let data = self.get_data();
        data.target = Some(target);
        data.resolve_target = resolve;
        self
    }
}
