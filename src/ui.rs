use bevy::{
    app::{self, AppExit},
    color::palettes::css::GOLD,
    prelude::*,
};

use crate::{cleanup, game::GameState, graphics::setup_graphics};

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Menu),
            (setup_graphics, setup_menu).chain(),
        )
        .add_systems(OnExit(GameState::Menu), cleanup::<Menu>);
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Menu;

#[derive(Debug, Clone, Copy, Component)]
struct StartButton;

#[derive(Debug, Clone, Copy, Component)]
struct QuitButton;

fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            Name::new("MainMenu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 1.0)),
            Menu,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Mancala: African Stones"),
                TextFont::from_font_size(60.0),
                TextColor(Color::WHITE),
            ));
            parent
                .spawn((StartButton, Button, BackgroundColor(Color::NONE)))
                .observe(hover_button)
                .observe(unhover_button)
                .observe(
                    |_trigger: Trigger<Pointer<Click>>,
                     mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::Playing);
                    },
                )
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont::from_font_size(40.0),
                        TextColor(Color::WHITE),
                    ));
                });
            parent
                .spawn((QuitButton, Button, BackgroundColor(Color::NONE)))
                .observe(hover_button)
                .observe(unhover_button)
                .observe(
                    |_click: Trigger<Pointer<Click>>, mut exit: EventWriter<AppExit>| {
                        exit.send(AppExit::Success);
                    },
                )
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quit"),
                        TextFont::from_font_size(40.0),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn hover_button(
    trigger: Trigger<Pointer<Over>>,
    children: Query<&Children>,
    mut colors: Query<&mut TextColor>,
) {
    let children = children.get(trigger.entity()).unwrap();
    let mut color = colors.get_mut(children[0]).unwrap();
    **color = Color::Srgba(GOLD);
}

pub fn unhover_button(
    trigger: Trigger<Pointer<Out>>,
    children: Query<&Children>,
    mut colors: Query<&mut TextColor>,
) {
    let children = children.get(trigger.entity()).unwrap();
    let mut color = colors.get_mut(children[0]).unwrap();
    **color = Color::WHITE;
}
