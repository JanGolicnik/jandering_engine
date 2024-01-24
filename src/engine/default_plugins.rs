use crate::{camera::DefaultCameraPlugin, plugins::Plugin};

pub fn default_plugins() -> Vec<Box<dyn Plugin>> {
    vec![Box::new(DefaultCameraPlugin::new())]
}
