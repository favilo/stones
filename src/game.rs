use avian3d::{math::Vector, prelude::*};
use bevy::{
    app,
    color::palettes::css::{GREEN, LIGHT_CYAN, SLATE_GRAY},
    ecs::query::QueryData,
    prelude::*,
    ui::FocusPolicy,
};
use bevy_asset_loader::{
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};

use crate::{cleanup, events::MoveEvent, GameAssets};

pub const PLAYER_COUNT: usize = 2;
pub const HOLE_COUNT: usize = 6;
pub const STARTING_PIECES: usize = 4;

pub struct Plugin;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum GameLayer {
    #[default]
    Default,
    PhysicsObject,
    MouseObject,
}

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::default())
            .insert_resource(PlayerTurn(1))
            .insert_resource(Selected(None))
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
                // .init_resource::<BoardCollider>(),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                (setup_board, setup_stones, setup_ui).chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    perform_move,
                    update_labels,
                    (winner_found, update_menu_button, update_winner),
                )
                    // .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, update_to_sleep)
            .add_systems(
                OnExit(GameState::Playing),
                (cleanup::<GameUi>, cleanup::<WinnerUi>),
            );
    }
}

#[derive(Debug, Default, Component, Copy, Clone)]
pub struct GameUi;

#[derive(Default, Debug, Clone, Copy, States, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Loading,

    Menu,
    Playing,
}

#[derive(Debug, Default)]
pub struct Side {
    buckets: [Vec<Entity>; HOLE_COUNT],

    #[allow(unused)]
    score: Vec<Entity>,
}

#[derive(Debug, Default, Resource)]
pub struct Board {
    players: [Side; PLAYER_COUNT],
}

impl Board {
    const HOLE_DROP_POSITIONS: [[Vec3; HOLE_COUNT]; PLAYER_COUNT] = [
        // Top Row
        [
            Vec3::new(-0.215, 0.075, -0.035),
            Vec3::new(-0.130, 0.075, -0.035),
            Vec3::new(-0.040, 0.075, -0.035),
            Vec3::new(00.042, 0.075, -0.035),
            Vec3::new(00.130, 0.075, -0.035),
            Vec3::new(00.215, 0.075, -0.035),
        ],
        // Bottom Row
        [
            Vec3::new(00.215, 0.075, 0.035),
            Vec3::new(00.130, 0.075, 0.035),
            Vec3::new(00.042, 0.075, 0.035),
            Vec3::new(-0.040, 0.075, 0.035),
            Vec3::new(-0.130, 0.075, 0.035),
            Vec3::new(-0.215, 0.075, 0.035),
        ],
    ];
    const BUCKET_POSITIONS: [Vec3; PLAYER_COUNT] =
        [Vec3::new(0.276, 0.075, 0.0), Vec3::new(-0.276, 0.075, 0.0)];

    pub fn bucket_position(index: Index) -> Vec3 {
        match index {
            Index::Player(Player(p), Hole(h)) => {
                assert!(p < PLAYER_COUNT, "Invalid player index");
                assert!(h < HOLE_COUNT, "Invalid hole index");
                Self::HOLE_DROP_POSITIONS[p][h]
            }
            Index::Score(Player(p)) => {
                assert!(p < PLAYER_COUNT, "Invalid player index");
                Self::BUCKET_POSITIONS[p]
            }
        }
    }

    pub fn get_bucket(&self, index: Index) -> &[Entity] {
        match index {
            Index::Player(Player(p), Hole(h)) => &self.players[p].buckets[h],
            Index::Score(Player(p)) => &self.players[p].score,
        }
    }

    pub fn get_bucket_mut(&mut self, index: Index) -> &mut Vec<Entity> {
        match index {
            Index::Player(Player(p), Hole(h)) => &mut self.players[p].buckets[h],
            Index::Score(Player(p)) => &mut self.players[p].score,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn perform_move(
        &mut self,
        player: usize,
        hole: usize,
        query: &mut Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
        turn: &mut ResMut<PlayerTurn>,
        winner_writer: &mut EventWriter<Winner>,
        to_sleep: &mut Option<ResMut<ToSleep>>,
        commands: &mut Commands,
    ) {
        let entities = std::mem::take(&mut self.players[player].buckets[hole]);
        let start_player = player;
        let mut index = Index::Player(Player(start_player), Hole(hole));
        for stone in entities.into_iter() {
            index = index.next(Player(start_player));

            self.get_bucket_mut(index).push(stone);
            let (mut transform, mut linear_velocity, mut angular_velocity) =
                query.get_mut(stone).unwrap();
            transform.translation = Self::bucket_position(index);
            transform.rotation = Quat::from_rotation_x(90.0);
            **linear_velocity = Vector::ZERO;
            **angular_velocity = Vector::ZERO;
            let mut e = commands.entity(stone);
            e.remove::<Sleeping>();
        }

        if !matches!(index, Index::Score(_)) {
            turn.0 = (turn.0 + 1) % 2;
        }

        if self
            .players
            .iter()
            .any(|side| side.buckets.iter().all(|bucket| bucket.is_empty()))
        {
            let winner = self
                .players
                .iter()
                .enumerate()
                .max_by_key(|(_, side)| side.score.len())
                .unwrap()
                .0;
            winner_writer.send(Winner(winner));
        } else {
            to_sleep.as_mut().map(|t| t.reset());
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

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Stone;

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Score;

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct Turn;

#[derive(Debug, Default, Clone, Copy, Resource)]
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

pub fn setup_board(
    mut board: ResMut<Board>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    // board_collider: Res<BoardCollider>,
) {
    *board = Board::default();

    // let board_mesh = meshes.get(&game_assets.board_collider).unwrap();
    // let collider = board_collider.0.clone();
    let collider = ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh);

    commands.spawn((
        GameUi,
        RigidBody::Static,
        collider,
        CollisionMargin(0.005),
        CollisionLayers::new(GameLayer::PhysicsObject, GameLayer::PhysicsObject),
        Friction::new(1.0),
        Name::from("Board"),
        SceneRoot::from(game_assets.board_scene.clone()),
    ));

    // let mesh = meshes.add(Sphere { radius: 0.03 }.mesh().ico(3).unwrap());

    for player in 0..PLAYER_COUNT {
        for hole in 0..HOLE_COUNT {
            // Invisible material for hole
            let mut bucket_position =
                Board::bucket_position(Index::Player(Player(player), Hole(hole)));
            bucket_position.y = 0.01;
            let color = match player {
                0 => Color::linear_rgba(0.0, 0.0, 1.0, 1.0),
                _ => Color::linear_rgba(0.0, 1.0, 0.0, 1.0),
            };
            commands
                .spawn((
                    Name::from(format!("bucket_{player}_{hole}")),
                    GameUi,
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
                    // Mesh3d(mesh.clone()),
                    Collider::sphere(0.03),
                    CollisionLayers::new(GameLayer::MouseObject, GameLayer::MouseObject),
                    Sensor,
                    PhysicsPickable,
                ))
                .observe(
                    move |over: Trigger<Pointer<Over>>,
                          mut lights: Query<&mut PointLight>,
                          turn: Res<PlayerTurn>,
                          game_state: Res<State<GameState>>| {
                        if *game_state != GameState::Playing {
                            return;
                        }
                        let entity = over.entity();
                        let mut light = lights.get_mut(entity).unwrap();
                        if turn.0 != player {
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
                          mut move_events: EventWriter<MoveEvent>,
                          game_state: Res<State<GameState>>| {
                        if is_invalid_selection(player, turn, game_state, board, hole) {
                            return;
                        }

                        if selected.0.is_none() {
                            return;
                        }
                        if selected.0.unwrap() != Index::Player(Player(player), Hole(hole)) {
                            return;
                        }
                        move_events.send(MoveEvent::HoleClicked(player, hole));
                        selected.0 = None;
                    },
                );

            let offset = match player {
                0 => Vec3::new(0.0, 0.0, -0.1),
                _ => Vec3::new(0.0, 0.0, 0.1),
            };
            let transform =
                Transform::from_translation(bucket_position + offset + Vec3::new(0.0, 0.05, 0.0))
                    .with_scale(Vec3::splat(0.001));
            commands.spawn((
                Name::from(format!("bucket_label_{player}_{hole}")),
                GameUi,
                Player(player),
                Hole(hole),
                // Hopefully this will allow billboarding text
                Text2d::new("4"),
                TextFont::from_font_size(50.0),
                TextColor(Color::linear_rgba(1.0, 1.0, 1.0, 1.0)),
                TextLayout::new_with_justify(JustifyText::Center),
                transform,
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
            .get_bucket(Index::Player(Player(player), Hole(hole)))
            .is_empty()
}

pub fn winner_found(
    mut winner: EventReader<Winner>,
    mut lights: Query<&mut PointLight>,
    mut commands: Commands,
) {
    if let Some(Winner(winner)) = winner.read().next() {
        for mut light in lights.iter_mut() {
            light.intensity = 0.0;
        }
        spawn_win_text(*winner, &mut commands);
    }
}

type InteractionsData<'world> = (&'world Interaction, &'world Children);

fn update_menu_button(
    mut state: ResMut<NextState<GameState>>,
    interactions: Query<InteractionsData, (With<MainMenuButton>, Changed<Interaction>)>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, children) in interactions.iter() {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match interaction {
            Interaction::Pressed => state.set(GameState::Menu),
            Interaction::Hovered => {
                **text_color = Color::WHITE;
            }
            Interaction::None => {
                **text_color = Color::Srgba(SLATE_GRAY);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn perform_move(
    mut move_events: EventReader<MoveEvent>,
    mut board: ResMut<Board>,
    mut transforms: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
    mut turns: ResMut<PlayerTurn>,
    mut winner: EventWriter<Winner>,
    mut to_sleep: Option<ResMut<ToSleep>>,
    mut commands: Commands,
    mut lights: Query<&mut PointLight>,
) {
    for event in move_events.read() {
        match *event {
            MoveEvent::HoleClicked(player, hole) => {
                board.perform_move(
                    player,
                    hole,
                    &mut transforms,
                    &mut turns,
                    &mut winner,
                    &mut to_sleep,
                    &mut commands,
                );
                // Make all the lights go out for now.
                lights.par_iter_mut().for_each(|mut light| {
                    light.intensity = 0.0;
                });
            }
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct BucketData {
    player: &'static Player,
    hole: &'static Hole,
    text: &'static mut Text2d,
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct TextData {
    player: &'static Player,
    text: &'static mut Text2d,
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
            let count = board.get_bucket(Index::Player(*player, *hole)).len();
            **text = count.to_string();
        },
    );

    score
        .par_iter_mut()
        .for_each(|TextDataItem { player, mut text }| {
            let count = board.get_bucket(Index::Score(*player)).len();
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

    let mesh = meshes.get(&game_assets.stone_collider).unwrap();
    let collider = Collider::convex_hull_from_mesh(mesh).unwrap();

    tracing::info!("Spawning stones");
    for player in 0..PLAYER_COUNT {
        for hole in 0..HOLE_COUNT {
            // let hole = 0;
            for i in 0..STARTING_PIECES {
                let position = Board::bucket_position(Index::Player(Player(player), Hole(hole)));
                let perturb = Vec3::new(
                    (i as f32 * 0.001).sin() * 0.0025,
                    i as f32 * BALL_RADIUS,
                    (i as f32 * 0.001).cos() * 0.0025,
                );
                // let perturb = Vec3::new(0.0, i as f32 * 0.1, 0.0);

                board.players[player].buckets[hole].push(
                    commands
                        .spawn((
                            Name::from(format!("stone_{player}_{hole}_{i}")),
                            Stone,
                            GameUi,
                            RigidBody::Dynamic,
                            collider.clone(),
                            CollisionMargin(0.0025),
                            CollisionLayers::new(
                                GameLayer::PhysicsObject,
                                GameLayer::PhysicsObject,
                            ),
                            // ToSleep::default(),
                            // Physics
                            (
                                GravityScale(0.25),
                                Mass(0.0001),
                                LinearVelocity(Vec3::ZERO),
                                AngularVelocity(Vec3::ZERO),
                                Restitution::new(0.01),
                                LinearDamping(0.999),
                                AngularDamping(100.0),
                                Friction::new(0.9),
                                Mesh3d(game_assets.stone_mesh.clone()),
                                MeshMaterial3d(materials.next().expect("cycles")),
                                Transform::from_translation(position + perturb)
                                    .with_rotation(Quat::from_rotation_x(90.0))
                                    .with_scale(Vec3::splat(SCALE)),
                                SpeculativeMargin(0.005),
                                // Maybe we'll turn this back on, but speculative is doing great.
                                // SweptCcd::default(),
                            ),
                        ))
                        .id(),
                );
            }
        }
    }
    commands.insert_resource(ToSleep::default());
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            GameUi,
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(10.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::SpaceAround,
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Player(1),
                Score,
                Text::new("0"),
                TextFont::from_font_size(40.0),
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
                TextFont::from_font_size(40.0),
                TextColor(Color::Srgba(GREEN)),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
            ));
            parent.spawn((
                Text::new("Mancala: African Stones"),
                TextFont::from_font_size(50.0),
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
                TextFont::from_font_size(40.0),
                TextColor(Color::Srgba(LIGHT_CYAN)),
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
                TextFont::from_font_size(40.0),
                TextColor(Color::Srgba(LIGHT_CYAN)),
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
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Main Menu"),
                        TextFont::from_font_size(20.0),
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

fn spawn_win_text(winner: usize, commands: &mut Commands) {
    assert!(winner < 2, "Invalid winner index");

    const WIN_COLOR: [&str; 2] = ["Blue", "Green"];
    const COLORS: [Color; 2] = [Color::Srgba(LIGHT_CYAN), Color::Srgba(GREEN)];
    commands
        .spawn((
            GameUi,
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
        ))
        .with_children(|parent| {
            parent.spawn((
                WinnerText,
                Text::new(format!("Player {} Wins!", WIN_COLOR[winner])),
                TextFont::from_font_size(50.0),
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

fn update_winner(
    mut state: ResMut<NextState<GameState>>,
    interactions: Query<&Interaction, (With<WinnerButton>, Changed<Interaction>)>,
) {
    for interaction in interactions.iter() {
        if interaction == &Interaction::Pressed {
            state.set(GameState::Menu)
        }
    }
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
