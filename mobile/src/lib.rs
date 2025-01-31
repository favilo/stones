//! This is defines the `main` function for the mobile builds of `stones`
//!
//! Uses `bevy_main` to set the correct stuff
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use stones::GamePlugin;

#[bevy_main]
fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LIB_BACKTRACE", "1");
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::INFO,
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Mancala: African Stones".to_string(),
                    resizable: false,
                    mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    ..Default::default()
                }),
                ..Default::default()
            }),
    )
    .add_plugins(GamePlugin);
    // #[cfg(target_os = "android")]
    // app.insert_resource(Msaa::Off);
    app.run();
}
