use bevy::{app, prelude::*};

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<MoveEvent>();
    }
}

#[derive(Debug, Event, Clone, Copy)]
pub enum MoveEvent {
    HoleClicked(usize, usize),
}
