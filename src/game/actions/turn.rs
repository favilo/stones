use std::time::Duration;

use avian3d::prelude::{AngularVelocity, LinearVelocity, Sleeping};
use bevy::{app, ecs::system::SystemId, prelude::*};
use bevy_sequential_actions::{
    actions, Action, ActionsProxy, ModifyActions, SequentialActions, StopReason,
};

use crate::{
    game::{Board, Player, PlayerTurn, Stone},
    rules::variants::Index,
};

use super::{ui::UpdateLabels, ChainActions, RunSystem, SystemInResource};

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let perform_move = app.register_system(perform_move);
        app.insert_resource(PlayerMoveResource(perform_move));
        app.add_systems(Update, update_wait_timer);
    }
}

pub type PlayerMove = RunSystem<PlayerMoveResource, Index, In<Index>>;

#[derive(Clone, Copy, Debug, Resource, Deref)]
pub struct PlayerMoveResource(SystemId<In<Index>>);

impl SystemInResource for PlayerMoveResource {
    type Input = In<Index>;

    fn system_id(&self) -> SystemId<Self::Input> {
        self.0
    }
}

/// The system that will perform the move that the player selected.
pub fn perform_move(
    index: In<Index>,
    mut board: ResMut<Board>,
    mut p_turn: ResMut<PlayerTurn>,
    mut lights: Query<&mut PointLight>,
    agent: Single<Entity, With<SequentialActions>>,
    mut commands: Commands,
) {
    let PlayerTurn::Player(turn) = *p_turn else {
        panic!("How can we perform a move, if there is no player?");
    };

    *p_turn = PlayerTurn::None;

    let actions = board.perform_move(*index, Player(turn));
    // Make all the lights go out for now.
    lights.par_iter_mut().for_each(|mut light| {
        light.intensity = 0.0;
    });
    commands
        .actions(*agent)
        .start(false)
        .add(UpdateLabels::new())
        .add(actions);
}

#[derive(Debug, Clone, Component, Deref, DerefMut)]
pub struct WaitTimer {
    pub timer: Timer,
}

fn update_wait_timer(mut timers: Query<&mut WaitTimer, With<SequentialActions>>, time: Res<Time>) {
    timers.iter_mut().for_each(|mut timer| {
        timer.tick(time.delta());
    });
}

pub struct Wait {
    pub duration: Duration,
}

impl Wait {
    pub fn from_secs(secs: f32) -> Self {
        Self {
            duration: Duration::from_secs_f32(secs),
        }
    }
}

impl Action for Wait {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        let Some(w_timer) = world.get::<WaitTimer>(agent) else {
            return true;
        };

        w_timer.finished()
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        let Some(mut timer) = world.get_mut::<WaitTimer>(agent) else {
            world.entity_mut(agent).insert(WaitTimer {
                timer: Timer::new(self.duration, TimerMode::Once),
            });
            return false;
        };
        timer.unpause();

        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        let Some(agent) = agent else {
            return;
        };

        if reason == StopReason::Paused {
            let mut timer = world.get_mut::<WaitTimer>(agent).unwrap();
            timer.pause();
        } else {
            world.entity_mut(agent).remove::<WaitTimer>();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MovePiece {
    stone: Entity,
    destination: Vec3,
}

impl MovePiece {
    pub fn new_action(stone: Entity, destination: Vec3) -> ChainActions<2> {
        ChainActions::new(actions![Self { stone, destination }, Wait::from_secs(0.1)])
    }
}

impl Action for MovePiece {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, world: &mut World) -> bool {
        let mut query =
            QueryState::<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>::new(world);
        let (mut transform, mut linear_velocity, mut angular_veocity) =
            query.get_mut(world, self.stone).unwrap();
        transform.translation = self.destination;
        transform.rotation = Quat::from_rotation_x(90.0_f32.to_radians());
        **linear_velocity = Vec3::ZERO;
        **angular_veocity = Vec3::ZERO;

        true
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}

pub struct NextPlayer(pub Player);

impl Action for NextPlayer {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        let mut p_turn = world.resource_mut::<PlayerTurn>();
        *p_turn = PlayerTurn::Player(*self.0);

        world.actions(agent).start(false).add((
            UpdateLabels::new(),
            Wait::from_secs(0.5),
            SleepPieces,
        ));

        true
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}

pub struct SleepPieces;

impl Action for SleepPieces {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, world: &mut World) -> bool {
        let stones = world
            .query_filtered::<Entity, With<Stone>>()
            .iter(world)
            .collect::<Vec<_>>();
        stones.into_iter().for_each(|stone| {
            let mut stone = world.entity_mut(stone);
            stone.insert(Sleeping);
        });

        true
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}
