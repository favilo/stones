use bevy::{
    app::{self, AppExit},
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
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 1.0)),
                ..Default::default()
            },
            Menu,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_sections([TextSection {
                    value: "Mancala: African Stones".to_string(),
                    style: TextStyle {
                        font_size: 60.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                }]),
                ..Default::default()
            });
            parent
                .spawn((
                    StartButton,
                    ButtonBundle {
                        background_color: BackgroundColor(Color::NONE),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_sections([TextSection {
                            value: "Start Game".to_string(),
                            style: TextStyle {
                                font_size: 40.0,
                                color: Color::WHITE,
                                ..Default::default()
                            },
                        }]),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    QuitButton,
                    ButtonBundle {
                        background_color: BackgroundColor(Color::NONE),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_sections([TextSection {
                            value: "Quit".to_string(),
                            style: TextStyle {
                                font_size: 40.0,
                                color: Color::WHITE,
                                ..Default::default()
                            },
                        }]),
                        ..Default::default()
                    });
                });
        });
}

fn update_menu(
    mut state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    start_interactions: Query<(&Interaction, &Children), (With<StartButton>, Changed<Interaction>)>,
    quit_interactions: Query<(&Interaction, &Children), (With<QuitButton>, Changed<Interaction>)>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children) in start_interactions.iter() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match interaction {
            Interaction::Pressed => state.set(GameState::Playing),
            Interaction::Hovered => {
                text.sections[0].style.color = Color::GOLD;
            }
            Interaction::None => {
                text.sections[0].style.color = Color::WHITE;
            }
        };
    }

    for (interaction, children) in quit_interactions.iter() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match interaction {
            Interaction::Pressed => {
                exit.send(AppExit);
            }
            Interaction::Hovered => {
                text.sections[0].style.color = Color::GOLD;
            }
            Interaction::None => {
                text.sections[0].style.color = Color::WHITE;
            }
        };
    }
}
