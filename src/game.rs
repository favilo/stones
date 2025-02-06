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
use bevy_sequential_actions::{ActionsProxy, ModifyActions, SequentialActions};

use crate::{
    assets::GameAssets,
    events::MoveEvent,
    physics::GameLayer,
    rules::variants::{ChosenVariant, Index, Variant},
    ui::{hover_button, unhover_button},
    PLAYER_COUNT,
};

use self::actions::{board::SpawnBoardAndPieces, turn::NextPlayer};

pub mod actions;

pub const BALL_RADIUS: f32 = 0.007;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(actions::Plugin)
            .insert_resource(ChosenVariant::default())
            .insert_resource(Board(ChosenVariant::default().to_variant()))
            .insert_resource(PlayerTurn::None)
            .insert_resource(Selected(None))
            .register_type::<Player>()
            .register_type::<Hole>()
            .init_state::<GameState>()
            .enable_state_scoped_entities::<GameState>()
            // .add_event::<Winner>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Menu)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "dynamic.assets.ron",
                    )
                    .load_collection::<GameAssets>(),
            )
            .add_systems(OnEnter(GameState::Playing), (setup_state, setup_ui))
        // .add_observer(winner_found);
        ;
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

impl Player {
    pub fn next(this: impl Into<Self>) -> Self {
        Self((this.into().0 + 1) % 2)
    }
}

impl From<usize> for Player {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

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

#[derive(Debug, Default, Clone, Copy, Resource, PartialEq, Eq)]
pub enum PlayerTurn {
    #[default]
    None,
    Player(usize),
}

impl From<PlayerTurn> for Option<usize> {
    fn from(value: PlayerTurn) -> Self {
        match value {
            PlayerTurn::Player(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, PartialEq, Eq, Hash)]
pub struct WinnerText;

pub fn setup_state(mut commands: Commands, agent_q: Single<Entity, With<SequentialActions>>) {
    commands
        .actions(*agent_q)
        .add((SpawnBoardAndPieces, NextPlayer(Player(1))));
}

fn is_invalid_selection(
    player: usize,
    p_turn: Res<'_, PlayerTurn>,
    game_state: Res<'_, State<GameState>>,
    board: Res<'_, Board>,
    hole: usize,
) -> bool {
    PlayerTurn::Player(player) != *p_turn
        || *game_state != GameState::Playing
        || board
            .get_bucket_entities(Index::Player(Player(player), Hole(hole)))
            .is_empty()
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
