//! The main game module.
//!
//! This module contains the main game entry point, as well as the game's main systems.

use avian3d::prelude::*;
use bevy::{
    asset::AssetMetaCheck, diagnostic::FrameTimeDiagnosticsPlugin, log::LogPlugin, prelude::*,
};

use bevy_mod_billboard::plugin::BillboardPlugin;
use game::GameState;
use iyes_progress::ProgressPlugin;
use tracing::Level;

mod assets;
mod events;
mod game;
mod graphics;
mod loading;
mod ui;

/// The Game Plugin that loads all the other bevy plugins.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.enable_state_scoped_entities::<GameState>()
            .add_plugins((
                FrameTimeDiagnosticsPlugin,
                ProgressPlugin::<GameState>::new()
                    .with_state_transition(GameState::Loading, GameState::Menu),
                // ObjPlugin,
                BillboardPlugin,
            ));
        #[cfg(not(target_os = "android"))]
        {
            app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new()
                .run_if(|config_store: Res<GizmoConfigStore>| -> bool{
                let (config, _) = config_store.config::<PhysicsGizmos>();
                config.enabled
            }))
            // .add_plugins(BlenvyPlugin::default())
            ;
        }
        app.add_plugins((
            PhysicsPlugins::default().with_length_unit(0.01),
            PhysicsDebugPlugin::default(),
            PhysicsPickingPlugin,
        ))
        .insert_resource(PhysicsPickingSettings {
            require_markers: true,
        })
        .insert_resource(SleepingThreshold {
            linear: 10000.0,
            angular: 10000.0,
        })
        .insert_resource(DeactivationTime(0.5))
        .insert_gizmo_config(
            PhysicsGizmos {
                // axis_lengths: Some(Vector::splat(0.02)),
                // aabb_color: Some(Color::WHITE),
                ..Default::default()
            },
            GizmoConfig {
                enabled: false,
                ..Default::default()
            },
        )
        .add_plugins((
            loading::Plugin,
            assets::Plugin,
            events::Plugin,
            game::Plugin,
            ui::Plugin,
        ))
        // .add_systems(OnEnter(GameState::Loading), setup_colliders)
        // .add_systems(
        //     Update,
        //     show_progress
        //         .run_if(in_state(GameState::Loading))
        //         .after(LoadingStateSet(GameState::Loading)),
        // )
        .add_systems(FixedUpdate, toggle_debug);
    }
}

/// The main entry point for the game.
pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: std::env::var("RUST_LOG").unwrap_or_default(),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Mancala: African Stones".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut config_store: ResMut<GizmoConfigStore>) {
    if !keys.just_pressed(KeyCode::KeyD) {
        return;
    }
    let (config, _) = config_store.config_mut::<PhysicsGizmos>();
    config.enabled = !config.enabled;
}
