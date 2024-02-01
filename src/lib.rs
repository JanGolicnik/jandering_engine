use jandering_engine::engine::{Engine, EngineDescriptor};
use jandering_engine::object::{primitives, Instance};
use jandering_engine::renderer::Renderer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Coultn init");

    let engine_descriptor = EngineDescriptor {
        plugins: vec![
            Box::<jandering_engine::plugins::time::TimePlugin>::default(),
            Box::<jandering_engine::plugins::resolution::ResolutionPlugin>::default(),
        ],
        resolution: (500, 500),
        ..Default::default()
    };
    let engine = Engine::new(engine_descriptor);

    engine
        .renderer
        .device
        .on_uncaptured_error(Box::new(move |e| log::error!("{:?}", e)));

    let quad = primitives::quad(&engine.renderer, vec![Instance::default()]);
    let mut objects = vec![quad];

    let doc = web_sys::window().and_then(|win| win.document()).unwrap();

    engine.run(move |renderer, encoder, plugins, surface, shaders, _, _| {
        if let Some(new_shader) = get_shader(&doc) {
            let bind_group_layouts = plugins
                .iter()
                .map(|e| e.get_bind_group_layouts())
                .filter(|e| e.is_some())
                .flat_map(|e| e.unwrap())
                .collect();
            renderer
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation);
            let new_shader = shader_from_source(new_shader, renderer, bind_group_layouts);
            if let Some(wgpu::Error::Validation { description, .. }) =
                pollster::block_on(renderer.device.pop_error_scope())
            {
                print_error(&doc, description);
                return;
            } else {
                print_error(&doc, "".to_string());
            }

            if shaders.len() == 0 {
                shaders.push(new_shader);
            } else {
                shaders[0] = new_shader;
            }
        }

        renderer.render(&mut objects, encoder, plugins, surface, shaders);
    });
}

fn print_error(doc: &web_sys::Document, mut err: String) {
    let el = doc
        .get_element_by_id("wgsltoy_error_box")
        .expect("should have #wgsltoy_error_box on the page");
    err = err.replace("\n", "<br>");
    el.set_inner_html(&err);
}

fn get_shader(doc: &web_sys::Document) -> Option<String> {
    if should_update_shader(&doc) {
        Some(get_shader_code(&doc))
    } else {
        None
    }
}

fn get_shader_code(doc: &web_sys::Document) -> String {
    let el = doc
        .get_element_by_id("wgsltoy_shadercode")
        .expect("should have #wgsltoy_shadercode on the page");

    let textarea = el
        .dyn_ref::<web_sys::HtmlTextAreaElement>()
        .expect("#wgsltoy_shadercode should be an `HtmlTextAreaElement`");

    textarea.value()
}

fn should_update_shader(doc: &web_sys::Document) -> bool {
    let el = doc
        .get_element_by_id("wgsltoy_updateshader")
        .expect("should have #wgsltoy_updateshader on the page");

    let input = el
        .dyn_ref::<web_sys::HtmlInputElement>()
        .expect("#wgsltoy_updateshader should be an `HtmlInputElement`");

    if input.value() == "true" {
        input.set_value("");
        return true;
    }

    false
}

fn shader_from_source(
    source: String,
    renderer: &Renderer,
    bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
) -> wgpu::RenderPipeline {
    let layout = renderer
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader Layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

    let source = format!("{}{source}", include_str!("shader_base.wgsl"));

    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    };
    let shader = renderer.device.create_shader_module(shader);
    renderer
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    jandering_engine::object::VertexRaw::desc(),
                    jandering_engine::object::InstanceRaw::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.config.format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::OVER,
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
}
