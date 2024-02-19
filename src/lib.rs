use std::{io::Write, path::PathBuf};

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    log::LogPlugin,
    prelude::*,
};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateSet};
use bevy_rapier3d::prelude::*;
use game::GameState;
use iyes_progress::{ProgressCounter, ProgressPlugin};
use tracing::Level;

use crate::assets::ColliderWrapper;

mod assets;
mod game;
mod graphics;

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "scenes/low_poly.glb#Scene0")]
    board: Handle<Scene>,

    // #[asset(path = "colliders/Board.msp")]
    // board_collider: Handle<ColliderWrapper>,
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
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
            FrameTimeDiagnosticsPlugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            ProgressPlugin::new(GameState::Loading).continue_to(GameState::Loaded),
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(DebugRenderContext {
            enabled: false,
            ..Default::default()
        })
        .add_plugins((assets::Plugin, game::Plugin))
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
    App::new().add_plugins(GamePlugin).run();
}

fn save_collider_to_file(keys: Res<Input<KeyCode>>, colliders: Query<(&Name, &Collider)>) {
    if !keys.just_pressed(KeyCode::S) {
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

fn toggle_debug(keys: Res<Input<KeyCode>>, mut debug_context: ResMut<DebugRenderContext>) {
    if !keys.just_pressed(KeyCode::D) {
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
                    .get(FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                    .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                    .unwrap_or(0.),
                progress
            );
        }
    }
}
