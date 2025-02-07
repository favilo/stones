use avian3d::prelude::{
    AngularDamping, AngularVelocity, Collider, ColliderConstructor, ColliderConstructorHierarchy,
    CollisionLayers, CollisionMargin, GravityScale, LinearDamping, LinearVelocity, Mass,
    PhysicsPickable, Restitution, RigidBody, Sensor, SpeculativeMargin,
};
use bevy::{
    app,
    ecs::{entity::Entity, system::SystemId, world::World},
    prelude::*,
};
use bevy_mod_billboard::BillboardText;
use bevy_sequential_actions::{Action, ActionsProxy, ModifyActions, SequentialActions, StopReason};

use crate::{
    assets::GameAssets,
    game::{
        is_invalid_selection, Board, GameState, Hole, Player, PlayerTurn, Selected, Stone,
        BALL_RADIUS,
    },
    physics::GameLayer,
    rules::variants::{ChosenVariant, Index},
    PLAYER_COUNT,
};

use super::turn::PlayerMove;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let setup_board = app.register_system(setup_board);
        app.insert_resource(SetupBoard(setup_board));
        let setup_stones = app.register_system(setup_stones);
        app.insert_resource(SetupStones(setup_stones));
    }
}

#[derive(Clone, Copy, Debug, Resource, Deref)]
struct SetupBoard(SystemId);

#[derive(Clone, Copy, Debug, Resource, Deref)]
struct SetupStones(SystemId);

pub struct SpawnBoardAndPieces;

impl Action for SpawnBoardAndPieces {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, world: &mut World) -> bool {
        world
            .run_system(world.get_resource::<SetupBoard>().unwrap().0)
            .unwrap();
        world
            .run_system(world.get_resource::<SetupStones>().unwrap().0)
            .unwrap();

        true
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}

pub fn setup_board(mut board: ResMut<Board>, mut commands: Commands, game_assets: Res<GameAssets>) {
    *board = Board(ChosenVariant::default().to_variant());

    let collider = ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh);

    commands.spawn((
        RigidBody::Static,
        collider,
        CollisionMargin(0.005),
        CollisionLayers::new(GameLayer::PhysicsObject, GameLayer::PhysicsObject),
        Restitution::new(0.0),
        Name::from("Board"),
        SceneRoot::from(game_assets.board_scene.clone()),
        StateScoped(GameState::Playing),
    ));

    for player in 0..PLAYER_COUNT {
        for hole in 0..board.hole_count() {
            // Invisible material for hole
            let mut bucket_position = board
                .bucket_position(Index::Player(Player(player), Hole(hole)))
                + Vec3::new(0.0, 5.0, 0.0);
            bucket_position.y = 0.01;
            let color = match player {
                0 => Color::linear_rgba(0.0, 0.0, 1.0, 1.0),
                _ => Color::linear_rgba(0.0, 1.0, 0.0, 1.0),
            };
            commands
                .spawn((
                    Name::from(format!("bucket_{player}_{hole}")),
                    Player(player),
                    Hole(hole),
                    PointLight {
                        color,
                        intensity: 0.0,
                        range: 0.5,
                        radius: 0.5,
                        ..Default::default()
                    },
                    Transform::from_translation(bucket_position),
                    Collider::sphere(0.03),
                    CollisionLayers::new(GameLayer::MouseObject, GameLayer::MouseObject),
                    Sensor,
                    PhysicsPickable,
                    StateScoped(GameState::Playing),
                ))
                .observe(
                    move |over: Trigger<Pointer<Over>>,
                          mut lights: Query<&mut PointLight>,
                          turn: Res<PlayerTurn>,
                          board: Res<Board>,
                          game_state: Res<State<GameState>>| {
                        let entity = over.entity();
                        let mut light = lights.get_mut(entity).unwrap();
                        if is_invalid_selection(player, turn, game_state, board, hole) {
                            light.intensity = 0.0;
                            return;
                        }
                        light.intensity = 500.0;
                    },
                )
                .observe(
                    |out: Trigger<Pointer<Out>>, mut lights: Query<&mut PointLight>| {
                        let entity = out.entity();
                        let mut light = lights.get_mut(entity).unwrap();
                        light.intensity = 0.0;
                    },
                )
                .observe(
                    move |_down: Trigger<Pointer<Down>>,
                          mut selected: ResMut<Selected>,
                          turn: Res<PlayerTurn>,
                          board: Res<Board>,
                          game_state: Res<State<GameState>>| {
                        if is_invalid_selection(player, turn, game_state, board, hole) {
                            return;
                        }

                        selected.0 = Some(Index::Player(Player(player), Hole(hole)));
                    },
                )
                .observe(
                    move |_up: Trigger<Pointer<Up>>,
                          mut selected: ResMut<Selected>,
                          turn: Res<PlayerTurn>,
                          board: Res<Board>,
                          game_state: Res<State<GameState>>,
                          agent: Single<Entity, With<SequentialActions>>,
                          mut commands: Commands| {
                        if is_invalid_selection(player, turn, game_state, board, hole) {
                            return;
                        }

                        {
                            let Some(selected) = selected.0.as_mut() else {
                                return;
                            };
                            if *selected != Index::Player(Player(player), Hole(hole)) {
                                return;
                            }
                            commands
                                .actions(*agent)
                                .add(PlayerMove::with_input(*selected));
                        }
                        **selected = None;
                    },
                );

            let offset = match player {
                0 => Vec3::new(0.0, 0.0, -0.1),
                _ => Vec3::new(0.0, 0.0, 0.1),
            };
            let transform = Transform::from_translation(bucket_position + offset)
                .with_scale(Vec3::splat(0.001));
            commands.spawn((
                Name::from(format!("bucket_label_{player}_{hole}")),
                Player(player),
                Hole(hole),
                BillboardText::new("4"),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(30.0),
                TextColor(Color::WHITE),
                transform,
                StateScoped(GameState::Playing),
            ));
        }
    }
}

pub fn setup_stones(
    mut commands: Commands,
    mut board: ResMut<Board>,
    game_assets: Res<GameAssets>,
    meshes: Res<Assets<Mesh>>,
) {
    const SCALE: f32 = 0.8;

    let mut materials = game_assets.stone_materials.iter().cycle().cloned();

    let mesh = meshes.get(&game_assets.stone_mesh).unwrap();
    let collider = Collider::convex_hull_from_mesh(mesh).unwrap();

    tracing::info!("Spawning stones");
    for player in 0..PLAYER_COUNT {
        for hole in 0..board.hole_count() {
            for i in 0..board.starting_pieces() {
                let position = board.bucket_position(Index::Player(Player(player), Hole(hole)));
                let perturb = Vec3::new(
                    (i as f32 * 0.001).sin() * 0.0025,
                    i as f32 * BALL_RADIUS,
                    (i as f32 * 0.001).cos() * 0.0025,
                );

                board.push_entity(
                    Index::Player(Player(player), Hole(hole)),
                    commands
                        .spawn((
                            Name::from(format!("stone_{player}_{hole}_{i}")),
                            Stone,
                            RigidBody::Dynamic,
                            collider.clone(),
                            CollisionMargin(0.0025),
                            CollisionLayers::new(
                                GameLayer::PhysicsObject,
                                GameLayer::PhysicsObject,
                            ),
                            // Physics
                            (
                                GravityScale(0.25),
                                Mass(0.0001),
                                LinearVelocity(Vec3::ZERO),
                                AngularVelocity(Vec3::ZERO),
                                Restitution::new(0.00),
                                LinearDamping(0.9999),
                                AngularDamping(100.0),
                                Mesh3d(game_assets.stone_mesh.clone()),
                                MeshMaterial3d(materials.next().expect("cycles")),
                                Transform::from_translation(position + perturb)
                                    .with_rotation(Quat::from_rotation_x(90.0))
                                    .with_scale(Vec3::splat(SCALE)),
                                SpeculativeMargin(0.005),
                                // Maybe we'll turn this back on, but speculative is doing great.
                            ),
                            StateScoped(GameState::Playing),
                        ))
                        .id(),
                );
            }
        }
    }
}
