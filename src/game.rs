use bevy::{app, prelude::*};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_mod_picking::{
    events::{Click, Out, Over, Pointer},
    prelude::{Listener, On},
    PickableBundle,
};
use bevy_rapier3d::prelude::*;
use tracing::instrument;

use crate::{events::MoveEvent, graphics::setup_graphics, GameAssets};

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Board::default())
            .register_type::<Player>()
            .register_type::<Hole>()
            .add_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Loaded)
                    .load_collection::<GameAssets>(),
            )
            .add_systems(
                OnEnter(GameState::Loaded),
                (setup_graphics, setup_board, setup_pieces),
            )
            .add_systems(Update, perform_move.run_if(in_state(GameState::Loaded)));
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

    #[allow(unused)]
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
            Vec3::new(-0.215, 0.135, -0.035),
            Vec3::new(-0.130, 0.135, -0.035),
            Vec3::new(-0.040, 0.135, -0.035),
            Vec3::new(00.042, 0.135, -0.035),
            Vec3::new(00.130, 0.135, -0.035),
            Vec3::new(00.215, 0.135, -0.035),
        ],
        // Bottom Row
        [
            Vec3::new(00.215, 0.135, 0.035),
            Vec3::new(00.130, 0.135, 0.035),
            Vec3::new(00.042, 0.135, 0.035),
            Vec3::new(-0.040, 0.135, 0.035),
            Vec3::new(-0.130, 0.135, 0.035),
            Vec3::new(-0.215, 0.135, 0.035),
        ],
    ];
    const BUCKET_POSITIONS: [Vec3; 2] =
        [Vec3::new(0.275, 0.135, 0.0), Vec3::new(-0.275, 0.135, 0.0)];

    pub fn bucket_position(index: Index) -> Vec3 {
        match index {
            Index::Player(Player(p), Hole(h)) => {
                assert!(p < 2, "Invalid player index");
                assert!(h < 6, "Invalid hole index");
                Self::HOLE_DROP_POSITIONS[p][h]
            }
            Index::Score(Player(p)) => {
                assert!(p < 2, "Invalid player index");
                Self::BUCKET_POSITIONS[p]
            }
        }
    }

    pub fn get_bucket_mut(&mut self, index: Index) -> &mut Vec<Entity> {
        match index {
            Index::Player(Player(p), Hole(h)) => {
                tracing::info!("Getting hole for player {:?} hole {:?}", p, h);
                &mut self.players[p].buckets[h]
            }
            Index::Score(Player(p)) => {
                tracing::info!("Getting score bucket for player {:?}", p);
                &mut self.players[p].score
            }
        }
    }

    pub fn perform_move(
        &mut self,
        player: usize,
        hole: usize,
        query: &mut Query<(&mut Transform, &mut Sleeping, &mut Ccd)>,
    ) {
        let entities = self.players[player].buckets[hole]
            .drain(..)
            .collect::<Vec<_>>();
        let start_player = player;
        let mut index = Index::Player(Player(start_player), Hole(hole));
        for ball in entities.into_iter() {
            index = index.next(Player(start_player));

            tracing::info!("Moving ball to {:?}", index);
            self.get_bucket_mut(index).push(ball);
            let (mut transform, mut sleeping, mut ccd) = query.get_mut(ball).unwrap();
            transform.translation = Self::bucket_position(index);
            sleeping.sleeping = false;
            ccd.enabled = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Index {
    Player(Player, Hole),
    Score(Player),
}

impl Index {
    fn next(self, Player(start): Player) -> Self {
        match self {
            Index::Player(Player(p), Hole(h)) => {
                if h >= 5 {
                    if p == start {
                        Index::Score(Player(p))
                    } else {
                        Index::Player(Player((p + 1) % 2), Hole(0))
                    }
                } else {
                    Index::Player(Player(p), Hole(h + 1))
                }
            }
            Index::Score(Player(p)) => Index::Player(Player((p + 1) % 2), Hole(0)),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Player(pub usize);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Hole(pub usize);

pub fn setup_board(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
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

    tracing::info!("Inserting async computed collider");
    board_entity.insert(AsyncSceneCollider {
        shape: Some(ComputedColliderShape::TriMesh),
        named_shapes: Default::default(),
    });
    let mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.03,
            subdivisions: 3,
        })
        .unwrap(),
    );

    for player in 0..2 {
        for hole in 0..6 {
            // Invisible material for hole
            let mut bucket_position =
                Board::bucket_position(Index::Player(Player(player), Hole(hole)));
            bucket_position.y = 0.01;
            commands.spawn((
                Player(player),
                Hole(hole),
                PointLightBundle {
                    point_light: PointLight {
                        intensity: 0.0,
                        range: 0.5,
                        radius: 0.5,
                        color: Color::rgba(1.0, 1.0, 0.0, 1.0),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(bucket_position),
                    ..Default::default()
                },
                mesh.clone(),
                Collider::ball(0.03),
                Sensor,
                PickableBundle::default(),
                On::<Pointer<Over>>::target_component_mut(|_, light: &mut PointLight| {
                    light.intensity = 1.0;
                }),
                On::<Pointer<Out>>::target_component_mut(|_, light: &mut PointLight| {
                    light.intensity = 0.0;
                }),
                On::<Pointer<Click>>::run(
                    move |_event: Listener<Pointer<Click>>,
                          mut move_events: EventWriter<MoveEvent>| {
                        move_events.send(MoveEvent::HoleClicked(player, hole));
                    },
                ),
            ));
        }
    }
}

pub fn perform_move(
    mut move_events: EventReader<MoveEvent>,
    mut board: ResMut<Board>,
    mut transforms: Query<(&mut Transform, &mut Sleeping, &mut Ccd)>,
) {
    for event in move_events.read() {
        match *event {
            MoveEvent::HoleClicked(player, hole) => {
                tracing::info!("Player {:?} moved hole {:?}", player, hole);
                board.perform_move(player, hole, &mut transforms);
            }
        }
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
                let position = Board::bucket_position(Index::Player(Player(player), Hole(hole)));
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
                            Sleeping {
                                sleeping: false,
                                ..Default::default()
                            },
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
