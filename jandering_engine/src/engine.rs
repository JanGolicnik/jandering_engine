use crate::renderer::Janderer;

use super::renderer::Renderer;

use je_windowing::{Window, WindowConfig, WindowManager, WindowManagerTrait};

pub struct Engine {
    pub renderer: Renderer,
    pub window_manager: WindowManager,
}

impl Engine {
    pub async fn default() -> Self {
        Self::new(EngineConfig::default()).await
    }
}

#[derive(Default)]
pub struct EngineConfig {
    pub enable_compute: bool,
}

impl Engine {
    pub async fn new(config: EngineConfig) -> Self {
        let renderer = Renderer::new(config).await;
        let window_manager = WindowManager::new();

        Self {
            renderer,
            window_manager,
        }
    }

    pub fn spawn_window(&mut self, window_config: WindowConfig) -> Window {
 self.window_manager.spawn_window(window_config)
    }

    pub fn run(self, mut function: impl FnMut(&mut Renderer, &mut WindowManager)){
        let Engine{
            mut renderer,
            window_manager
        } = self;

        window_manager.run(move |window_manager| {
            function(&mut renderer, window_manager);
            renderer.present();
        });
    }
}
