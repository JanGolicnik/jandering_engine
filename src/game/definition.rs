use crate::game::constants::HALF_TILE_SIZE;

use super::{
    constants::{RESOLUTION_X, RESOLUTION_Y, TILE_SIZE},
    map::{tiles::GROUND, Map},
    player::Player,
    post_processing::PostProcessing,
    ui::UserInterface,
    GameState,
};
use jandering_engine::{
    bind_group::{camera::d2::D2CameraBindGroup, resolution::ResolutionBindGroup},
    engine::{Engine, EngineContext, EngineDescriptor},
    renderer::{BindGroupHandle, Renderer},
    types::Vec2,
    utils::FilePath,
};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use super::Game;

impl Game {
    pub async fn new() -> Self {
        let mut engine = Engine::new(EngineDescriptor {
            resolution: (RESOLUTION_X as u32, RESOLUTION_Y as u32),
        });

        let mut camera = D2CameraBindGroup::new(&engine.renderer, false);
        camera.data.position = Vec2::new(-RESOLUTION_X * 0.5, -RESOLUTION_Y * 0.5 + HALF_TILE_SIZE);
        camera.update_uniform();
        let camera_bg: BindGroupHandle<D2CameraBindGroup> = engine.renderer.add_bind_group(camera);
        let resolution_bg: BindGroupHandle<ResolutionBindGroup> = engine
            .renderer
            .add_bind_group(ResolutionBindGroup::new(&engine.renderer));

        let map = Map::new(
            &mut engine.renderer,
            FilePath::FileName("test_map.map"),
            camera_bg,
        )
        .await;

        let player = Player::new(&mut engine.renderer, camera_bg).await;
        let post_processing = PostProcessing::new(&mut engine.renderer, resolution_bg).await;
        let ui = UserInterface::new(&mut engine.renderer, camera_bg, resolution_bg).await;

        Self {
            engine,
            player,
            map,
            camera_bg,
            post_processing,
            state: super::GameState::Init,
            ui,
        }
    }

    pub fn run(self) {
        let Self {
            engine,
            mut player,
            mut map,
            mut state,
            mut post_processing,
            camera_bg,
            mut ui,
            ..
        } = self;

        #[allow(unused_assignments)]
        let mut world_mouse_pos: Option<Vec2> = None;
        #[allow(unused_assignments)]
        let mut resolution = Vec2::new(0.0, 0.0);

        let mut time = 0.0;

        let mut paused = false;

        engine.run(move |context, renderer: &mut Renderer| {
            time += context.dt as f32;
            if time > 1.0 {
                time = 0.0;
                let result = web_sys::HtmlAudioElement::new_with_src("res/MJ.mp3");
                let _ = result.unwrap().play();
            }

            let popr_target_texture = post_processing.get_texture_handle();
            renderer.set_render_target(popr_target_texture);
            renderer.clear_texture(
                context.encoder,
                popr_target_texture,
                wgpu::Color {
                    r: 0.01,
                    g: 0.008,
                    b: 0.006,
                    a: 1.0,
                },
            );

            resolution = Vec2::new(renderer.config.width as f32, renderer.config.height as f32);

            for event in context.events.iter() {
                if let WindowEvent::CursorMoved { position, .. } = event {
                    let mut screen_pos = Vec2::new(
                        position.x as f32,
                        renderer.config.height as f32 - position.y as f32,
                    );
                    screen_pos -= Vec2::new(resolution.x * 0.5, resolution.y * 0.5);
                    world_mouse_pos = Some(
                        screen_pos - renderer.get_bind_group_t(camera_bg).unwrap().data.position,
                    );
                }
            }

            loop {
                let prev_state = state;
                if let Some(action) =
                    ui.update(context, renderer, world_mouse_pos, context.dt as f32)
                {
                    match action {
                        super::ui::UIAction::Play => {
                            ui.show_playing();
                            paused = false;
                            state = GameState::SetupPlaying
                        }
                        super::ui::UIAction::Create => {
                            ui.show_creating();
                            state = GameState::Creating
                        }
                        super::ui::UIAction::Exit => {
                            ui.show_mainmenu();
                            state = GameState::MainMenu
                        }
                        super::ui::UIAction::Pause => {
                            ui.show_paused();
                            paused = true;
                        }
                        super::ui::UIAction::Resume => {
                            ui.show_playing();
                            paused = false;
                        }
                    }
                }

                match state {
                    GameState::MainMenu => {
                        Self::update_mainmenu(context, renderer, &mut map, &mut ui)
                    }
                    GameState::SetupPlaying => {
                        Self::update_setupplaying(&mut player, &mut map);
                        state = GameState::Playing;
                    }
                    GameState::Playing => {
                        Self::update_playing(context, renderer, &mut player, &mut map, paused)
                    }
                    GameState::Creating => {
                        Self::update_creating(context, renderer, &mut map, &world_mouse_pos)
                    }
                    GameState::Init => {
                        ui.show_mainmenu();
                        state = GameState::MainMenu
                    }
                }

                ui.render(context, renderer);
                if prev_state == state {
                    break;
                }
            }

            post_processing.update(renderer, context);
            post_processing.render_bloom(renderer, context);
            post_processing.render_tonemap(renderer, context);
        });
    }

    fn update_setupplaying(player: &mut Player, map: &mut Map) {
        player.reset(Vec2::new(0.0, TILE_SIZE * 2.0));
        map.position.x = 0.0;
        map.hue = 0.0;
    }

    fn update_playing(
        context: &mut EngineContext,
        renderer: &mut Renderer,
        player: &mut Player,
        map: &mut Map,
        paused: bool,
    ) {
        if !paused {
            if !player.is_alive() {
                player.reset(Vec2::new(0.0, TILE_SIZE * 2.0));
                map.position.x = 0.0;
            }
            if player.is_alive() {
                player.update(context, map);
            }

            let speed = context.dt as f32 * 5.0 * TILE_SIZE;
            if player.position.x < 6.0 * TILE_SIZE {
                player.position.x += speed;
            } else {
                map.position.x -= speed;
            }

            player.hue = map.hue + 0.5;
        }

        map.render(context, renderer);
        player.render(context, renderer);
    }

    fn update_mainmenu(
        context: &mut EngineContext,
        renderer: &mut Renderer,
        map: &mut Map,
        ui: &mut UserInterface,
    ) {
        map.position.x = 0.0;
        map.render(context, renderer);
        ui.render(context, renderer);
    }

    fn update_creating(
        context: &mut EngineContext,
        renderer: &mut Renderer,
        map: &mut Map,
        mouse_pos: &Option<Vec2>,
    ) {
        for event in context.events.iter() {
            match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match keycode {
                    VirtualKeyCode::D => {
                        map.position.x -= TILE_SIZE;
                    }
                    VirtualKeyCode::A => {
                        map.position.x += TILE_SIZE;
                    }
                    _ => {}
                },
                WindowEvent::MouseInput {
                    state: winit::event::ElementState::Pressed,
                    button: winit::event::MouseButton::Left,
                    ..
                } => {
                    if let Some(mouse_pos) = mouse_pos {
                        map.set_tile(mouse_pos.x as u32, mouse_pos.y as u32, GROUND);
                    }
                }
                _ => {}
            }
        }
        map.render(context, renderer);
    }
}
