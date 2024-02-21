use std::{io::Write, path::PathBuf};

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    log::LogPlugin,
    prelude::*,
    utils::HashMap,
};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateSet};
use bevy_mod_picking::{debug::DebugPickingPlugin, DefaultPickingPlugins};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use game::GameState;
use iyes_progress::{ProgressCounter, ProgressPlugin};
use tracing::Level;

mod assets;
mod events;
mod game;
mod graphics;

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(key = "board_mesh")]
    board_mesh: Handle<Mesh>,

    #[asset(key = "board_textures", collection(typed, mapped))]
    #[asset(image(sampler = nearest))]
    board_textures: HashMap<String, Handle<Image>>,

    #[asset(key = "piece_mesh")]
    piece_mesh: Handle<Mesh>,

    #[asset(key = "piece_collider")]
    piece_collider: Handle<Mesh>,

    #[asset(key = "green_textures", collection(typed, mapped))]
    #[asset(image(sampler = nearest))]
    green_textures: HashMap<String, Handle<Image>>,

    #[asset(key = "blue_textures", collection(typed, mapped))]
    #[asset(image(sampler = nearest))]
    blue_textures: HashMap<String, Handle<Image>>,

    #[asset(key = "red_textures", collection(typed, mapped))]
    #[asset(image(sampler = nearest))]
    red_textures: HashMap<String, Handle<Image>>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin,
            ProgressPlugin::new(GameState::Loading).continue_to(GameState::Loaded),
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
            ObjPlugin,
        ));
        #[cfg(not(target_os = "android"))]
        {
            app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
        }
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
            .insert_resource(DebugRenderContext {
                enabled: false,
                ..Default::default()
            })
            .add_plugins((assets::Plugin, events::Plugin, game::Plugin))
            .add_systems(
                Update,
                (print_progress,)
                    .run_if(in_state(GameState::Loading))
                    .after(LoadingStateSet(GameState::Loading)),
            )
            .add_systems(Update, (save_collider_to_file, toggle_debug));
    }
}

pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::INFO,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Mancala: African Stones".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}

fn save_collider_to_file(keys: Res<ButtonInput<KeyCode>>, colliders: Query<(&Name, &Collider)>) {
    if !keys.just_pressed(KeyCode::KeyS) {
        return;
    }

    for (name, collider) in colliders.iter() {
        // Save the collider to a file
        let path = PathBuf::from("assets")
            .join("colliders")
            .join(format!("{}.msp", name.as_str()));
        tracing::info!("Saving collider to file: {}", path.display());
        let mut file = std::fs::File::create(path).unwrap();
        let packed = rmp_serde::to_vec(&collider).unwrap();
        file.write_all(&packed).unwrap();
    }
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut debug_context: ResMut<DebugRenderContext>) {
    if !keys.just_pressed(KeyCode::KeyD) {
        return;
    }
    debug_context.enabled = !debug_context.enabled;
}

fn print_progress(
    progress: Option<Res<ProgressCounter>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    if let Some(progress) = progress.map(|counter| counter.progress()) {
        if progress.done > *last_done {
            *last_done = progress.done;
            info!(
                "[Frame {}] Changed progress: {:?}",
                diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                    .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                    .unwrap_or(0.),
                progress
            );
        }
    }
}
