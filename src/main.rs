use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_state::<GameState>()
        .add_systems(OnEnter(GameState::Loading), start_asset_loading)
        .add_systems(
            Update,
            (check_if_loaded).run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnEnter(GameState::Loaded), (setup_graphics, setup_physics))
        .run();
}

#[derive(Default, Debug, Clone, Copy, States, Hash, PartialEq, Eq)]
enum GameState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Default, Resource)]
struct GameAssets {
    board: Handle<Scene>,
    board_mesh: Handle<Mesh>,
}

fn start_asset_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let board = asset_server.load("scenes/board.glb#Scene0");
    let board_mesh = asset_server.load("scenes/board.glb#Mesh0/Primitive0");
    commands.insert_resource(GameAssets { board, board_mesh });
}

fn check_if_loaded(
    game_assets: Res<GameAssets>,
    scenes: Res<Assets<Scene>>,
    meshes: Res<Assets<Mesh>>,
    mut state: ResMut<NextState<GameState>>,
) {
    if scenes.get(&game_assets.board).is_none() || meshes.get(&game_assets.board_mesh).is_none() {
        return;
    }
    state.set(GameState::Loaded);
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
    Vec3::new(-0.2, 0.5, -0.035),
    Vec3::new(-0.12, 0.5, -0.035),
    Vec3::new(-0.035, 0.5, -0.035),
    Vec3::new(0.035, 0.5, -0.035),
    Vec3::new(0.12, 0.5, -0.035),
    Vec3::new(0.2, 0.5, -0.035),
    // Bottom Row
    Vec3::new(-0.2, 0.5, 0.035),
    Vec3::new(-0.12, 0.5, 0.035),
    Vec3::new(-0.035, 0.5, 0.035),
    Vec3::new(0.035, 0.5, 0.035),
    Vec3::new(0.12, 0.5, 0.035),
    Vec3::new(0.2, 0.5, 0.035),
];

fn setup_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    commands.spawn((
        RigidBody::Fixed,
        Collider::from_bevy_mesh(
            meshes
                .get(&game_assets.board_mesh)
                .expect("should be loaded"),
            &ComputedColliderShape::TriMesh,
            // &ComputedColliderShape::ConvexDecomposition(Default::default()),
        )
        .expect("mesh should work"),
        ColliderScale::Relative(Vect::new(0.25, 0.015, 0.075)),
        Friction {
            coefficient: 100.0,
            combine_rule: CoefficientCombineRule::Average,
        },
        SceneBundle {
            scene: asset_server.load("scenes/board.glb#Scene0"),
            ..default()
        },
    ));

    for position in HOLE_DROP_POSITIONS[2..5].iter().cloned() {
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
                    ..default()
                },
                Ccd::enabled(),
            ));
        }
    }
}
