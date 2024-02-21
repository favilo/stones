use bevy::{prelude::*, log::{Level, LogPlugin}};
use stones::GamePlugin;

// TODO: Replace this with bevy_main, but we need to test things first
#[no_mangle]
fn android_main(android_app: bevy::winit::android_activity::AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LIB_BACKTRACE", "1");
    let _ = bevy::winit::ANDROID_APP.set(android_app);
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
                    mode: bevy::window::WindowMode::BorderlessFullscreen,
                    ..Default::default()
                }),
                ..Default::default()
            }),
    )
    .add_plugins(GamePlugin);
    #[cfg(target_os = "android")]
    app.insert_resource(Msaa::Off);
    app.run();
}

#[cfg_attr(target_os = "android", link(name = "c++_shared"))]
extern "C" {}
