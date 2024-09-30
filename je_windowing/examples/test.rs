use std::time::Duration;

use je_windowing::{WindowConfig, WindowManager, WindowManagerTrait, WindowTrait};

// fn main() {
//     let mut window_manager = WindowManager::new();

//     let mut resolution = 250;

//     let mut window = window_manager.spawn_window(
//         // also registers window by itself
//         WindowConfig::default()
//             .with_cursor(true)
//             .with_resolution(250, 250)
//             // .with_decorations(false)
//             // .with_transparency(true)
//             .with_title("beast"),
//     );

//     while !window.should_close() {
//         resolution += 1;
//         window.resize(resolution, resolution);
//         std::thread::sleep(Duration::from_millis(1));
//     }
// }

fn main() {
    let mut window_manager = WindowManager::new();

    let mut resolution = 250;

    let mut window = window_manager.spawn_window(
        WindowConfig::default()
            .with_cursor(true)
            .with_resolution(resolution, resolution)
            .with_title("beast"),
    );

    window_manager.run(move |window_manager| {
        if window.should_close() {
            window_manager.end();
        }
        
        window.poll_events();

        for event in window.events().iter() {
            dbg!(event);
        }

        resolution += 3;
        window.resize(resolution, resolution);
        std::thread::sleep(Duration::from_millis(16));
        window.request_redraw();
    });
}
