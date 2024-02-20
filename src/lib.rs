use jandering_engine::bind_group::resolution::ResolutionBindGroup;
use jandering_engine::bind_group::time::TimeBindGroup;
use jandering_engine::engine::{Engine, EngineDescriptor};
use jandering_engine::object::{primitives, Instance, VertexRaw};
use jandering_engine::shader::ShaderDescriptor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Coultn init");

    let engine_descriptor = EngineDescriptor {
        resolution: (500, 500),
    };
    let mut engine = Engine::new(engine_descriptor);

    engine
        .renderer
        .device
        .on_uncaptured_error(Box::new(move |e| log::error!("{:?}", e)));

    let time_bg = engine
        .renderer
        .add_bind_group(TimeBindGroup::new(&engine.renderer));
    let resolution_bg = engine
        .renderer
        .add_bind_group(ResolutionBindGroup::new(&engine.renderer));
    let bind_groups = [time_bg.into(), resolution_bg.into()];

    let mut shader = None;

    let quad = primitives::quad(&engine.renderer, vec![Instance::default()]);

    let doc = web_sys::window().and_then(|win| win.document()).unwrap();

    engine.run(move |context, renderer| {
        if let Some(new_shader) = get_shader(&doc) {
            renderer
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation);

            shader = Some(jandering_engine::shader::create_shader(
                renderer,
                ShaderDescriptor {
                    code: format!("{}{new_shader}", include_str!("shader_base.wgsl")).as_str(),
                    descriptors: &[VertexRaw::desc(), Instance::desc()],
                    bind_groups: &bind_groups,
                    ..Default::default()
                },
            ));

            if let Some(wgpu::Error::Validation { description, .. }) =
                pollster::block_on(renderer.device.pop_error_scope())
            {
                print_error(&doc, description);
                return;
            }

            print_error(&doc, "".to_string());
        }

        if let Some(shader) = shader.as_ref() {
            let time_bind_group: &mut TimeBindGroup =
                renderer.get_bind_group_t_mut(time_bg).unwrap();
            time_bind_group.update(context.dt as f32);
            renderer.render(&[&quad], context, shader, &bind_groups);
        }
    });
}

fn print_error(doc: &web_sys::Document, mut err: String) {
    let el = doc
        .get_element_by_id("wgsltoy_error_box")
        .expect("should have #wgsltoy_error_box on the page");
    err = err.replace('\n', "<br>");
    el.set_inner_html(&err);
}

fn get_shader(doc: &web_sys::Document) -> Option<String> {
    if should_update_shader(doc) {
        Some(get_shader_code(doc))
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
