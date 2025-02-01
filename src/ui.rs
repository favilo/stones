use bevy::{
    app::{self, AppExit},
    color::palettes::css::GOLD,
    prelude::*,
};

use crate::{assets::GameAssets, game::GameState};

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu);
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Menu;

#[derive(Debug, Clone, Copy, Component)]
struct StartButton;

#[derive(Debug, Clone, Copy, Component)]
struct QuitButton;

fn setup_menu(mut commands: Commands, game_assets: Res<GameAssets>) {
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
            StateScoped(GameState::Menu),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Mancala: African Stones"),
                TextFont::from_font(game_assets.main_font.clone()).with_font_size(60.0),
                TextColor(Color::WHITE),
            ));
            parent
                .spawn((StartButton, Button, BackgroundColor(Color::NONE)))
                .observe(hover_button(Color::Srgba(GOLD)))
                .observe(unhover_button(Color::WHITE))
                .observe(
                    |_trigger: Trigger<Pointer<Click>>,
                     mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::Playing);
                    },
                )
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont::from_font(game_assets.main_font.clone()).with_font_size(40.0),
                        TextColor(Color::WHITE),
                    ));
                });
            parent
                .spawn((QuitButton, Button, BackgroundColor(Color::NONE)))
                .observe(hover_button(Color::WHITE))
                .observe(unhover_button(Color::WHITE))
                .observe(
                    |_click: Trigger<Pointer<Click>>, mut exit: EventWriter<AppExit>| {
                        exit.send(AppExit::Success);
                    },
                )
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quit"),
                        TextFont::from_font(game_assets.main_font.clone()).with_font_size(40.0),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn hover_button(
    new_color: Color,
) -> impl FnMut(Trigger<Pointer<Over>>, Query<&Children>, Query<&mut TextColor>) {
    move |trigger: Trigger<Pointer<Over>>,
          children: Query<&Children>,
          mut colors: Query<&mut TextColor>| {
        let children = children.get(trigger.entity()).unwrap();
        let mut color = colors.get_mut(children[0]).unwrap();
        **color = new_color;
    }
}

pub fn unhover_button(
    new_color: Color,
) -> impl FnMut(Trigger<Pointer<Out>>, Query<&Children>, Query<&mut TextColor>) {
    move |trigger: Trigger<Pointer<Out>>,
          children: Query<&Children>,
          mut colors: Query<&mut TextColor>| {
        let children = children.get(trigger.entity()).unwrap();
        let mut color = colors.get_mut(children[0]).unwrap();
        **color = new_color;
    }
}
