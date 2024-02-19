use bevy::{app, prelude::*};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_rapier3d::prelude::*;

use crate::{assets::ColliderWrapper, graphics::setup_graphics, GameAssets};

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Board::default())
            .add_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Loaded)
                    .load_collection::<GameAssets>(),
            )
            .add_systems(
                OnEnter(GameState::Loaded),
                (setup_graphics, setup_board, setup_pieces),
            );
    }
}

#[derive(Default, Debug, Clone, Copy, States, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Debug, Default)]
pub struct Side {
    buckets: [Vec<Entity>; 6],
    score: Vec<Entity>,
}

#[derive(Debug, Default, Resource)]
pub struct Board {
    players: [Side; 2],
}

impl Board {
    const HOLE_DROP_POSITIONS: [[Vec3; 6]; 2] = [
        // Top Row
        [
            Vec3::new(-0.2, 0.135, -0.035),
            Vec3::new(-0.12, 0.135, -0.035),
            Vec3::new(-0.035, 0.135, -0.035),
            Vec3::new(0.035, 0.135, -0.035),
            Vec3::new(0.12, 0.135, -0.035),
            Vec3::new(0.2, 0.135, -0.035),
        ],
        // Bottom Row
        [
            Vec3::new(-0.2, 0.135, 0.035),
            Vec3::new(-0.12, 0.135, 0.035),
            Vec3::new(-0.035, 0.135, 0.035),
            Vec3::new(0.035, 0.135, 0.035),
            Vec3::new(0.12, 0.135, 0.035),
            Vec3::new(0.2, 0.135, 0.035),
        ],
    ];

    pub const fn bucket_position(player: usize, hole: usize) -> Vec3 {
        assert!(player < 2, "Invalid player index");
        assert!(hole < 6, "Invalid hole index");
        Self::HOLE_DROP_POSITIONS[player][hole]
    }
}

pub fn setup_board(
    mut commands: Commands,
    colliders: Res<Assets<ColliderWrapper>>,
    game_assets: Res<GameAssets>,
) {
    let mut board_entity = commands.spawn((
        RigidBody::Fixed,
        ColliderScale::Relative(Vect::new(0.325, 0.015, 0.075)),
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

    // if let Some(collider) = colliders.get(&game_assets.board_collider) {
    if let Some(collider) = None::<&ColliderWrapper> {
        tracing::info!("Inserting loaded collider");
        board_entity.insert(collider.0.clone());
    } else {
        tracing::info!("Inserting async computed collider");
        board_entity.insert(AsyncSceneCollider {
            shape: Some(ComputedColliderShape::TriMesh),
            // shape: Some(ComputedColliderShape::ConvexDecomposition(
            //     VHACDParameters {
            //         resolution: 512 + 128,
            //         concavity: 0.000000001,
            //         max_convex_hulls: 2048,
            //         convex_hull_approximation: false,
            //         fill_mode: FillMode::SurfaceOnly,
            //         alpha: 0.05,
            //         beta: 0.05,
            //         ..Default::default()
            //     },
            // )),
            named_shapes: Default::default(),
        });
    }
}

pub fn setup_pieces(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut board: ResMut<Board>,
) {
    let mesh = meshes.add(
        shape::Icosphere {
            radius: 0.007,
            ..Default::default()
        }
        .try_into()
        .unwrap(),
    );
    let material_list = generate_materials(materials);
    let mut mat_iter = material_list.into_iter().cycle();
    let collider = Collider::ball(0.007);
    for player in 0..2 {
        for hole in 0..6 {
            for i in 0..4 {
                let position = Board::bucket_position(player, hole);
                let perturb = Vec3::new(
                    (i as f32 * 0.001).sin() * 0.0025,
                    i as f32 * 0.05,
                    (i as f32 * 0.001).cos() * 0.0025,
                );
                // let perturb = Vec3::new(0.0, i as f32 * 0.1, 0.0);

                board.players[player].buckets[hole].push(
                    commands
                        .spawn((
                            RigidBody::Dynamic,
                            collider.clone(),
                            GravityScale(2.5),
                            Damping {
                                linear_damping: 0.5,
                                angular_damping: 1.0,
                            },
                            Friction {
                                coefficient: 1.000,
                                combine_rule: CoefficientCombineRule::Average,
                            },
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: mat_iter.next().expect("cycles").clone(),
                                transform: Transform::from_translation(position + perturb),
                                ..default()
                            },
                            Ccd::enabled(),
                        ))
                        .id(),
                );
            }
        }
    }
}

fn generate_materials(
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
) -> Vec<Handle<StandardMaterial>> {
    let material_list = vec![
        materials.add(StandardMaterial {
            base_color: Color::rgba(0.1, 0.7, 0.6, 0.7),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.5,
            specular_transmission: 0.5,
            ior: 1.46,
            thickness: 0.5,
            metallic: 0.2,
            ..Default::default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::rgba(0.7, 0.1, 0.6, 0.7),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.5,
            specular_transmission: 0.5,
            ior: 1.46,
            thickness: 0.5,
            metallic: 0.2,
            ..Default::default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::rgba(0.6, 0.7, 0.1, 0.7),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.5,
            specular_transmission: 0.5,
            ior: 1.46,
            thickness: 0.5,
            metallic: 0.2,
            ..Default::default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::rgba(0.1, 0.6, 0.7, 0.7),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.5,
            specular_transmission: 0.5,
            ior: 1.46,
            thickness: 0.5,
            metallic: 0.2,
            ..Default::default()
        }),
    ];
    material_list
}
