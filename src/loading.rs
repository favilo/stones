use bevy::{app, prelude::*};
use iyes_progress::ProgressTracker;

use crate::{game::GameState, graphics::setup_graphics};

pub struct Plugin;
impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading),
            (setup_graphics, setup_loading_ui).chain(),
        )
        .add_systems(Update, update_progress.run_if(in_state(GameState::Loading)));
    }
}

#[derive(Default, Reflect, Component)]
struct Loading;

#[derive(Default, Reflect, Component)]
struct ProgressBarInner;

#[derive(Default, Reflect, Component)]
struct ProgressBarText;

fn setup_loading_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Loading...".to_owned()),
        TextFont::from_font_size(30.0),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(40.0),
            align_items: AlignItems::Center,
            ..Default::default()
        },
        StateScoped(GameState::Loading),
    ));
    commands
        .spawn((
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            BorderColor(Color::srgb(1.0, 1.0, 1.0)),
            BorderRadius::all(Val::Px(6.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(45.0),
                bottom: Val::Percent(45.0),
                left: Val::Percent(10.0),
                right: Val::Percent(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            StateScoped(GameState::Loading),
        ))
        .with_child((
            BackgroundColor(Color::srgb(0.75, 0.75, 0.75)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
            BorderRadius::all(Val::Px(8.0)),
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(0.0),
                padding: UiRect::left(Val::Px(16.0)),
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            ProgressBarInner,
        ))
        .with_child((
            Text::new("0/0".to_owned()),
            TextColor(Color::WHITE),
            ProgressBarText,
        ));
}

fn update_progress(
    progress: Res<ProgressTracker<GameState>>,
    mut q_bar_inner: Query<&mut Node, With<ProgressBarInner>>,
    mut q_bar_text: Query<&mut Text, With<ProgressBarText>>,
) {
    let progress = progress.get_global_progress();
    let ratio: f32 = progress.into();
    q_bar_inner.iter_mut().for_each(|mut node| {
        node.width = Val::Percent(ratio * 100.0);
    });
    q_bar_text.iter_mut().for_each(|mut text| {
        text.0 = format!("{}/{}", progress.done, progress.total);
    });
}
