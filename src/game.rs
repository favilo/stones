use std::hash::Hash;

use avian3d::prelude::*;
use bevy::{
    app,
    color::palettes::css::{DARK_CYAN, GOLD, GREEN, LIGHT_CYAN, SLATE_GRAY},
    ecs::{query::QueryData, system::SystemId},
    prelude::*,
    ui::FocusPolicy,
};
use bevy_asset_loader::{
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
use bevy_mod_billboard::BillboardText;

use crate::{
    assets::GameAssets,
    events::MoveEvent,
    physics::GameLayer,
    rules::variants::{ChosenVariant, Index, Variant},
    ui::{hover_button, unhover_button},
    PLAYER_COUNT,
};

pub struct Plugin;

#[derive(Resource, Debug, PartialEq, Eq, Clone, Deref)]
pub(crate) struct UpdateLabels(SystemId);

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChosenVariant::default())
            .insert_resource(Board(ChosenVariant::default().to_variant()))
            .register_type::<Player>()
            .register_type::<Hole>()
            .init_state::<GameState>()
            .add_event::<Winner>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Menu)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "dynamic.assets.ron",
                    )
                    .load_collection::<GameAssets>(),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                (setup_state, setup_board, setup_stones, setup_ui).chain(),
            )
            .add_systems(
                FixedUpdate,
                update_to_sleep.run_if(in_state(GameState::Playing)),
            )
            .add_observer(winner_found)
            .add_observer(perform_move);
        let update_label_system = app.register_system(update_labels);
        app.insert_resource(UpdateLabels(update_label_system));
    }
}

#[derive(Default, Debug, Clone, Copy, States, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Loading,

    Menu,
    Playing,
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct Player(pub usize);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct Hole(pub usize);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Stone;

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Score;

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Turn;

#[derive(Debug, Resource, Reflect, Deref, DerefMut)]
pub struct Board(Box<dyn Variant + 'static>);

#[derive(Debug, Default, Clone, Copy, Resource, Deref, DerefMut)]
pub struct Selected(Option<Index>);

#[derive(Debug, Default, Clone, Copy, Resource)]
pub struct PlayerTurn(pub usize);

#[derive(Debug, Clone, Copy, Default, Event)]
pub struct Winner(pub usize);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct WinnerText;

#[derive(Debug, Clone, Resource, Reflect, PartialEq, Eq, Deref, DerefMut)]
pub struct ToSleep(pub Timer);

impl Default for ToSleep {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

pub fn setup_state(mut commands: Commands) {
    commands.insert_resource(PlayerTurn(1));
    commands.insert_resource(Selected(None));
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
                        light.intensity = 1000.0;
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
                            commands.trigger(MoveEvent::HoleClicked(*selected));
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

fn is_invalid_selection(
    player: usize,
    turn: Res<'_, PlayerTurn>,
    game_state: Res<'_, State<GameState>>,
    board: Res<'_, Board>,
    hole: usize,
) -> bool {
    player != turn.0
        || *game_state != GameState::Playing
        || board
            .get_bucket_entities(Index::Player(Player(player), Hole(hole)))
            .is_empty()
}

pub fn winner_found(
    trigger: Trigger<Winner>,
    mut lights: Query<&mut PointLight>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    let Winner(winner) = *trigger;
    for mut light in lights.iter_mut() {
        light.intensity = 0.0;
    }
    spawn_win_text(winner, &mut commands, &game_assets);
}

pub fn perform_move(
    move_event: Trigger<MoveEvent>,
    mut board: ResMut<Board>,
    mut transforms: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
    mut turns: ResMut<PlayerTurn>,
    mut to_sleep: Option<ResMut<ToSleep>>,
    mut lights: Query<&mut PointLight>,
    update_labels: Res<UpdateLabels>,
    mut commands: Commands,
) {
    let event = *move_event;
    match event {
        MoveEvent::HoleClicked(index) => {
            board.perform_move(
                index,
                &mut transforms,
                &mut turns,
                &mut to_sleep,
                &mut commands,
            );
            // Make all the lights go out for now.
            lights.par_iter_mut().for_each(|mut light| {
                light.intensity = 0.0;
            });
        }
    }
    commands.run_system(**update_labels);
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct BucketData {
    player: &'static Player,
    hole: &'static Hole,
    text: &'static mut BillboardText,
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct TextData {
    player: &'static Player,
    text: &'static mut Text,
}

pub fn update_labels(
    board: Res<Board>,
    turn: Res<PlayerTurn>,
    mut buckets: Query<BucketData, (Without<Score>, Without<Turn>)>,
    mut score: Query<TextData, (With<Score>, Without<Turn>)>,
    mut turns: Query<TextData, (With<Turn>, Without<Score>)>,
) {
    buckets.par_iter_mut().for_each(
        |BucketDataItem {
             player,
             hole,
             mut text,
         }| {
            let count = board
                .get_bucket_entities(Index::Player(*player, *hole))
                .len();
            text.0 = count.to_string();
        },
    );

    score
        .par_iter_mut()
        .for_each(|TextDataItem { player, mut text }| {
            let count = board.get_bucket_entities(Index::Score(*player)).len();
            **text = count.to_string();
        });

    turns
        .par_iter_mut()
        .for_each(|TextDataItem { player, mut text }| {
            if player.0 == turn.0 {
                **text = "*".to_string();
            } else {
                **text = " ".to_string();
            }
        });
}

pub fn setup_stones(
    mut commands: Commands,
    mut board: ResMut<Board>,
    game_assets: Res<GameAssets>,
    meshes: Res<Assets<Mesh>>,
) {
    const BALL_RADIUS: f32 = 0.007;
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
    commands.insert_resource(ToSleep::default());
}

fn setup_ui(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(10.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::SpaceAround,
                ..Default::default()
            },
            StateScoped(GameState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Player(1),
                Score,
                Text::new("0"),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(40.0),
                TextColor(Color::Srgba(GREEN)),
                TextLayout::new_with_justify(JustifyText::Left),
                Node {
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
            ));
            parent.spawn((
                Player(1),
                Turn,
                Text::new("*"),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(40.0),
                TextColor(Color::Srgba(GREEN)),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
            ));
            parent.spawn((
                Text::new("Mancala: African Stones"),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(50.0),
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
            ));
            parent.spawn((
                Player(0),
                Turn,
                Text::new("*"),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(40.0),
                TextColor(Color::Srgba(DARK_CYAN)),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
            ));
            parent.spawn((
                Player(0),
                Score,
                Text::new("0"),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(40.0),
                TextColor(Color::Srgba(DARK_CYAN)),
                TextLayout::new_with_justify(JustifyText::Right),
                Node {
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
            ));
            parent
                .spawn((
                    MainMenuButton,
                    Button,
                    Node {
                        width: Val::Px(60.0),
                        ..Default::default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .observe(hover_button(Color::Srgba(GOLD)))
                .observe(unhover_button(Color::Srgba(SLATE_GRAY)))
                .observe(
                    |_click: Trigger<Pointer<Click>>, mut state: ResMut<NextState<GameState>>| {
                        state.set(GameState::Menu);
                    },
                )
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Main Menu"),
                        TextFont::from_font(game_assets.main_font.clone()).with_font_size(20.0),
                        TextColor(Color::Srgba(SLATE_GRAY)),
                    ));
                });
        });
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct WinnerUi;

#[derive(Debug, Default, Clone, Copy, Component)]
struct WinnerButton;

#[derive(Debug, Default, Clone, Copy, Component)]
struct MainMenuButton;

fn spawn_win_text(winner: usize, commands: &mut Commands, game_assets: &Res<GameAssets>) {
    assert!(winner < 2, "Invalid winner index");

    const WINNER_NAMES: [&str; 2] = ["Blue", "Green"];
    const COLORS: [Color; 2] = [Color::Srgba(LIGHT_CYAN), Color::Srgba(GREEN)];
    commands
        .spawn((
            WinnerButton,
            Button,
            FocusPolicy::Pass,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_self: AlignSelf::Center,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            BackgroundColor(Color::NONE),
            StateScoped(GameState::Playing),
        ))
        .observe(
            |_click: Trigger<Pointer<Click>>, mut state: ResMut<NextState<GameState>>| {
                state.set(GameState::Menu);
            },
        )
        .with_children(|parent| {
            parent.spawn((
                WinnerText,
                Text::new(format!("{} Player Wins!", WINNER_NAMES[winner])),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(50.0),
                TextColor(COLORS[winner]),
                TextLayout::new_with_justify(JustifyText::Center),
                FocusPolicy::Pass,
                Node {
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
            ));
        });
}

fn update_to_sleep(
    mut commands: Commands,
    delta_time: Res<Time>,
    to_sleep: Option<ResMut<ToSleep>>,
    mut timer: Query<Entity, With<Stone>>,
) {
    let Some(mut to_sleep) = to_sleep else {
        return;
    };
    to_sleep.0.tick(delta_time.delta());
    timer.iter_mut().for_each(move |entity| {
        if to_sleep.0.just_finished() {
            let mut stone = commands.entity(entity);
            stone.insert(Sleeping);
        }
    });
}
