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
        .add_systems(Update, (update_menu,).run_if(in_state(GameState::Menu)))
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
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont::from_font_size(40.0),
                        TextColor(Color::WHITE),
                    ));
                });
            parent
                .spawn((QuitButton, Button, BackgroundColor(Color::NONE)))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quit"),
                        TextFont::from_font_size(40.0),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

type InteractionData<'world> = (&'world Interaction, &'world Children);

fn update_menu(
    mut state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    start_interactions: Query<InteractionData, (With<StartButton>, Changed<Interaction>)>,
    quit_interactions: Query<InteractionData, (With<QuitButton>, Changed<Interaction>)>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, children) in start_interactions.iter() {
        let mut color = text_query.get_mut(children[0]).unwrap();
        match interaction {
            Interaction::Pressed => state.set(GameState::Playing),
            Interaction::Hovered => {
                **color = Color::Srgba(GOLD);
            }
            Interaction::None => {
                **color = Color::WHITE;
            }
        };
    }

    for (interaction, children) in quit_interactions.iter() {
        let mut color = text_query.get_mut(children[0]).unwrap();
        match interaction {
            Interaction::Pressed => {
                exit.send(AppExit::Success);
            }
            Interaction::Hovered => {
                **color = Color::Srgba(GOLD);
            }
            Interaction::None => {
                **color = Color::WHITE;
            }
        };
    }
}
