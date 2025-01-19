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
            )
            .add_systems(
                OnEnter(GameState::Playing),
                (setup_board, setup_pieces, setup_ui),
            )
            .add_systems(
                Update,
                (
                    perform_move,
                    update_labels,
                    (winner_found, update_menu_button, update_winner),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
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

    pub fn perform_move(
        &mut self,
        player: usize,
        hole: usize,
        query: &mut Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
        turn: &mut ResMut<PlayerTurn>,
        winner_writer: &mut EventWriter<Winner>,
        commands: &mut Commands,
    ) {
        let entities = self.players[player].buckets[hole]
            .drain(..)
            .collect::<Vec<_>>();
        let start_player = player;
        let mut index = Index::Player(Player(start_player), Hole(hole));
        for ball in entities.into_iter() {
            index = index.next(Player(start_player));

            self.get_bucket_mut(index).push(ball);
            let (mut transform, mut linear_velocity, mut angular_velocity) =
                query.get_mut(ball).unwrap();
            transform.translation = Self::bucket_position(index);
            transform.rotation = Quat::default();
            **linear_velocity = Vector::ZERO;
            **angular_velocity = Vector::ZERO;
            let mut e = commands.entity(ball);
            e.insert(SweptCcd::default());
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

pub fn setup_board(
    mut board: ResMut<Board>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    tracing::trace!("Textures: {:#?}", game_assets.board_textures);
    *board = Board::default();
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(
            game_assets.board_textures
            ["scenes/mancala_board/textures/mancala_board_hi_standardSurface1_BaseColor.png"]
            .clone(),
        ),
        emissive_texture: Some(
            game_assets.board_textures["scenes/mancala_board/textures/mancala_board_hi_standardSurface1_Emissive.png"].clone(),
        ),
        metallic_roughness_texture: Some(
            game_assets.board_textures["scenes/mancala_board/textures/mancala_board_hi_standardSurface1_MetallicRoughness.png"]
            .clone(),
        ),
        // normal_map_texture: Some(
        //     game_assets.board_textures["scenes/mancala_board/textures/mancala_board_hi_standardSurface1_Normal.png"].clone(),
        // ),
        // depth_map: Some(
        //     game_assets.board_textures["scenes/mancala_board/textures/mancala_board_hi_standardSurface1_Height.png"].clone(),
        // ),
        ..Default::default()
    });
    let mut mesh = meshes.get(&game_assets.board_mesh).unwrap().clone();
    mesh.generate_tangents().unwrap();

    commands.spawn((
        GameUi,
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
        GravityScale(0.25),
        Mass(1000.0),
        Friction {
            static_coefficient: 100.0,
            dynamic_coefficient: 100.0,
            // combine_rule: CoefficientCombine::Max,
            ..Default::default()
        },
        Name::from("Board"),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
    ));

    let mesh = meshes.add(Sphere { radius: 0.03 }.mesh().ico(3).unwrap());

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
                    Mesh3d(mesh.clone()),
                    Collider::sphere(0.03),
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
                        if turn.0 != player {
                            return;
                        }
                        let entity = over.entity();
                        // let mut light = lights.get_mut(event.listener()).unwrap();
                        let mut light = lights.get_mut(entity).unwrap();
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

pub fn perform_move(
    mut move_events: EventReader<MoveEvent>,
    mut board: ResMut<Board>,
    mut transforms: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
    mut turns: ResMut<PlayerTurn>,
    mut winner: EventWriter<Winner>,
    mut commands: Commands,
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
                    &mut commands,
                );
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

pub fn setup_pieces(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut board: ResMut<Board>,
    game_assets: Res<GameAssets>,
) {
    const BALL_RADIUS: f32 = 0.005;
    const SCALE: f32 = 0.8;

    let mesh = game_assets.piece_mesh.clone();
    let material_list = generate_materials(materials, &game_assets);
    let mut mat_iter = material_list.into_iter().cycle();

    let collider_mesh = meshes.get(&game_assets.piece_collider).unwrap();
    let mut collider = Collider::convex_hull_from_mesh(collider_mesh)
        .expect("Failed to create convex hull collider for piece");
    collider.set_scale(Vector::splat(1.5), 2);

    tracing::info!("Spawning pieces");
    for player in 0..PLAYER_COUNT {
        for hole in 0..HOLE_COUNT {
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
                            GameUi,
                            RigidBody::Dynamic,
                            collider.clone(),
                            GravityScale(1.0),
                            // ColliderMassProperties::Mass(10.0),
                            Mass(10.0),
                            LinearVelocity(Vec3::ZERO),
                            AngularVelocity(Vec3::ZERO),
                            LinearDamping(50.0),
                            AngularDamping(0.5),
                            Friction {
                                static_coefficient: 100.0,
                                dynamic_coefficient: 100.0,
                                // combine_rule: CoefficientCombine::Max,
                                ..Default::default()
                            },
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(mat_iter.next().expect("cycles").clone()),
                            Transform::from_translation(position + perturb)
                                .with_scale(Vec3::splat(SCALE)),
                            SweptCcd::default(),
                        ))
                        .id(),
                );
            }
        }
    }
}

fn generate_materials(
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    game_assets: &Res<GameAssets>,
) -> Vec<Handle<StandardMaterial>> {
    let material_list = vec![
        materials.add(StandardMaterial {
            base_color_texture: Some(
                game_assets
                    .green_textures[
                        "scenes/mancala_stone/textures/green/mancala_stone_hi_standardSurface1_BaseColor.png"
                    ].clone()
            ),
            emissive_texture: Some(
                game_assets
                    .green_textures[
                        "scenes/mancala_stone/textures/green/mancala_stone_hi_standardSurface1_Emissive.png"
                    ].clone()
            ),
            metallic_roughness_texture: Some(
                game_assets
                    .green_textures[
                        "scenes/mancala_stone/textures/green/mancala_stone_hi_standardSurface1_MetallicRoughness.png"
                    ].clone()
            ),
            normal_map_texture: Some(
                game_assets
                    .green_textures[
                        "scenes/mancala_stone/textures/green/mancala_stone_hi_standardSurface1_Normal.png"
                    ].clone()
            ),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        }),
        materials.add(StandardMaterial {
            base_color_texture: Some(
                game_assets
                    .blue_textures[
                        "scenes/mancala_stone/textures/blue/mancala_stone_hi_standardSurface1_BaseColor.png"
                    ].clone()
            ),
            emissive_texture: Some(
                game_assets
                    .blue_textures[
                        "scenes/mancala_stone/textures/blue/mancala_stone_hi_standardSurface1_Emissive.png"
                    ].clone()
            ),
            metallic_roughness_texture: Some(
                game_assets
                    .blue_textures[
                        "scenes/mancala_stone/textures/blue/mancala_stone_hi_standardSurface1_MetallicRoughness.png"
                    ].clone()
            ),
            normal_map_texture: Some(
                game_assets
                    .green_textures[
                        "scenes/mancala_stone/textures/green/mancala_stone_hi_standardSurface1_Normal.png"
                    ].clone()
            ),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        }),
        materials.add(StandardMaterial {
            base_color_texture: Some(
                game_assets
                    .red_textures[
                        "scenes/mancala_stone/textures/red/mancala_stone_hi_standardSurface1_BaseColor.png"
                    ].clone()
            ),
            emissive_texture: Some(
                game_assets
                    .red_textures[
                        "scenes/mancala_stone/textures/red/mancala_stone_hi_standardSurface1_Emissive.png"
                    ].clone()
            ),
            metallic_roughness_texture: Some(
                game_assets
                    .red_textures[
                        "scenes/mancala_stone/textures/red/mancala_stone_hi_standardSurface1_MetallicRoughness.png"
                    ].clone()
            ),
            normal_map_texture: Some(
                game_assets
                    .green_textures[
                        "scenes/mancala_stone/textures/green/mancala_stone_hi_standardSurface1_Normal.png"
                    ].clone()
            ),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        }),
    ];
    material_list
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
