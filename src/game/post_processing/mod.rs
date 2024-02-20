use jandering_engine::{
    bind_group::{resolution::ResolutionBindGroup, texture::TextureBindGroup},
    engine::EngineContext,
    object::{primitives, D2Instance, Object, VertexRaw},
    renderer::{BindGroupHandle, Renderer, TextureHandle, UntypedBindGroupHandle},
    shader::Shader,
    texture::Texture,
    types::UVec2,
};
use wgpu::{ColorTargetState, Extent3d, ImageCopyTexture, Origin3d};

use self::bind_groups::FactorBindGroup;

const BLOOM_SHADER_PASSSES: usize = 4;

mod bind_groups;

pub struct PostProcessing {
    quad: Object<D2Instance>,

    tonemap_shader: Shader,

    bind_groups: [UntypedBindGroupHandle; 2],
    target_texture: TextureHandle,
    read_texture_bg: BindGroupHandle<TextureBindGroup>,
    read_texture: TextureHandle,

    bloom_textures: Vec<TextureHandle>,

    add_bind_groups: [UntypedBindGroupHandle; 2],
    factor_bg: BindGroupHandle<FactorBindGroup>,
    add_shader: Shader,
    blit_shader: Shader,
    threshold_blit_shader: Shader,
}

impl PostProcessing {
    pub async fn new(
        renderer: &mut Renderer,
        resolution_bg: BindGroupHandle<ResolutionBindGroup>,
    ) -> Self {
        let quad = primitives::quad(renderer, vec![D2Instance::default()]);

        let target_texture = renderer.add_texture(Texture::new_color(
            renderer,
            UVec2::new(renderer.config.width, renderer.config.height),
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            Some(wgpu::TextureFormat::Rgba16Float),
        ));

        let read_texture = renderer.add_texture(Texture::new_color(
            renderer,
            UVec2::new(renderer.config.width, renderer.config.height),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            Some(wgpu::TextureFormat::Rgba16Float),
        ));

        let read_texture_bind_group = TextureBindGroup::new(renderer, read_texture);
        let read_texture_bg = renderer.add_bind_group(read_texture_bind_group);
        let bind_groups = [resolution_bg.into(), read_texture_bg.into()];

        let factor_bg = renderer.add_bind_group(FactorBindGroup::new(renderer));
        let add_bind_groups = [factor_bg.into(), read_texture_bg.into()];

        let bloom_shader_code = include_str!("bloom_shader.wgsl");

        let tonemap_shader = jandering_engine::shader::create_shader(
            renderer,
            jandering_engine::shader::ShaderDescriptor {
                code: bloom_shader_code,
                descriptors: &[VertexRaw::desc(), D2Instance::desc()],
                bind_groups: &bind_groups,
                fs_entry: "fs_tonemap",
                ..Default::default()
            },
        );

        let blit_shader = jandering_engine::shader::create_shader(
            renderer,
            jandering_engine::shader::ShaderDescriptor {
                code: bloom_shader_code,
                descriptors: &[VertexRaw::desc(), D2Instance::desc()],
                bind_groups: &bind_groups,
                targets: Some(&[Some(ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]),
                fs_entry: "fs_blit",
                ..Default::default()
            },
        );

        let threshold_blit_shader = jandering_engine::shader::create_shader(
            renderer,
            jandering_engine::shader::ShaderDescriptor {
                code: bloom_shader_code,
                descriptors: &[VertexRaw::desc(), D2Instance::desc()],
                bind_groups: &bind_groups,
                targets: Some(&[Some(ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]),
                fs_entry: "fs_thresholdblit",
                ..Default::default()
            },
        );

        let add_shader = jandering_engine::shader::create_shader(
            renderer,
            jandering_engine::shader::ShaderDescriptor {
                code: include_str!("add_shader.wgsl"),
                descriptors: &[VertexRaw::desc(), D2Instance::desc()],
                bind_groups: &bind_groups,
                targets: Some(&[Some(ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })]),
                ..Default::default()
            },
        );
        let bloom_textures = (0..BLOOM_SHADER_PASSSES)
            .map(|n| {
                let scale = 2u32.pow(n as u32);
                renderer.add_texture(Texture::new_color(
                    renderer,
                    UVec2::new(
                        renderer.config.width / scale,
                        renderer.config.height / scale,
                    ),
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    Some(wgpu::TextureFormat::Rgba16Float),
                ))
            })
            .collect();

        Self {
            quad,

            tonemap_shader,

            bind_groups,
            target_texture,
            read_texture_bg,
            read_texture,

            bloom_textures,

            add_bind_groups,
            factor_bg,
            add_shader,
            blit_shader,
            threshold_blit_shader,
        }
    }

    fn update_render_data(&mut self, _context: &mut EngineContext, _renderer: &mut Renderer) {}

    pub fn update(&mut self, renderer: &mut Renderer, context: &mut EngineContext) {
        self.update_render_data(context, renderer);
        self.quad.update(context, renderer);
    }

    pub fn set_read_texture(
        &mut self,
        renderer: &mut Renderer,
        texture: TextureHandle,
    ) -> TextureHandle {
        let texture_bind_group = renderer.get_bind_group_t(self.read_texture_bg).unwrap();
        let device = &renderer.device;
        let texture = renderer.get_texture(texture).unwrap();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });
        let prev_handle = texture_bind_group.texture_handle;
        let texture_bind_group = renderer.get_bind_group_t_mut(self.read_texture_bg).unwrap();
        texture_bind_group.bind_group = bind_group;
        prev_handle
    }

    pub fn render_bloom(&mut self, renderer: &mut Renderer, context: &mut EngineContext) {
        Self::copy_from_to(renderer, context, self.target_texture, self.read_texture);

        self.set_read_texture(renderer, self.read_texture);

        // downsample to bloom 1, then from bloom 1 to bloom 2, ... then from bloom n - 1  to bloom n..
        for i in 0..BLOOM_SHADER_PASSSES {
            let tex = self.bloom_textures[i];
            renderer.set_render_target(tex);
            let shader = if i == 0 {
                &self.threshold_blit_shader
            } else {
                &self.blit_shader
            };
            renderer.render(&[&self.quad], context, shader, &self.bind_groups);
            self.set_read_texture(renderer, tex);
        }

        // upsample from bloom n to bloom n - 1, ... then from bloom 2 to bloom 1
        {
            let factor_bind_group = renderer.get_bind_group_t_mut(self.factor_bg).unwrap();
            factor_bind_group.uniform.factor = 1.0;
        }
        for i in (1..BLOOM_SHADER_PASSSES).rev() {
            let target = self.bloom_textures[i - 1];
            renderer.set_render_target(target);
            let src = self.bloom_textures[i];
            self.set_read_texture(renderer, src);
            renderer.render(
                &[&self.quad],
                context,
                &self.add_shader,
                &self.add_bind_groups,
            );
        }

        // add bloom 1 to original image
        {
            let factor_bind_group = renderer.get_bind_group_t_mut(self.factor_bg).unwrap();
            factor_bind_group.uniform.factor = 0.8;
        }

        self.set_read_texture(renderer, self.bloom_textures[0]);
        renderer.set_render_target(self.target_texture);
        renderer.render(
            &[&self.quad],
            context,
            &self.add_shader,
            &self.add_bind_groups,
        );
    }

    pub fn render_tonemap(&mut self, renderer: &mut Renderer, context: &mut EngineContext) {
        Self::copy_from_to(renderer, context, self.target_texture, self.read_texture);
        self.set_read_texture(renderer, self.read_texture);
        renderer.set_target_surface();
        renderer.render(
            &[&self.quad],
            context,
            &self.tonemap_shader,
            &self.bind_groups,
        );
    }

    fn copy_from_to(
        renderer: &mut Renderer,
        context: &mut EngineContext,
        src: TextureHandle,
        target: TextureHandle,
    ) {
        let source_tex = renderer.get_texture(src).unwrap();
        let source = ImageCopyTexture {
            texture: source_tex.texture.as_ref().unwrap(),
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };
        let dest_tex = renderer.get_texture(target).unwrap();
        let dest = ImageCopyTexture {
            texture: dest_tex.texture.as_ref().unwrap(),
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };
        context.encoder.copy_texture_to_texture(
            source,
            dest,
            Extent3d {
                width: dest_tex.width().min(source_tex.width()),
                height: dest_tex.height().min(source_tex.height()),
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn get_texture_handle(&self) -> TextureHandle {
        self.target_texture
    }
}
