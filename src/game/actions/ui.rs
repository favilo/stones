use bevy::color::palettes::css::{GREEN, LIGHT_CYAN};
use bevy::ecs::query::QueryData;
use bevy::ui::FocusPolicy;
use bevy::{app, ecs::system::SystemId, prelude::*};
use bevy_mod_billboard::BillboardText;

use crate::assets::GameAssets;
use crate::game::{
    Board, GameState, Hole, Player, PlayerTurn, Score, Turn, WinnerButton, WinnerText,
};
use crate::rules::variants::Index;

use super::{RunSystem, SystemInResource};

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let update_labels = app.register_system(update_labels);
        app.insert_resource(UpdateLabelsResource(update_labels));

        let winner_found = app.register_system(winner_found);
        app.insert_resource(WinnerFound(winner_found));
    }
}

#[derive(Clone, Copy, Debug, Resource, Deref)]
pub struct UpdateLabelsResource(SystemId);

impl SystemInResource for UpdateLabelsResource {
    type Input = ();

    fn system_id(&self) -> SystemId {
        self.0
    }
}

pub type UpdateLabels = RunSystem<UpdateLabelsResource>;

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

fn update_labels(
    board: Res<Board>,
    p_turn: Res<PlayerTurn>,
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
            if PlayerTurn::Player(player.0) == *p_turn {
                **text = "*".to_string();
            } else {
                **text = " ".to_string();
            }
        });
}

#[derive(Debug, Clone, Copy, Resource, Deref)]
pub struct WinnerFound(SystemId<In<Player>>);

impl SystemInResource for WinnerFound {
    type Input = In<Player>;

    fn system_id(&self) -> SystemId<Self::Input> {
        self.0
    }
}

pub type DeclareWinner = RunSystem<WinnerFound, Player, In<Player>>;

pub fn winner_found(
    In(winner): In<Player>,
    mut lights: Query<&mut PointLight>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    let Player(winner) = winner;
    for mut light in lights.iter_mut() {
        light.intensity = 0.0;
    }
    spawn_win_text(winner, &mut commands, &game_assets);
}

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
