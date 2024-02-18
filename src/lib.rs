use std::{
    f32::consts::{FRAC_PI_2, PI},
    io::Write,
    path::PathBuf,
};

use assets::{AssetPlugin, ColliderWrapper};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    log::LogPlugin,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        config::ConfigureLoadingState, LoadingState, LoadingStateAppExt, LoadingStateSet,
    },
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use iyes_progress::{ProgressCounter, ProgressPlugin};
use tracing::Level;

mod assets;

#[derive(Default, Debug, Clone, Copy, States, Hash, PartialEq, Eq)]
enum GameState {
    #[default]
    Loading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "scenes/board.glb#Scene0")]
    board: Handle<Scene>,

    #[asset(path = "colliders/Board.msp")]
    board_collider: Handle<ColliderWrapper>,
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: Level::INFO,
                ..Default::default()
            }),
            FrameTimeDiagnosticsPlugin,
            // WorldInspectorPlugin::new(),
            ProgressPlugin::new(GameState::Loading).continue_to(GameState::Loaded),
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(AssetPlugin)
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Loaded)
                .load_collection::<GameAssets>(),
        )
        .add_systems(
            Update,
            (print_progress,)
                .run_if(in_state(GameState::Loading))
                .after(LoadingStateSet(GameState::Loading)),
        )
        .add_systems(OnEnter(GameState::Loaded), (setup_graphics, setup_physics))
        .add_systems(Update, save_collider_to_file);
    }
}

pub fn run() {
    App::new().add_plugins(GamePlugin).run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.35, 0.5)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    },));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 2.6,
            ..default()
        }
        .into(),
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            PI / 5.0,
            -FRAC_PI_2,
        )),
        ..default()
    });
}

const HOLE_DROP_POSITIONS: [Vec3; 12] = [
    // Top Row
    Vec3::new(-0.2, 0.255, -0.035),
    Vec3::new(-0.12, 0.255, -0.035),
    Vec3::new(-0.035, 0.255, -0.035),
    Vec3::new(0.035, 0.255, -0.035),
    Vec3::new(0.12, 0.255, -0.035),
    Vec3::new(0.2, 0.255, -0.035),
    // Bottom Row
    Vec3::new(-0.2, 0.255, 0.035),
    Vec3::new(-0.12, 0.255, 0.035),
    Vec3::new(-0.035, 0.255, 0.035),
    Vec3::new(0.035, 0.255, 0.035),
    Vec3::new(0.12, 0.255, 0.035),
    Vec3::new(0.2, 0.255, 0.035),
];

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    colliders: Res<Assets<ColliderWrapper>>,
    game_assets: Res<GameAssets>,
) {
    // tracing::info!("Opening collider file");
    // let collider_file = std::fs::File::open("assets/colliders/Board.msp").ok();
    // let collider: Option<Collider> = collider_file
    //     .map(|f| rmp_serde::from_read(f).ok())
    //     .flatten();

    let mut board = commands.spawn((
        RigidBody::Fixed,
        ColliderScale::Relative(Vect::new(0.25, 0.015, 0.075)),
        Friction {
            coefficient: 100.0,
            combine_rule: CoefficientCombineRule::Average,
        },
        Name::from("Board"),
        SceneBundle {
            scene: game_assets.board.clone(),
            ..default()
        },
    ));
    if let Some(collider) = colliders.get(&game_assets.board_collider) {
        tracing::info!("Inserting generated collider");
        board.insert(collider.0.clone());
    } else {
        tracing::info!("Inserting async collider");
        board.insert(AsyncSceneCollider {
            shape: Some(ComputedColliderShape::ConvexDecomposition(
                VHACDParameters {
                    resolution: 512,
                    concavity: 0.000001,
                    ..Default::default()
                },
            )),
            named_shapes: Default::default(),
        });
    }

    for position in HOLE_DROP_POSITIONS[..].iter().cloned() {
        for i in 0..4 {
            // let i = 0.0;
            // let perturb = Vec3::new(
            //     (i as f32 * 0.001).sin() * 0.00025,
            //     i as f32 * 0.1,
            //     (i as f32 * 0.001).cos() * 0.00025,
            // );
            let perturb = Vec3::new(0.0, i as f32 * 0.1, 0.0);
            commands.spawn((
                RigidBody::Dynamic,
                Collider::ball(0.005),
                // ColliderScale::Relative(Vect::new(1.0, 0.5, 1.0)),
                // GravityScale(0.5),
                GravityScale(2.5),
                Friction {
                    coefficient: 1.000,
                    combine_rule: CoefficientCombineRule::Average,
                },
                PbrBundle {
                    mesh: meshes.add(
                        shape::Icosphere {
                            radius: 0.005,
                            ..Default::default()
                        }
                        .try_into()
                        .unwrap(),
                    ),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_translation(position + perturb),
                    // .with_scale(Vec3::new(1.0, 0.5, 1.0))
                    ..default()
                },
                Ccd::enabled(),
            ));
        }
    }
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
