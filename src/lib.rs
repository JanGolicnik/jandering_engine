use game::Game;
use wasm_bindgen::prelude::*;
mod game;

#[wasm_bindgen(start)]
async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Coultn init");

    let game = Game::new().await;
    game.run();
}
