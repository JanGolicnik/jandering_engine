use jandering_engine::{
    bind_group::camera::d2::D2CameraBindGroup, engine::Engine, renderer::BindGroupHandle,
};
use map::Map;
use player::Player;

use post_processing::PostProcessing;

pub mod constants;
pub mod definition;
pub mod hue_plugin;
pub mod map;
pub mod player;
pub mod post_processing;

#[derive(PartialEq, Eq, Copy, Clone)]
enum GameState {
    MainMenu,
    SetupPlaying,
    Playing,
    Creating,
}

pub struct Game {
    engine: Engine,
    player: Player,
    map: Map,
    post_processing: PostProcessing,
    camera_bg: BindGroupHandle<D2CameraBindGroup>,
    state: GameState,
}
