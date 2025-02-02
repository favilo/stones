use bevy::{app, prelude::*};

use crate::rules::variants::Index;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveEvent>();
    }
}

#[derive(Debug, Event, Clone, Copy)]
pub enum MoveEvent {
    HoleClicked(Index),
}
