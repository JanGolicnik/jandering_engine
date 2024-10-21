use std::path::Path;

use crate::renderer::Janderer;

use super::renderer::Renderer;

use je_windowing::{Window, WindowConfig, WindowManager, WindowManagerTrait};
use notify::{event::ModifyKind, EventKind};

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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_events(
        self,
        mut function: impl FnMut(&mut Renderer, &mut WindowManager, &[EngineEvent]),
    ) {
        use notify::{Config, RecommendedWatcher, Watcher};

        let Engine {
            mut renderer,
            window_manager,
        } = self;

        let (file_change_sender, file_change_receiver) = std::sync::mpsc::channel();
        let mut file_watcher =
            RecommendedWatcher::new(file_change_sender, Config::default()).unwrap();
        file_watcher
            .watch(
                Path::new("./res").as_ref(),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();

        window_manager.run(move |window_manager| {
            let events = file_change_receiver
                .try_iter()
                .flatten()
                .filter(|e| {
                    matches!(
                        e.kind,
                        EventKind::Modify(ModifyKind::Data(
                            notify::event::DataChange::Content | notify::event::DataChange::Any
                        ))
                    )
                })
                .flat_map(|e| {
                    e.paths
                        .iter()
                        .flat_map(|e| e.file_name())
                        .map(|e| EngineEvent::FileChanged(e.to_str().unwrap().to_string()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            function(&mut renderer, window_manager, &events);
            renderer.present();
        });
    }

    pub fn run(self, mut function: impl FnMut(&mut Renderer, &mut WindowManager)) {
        let Engine {
            mut renderer,
            window_manager,
        } = self;

        window_manager.run(move |window_manager| {
            function(&mut renderer, window_manager);
            renderer.present();
        });
    }
}

#[derive(Debug)]
pub enum EngineEvent {
    FileChanged(String),
}
