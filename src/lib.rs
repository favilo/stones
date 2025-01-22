use avian3d::prelude::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    log::LogPlugin,
    prelude::*,
};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateSet};

use game::GameState;
use iyes_progress::{ProgressPlugin, ProgressTracker};
use tracing::Level;

mod assets;
mod events;
mod game;
mod graphics;
mod ui;

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(key = "board_scene")]
    board_scene: Handle<Scene>,

    // #[asset(key = "board_collider")]
    // board_collider: Handle<Scene>,

    // #[asset(key = "board_textures", collection(typed, mapped))]
    // #[asset(image(sampler(filter = nearest)))]
    // board_textures: HashMap<String, Handle<Image>>,
    //
    #[asset(key = "stone_mesh")]
    stone_mesh: Handle<Mesh>,

    #[asset(key = "stone_materials", collection(typed))]
    stone_materials: Vec<Handle<StandardMaterial>>,

    #[asset(key = "stone_collider")]
    stone_collider: Handle<Mesh>,
    //
    // #[asset(key = "stone_scene")]
    // stone_scenes: Handle<Scene>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin,
            ProgressPlugin::<GameState>::new()
                .with_state_transition(GameState::Loading, GameState::Menu),
            // ObjPlugin,
            // BillboardPlugin,
        ));
        #[cfg(not(target_os = "android"))]
        {
            app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
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
        .add_plugins((assets::Plugin, events::Plugin, game::Plugin, ui::Plugin))
        // .add_systems(OnEnter(GameState::Loading), setup_colliders)
        .add_systems(
            FixedUpdate,
            (print_progress,)
                .run_if(in_state(GameState::Loading))
                .after(LoadingStateSet(GameState::Loading)),
        )
        .add_systems(FixedUpdate, toggle_debug);
    }
}

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
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}

// fn save_collider_to_file(keys: Res<ButtonInput<KeyCode>>, colliders: Query<(&Name, &Collider)>) {
//     if !keys.just_pressed(KeyCode::KeyS) {
//         return;
//     }

//     for (name, collider) in colliders.iter() {
//         // Save the collider to a file
//         let path = PathBuf::from("assets")
//             .join("colliders")
//             .join(format!("{}.msp", name.as_str()));
//         tracing::info!("Saving collider to file: {}", path.display());
//         let mut file = std::fs::File::create(path).unwrap();
//         let packed = rmp_serde::to_vec(&collider).unwrap();
//         file.write_all(&packed).unwrap();
//     }
// }

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut config_store: ResMut<GizmoConfigStore>) {
    if !keys.just_pressed(KeyCode::KeyD) {
        return;
    }
    let (config, _) = config_store.config_mut::<PhysicsGizmos>();
    config.enabled = !config.enabled;
}

fn print_progress(
    progress: Option<Res<ProgressTracker<GameState>>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    if let Some(progress) = progress.map(|counter| counter.get_global_progress()) {
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

pub fn cleanup<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    entities.iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}
