use std::marker::PhantomData;

use bevy::ecs::system::SystemId;
use bevy::{app, prelude::*};
use bevy_sequential_actions::{
    Action, ActionsProxy, AddOrder, BoxedAction, DropReason, IntoBoxedAction, IntoBoxedActions,
    ModifyActions, SequentialActions, StopReason,
};

pub mod board;
pub mod turn;
pub mod ui;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((board::Plugin, turn::Plugin, ui::Plugin))
            .add_systems(Startup, spawn_agent);
    }
}

fn spawn_agent(mut commands: Commands) {
    commands.spawn(SequentialActions);
}

trait SystemInResource: Resource {
    type Input: SystemInput;

    fn system_id(&self) -> SystemId<Self::Input>;
}

#[derive(Clone, Copy, Debug)]
pub struct RunSystem<R, I = (), P = ()>
where
    P: SystemInput,
{
    input: I,
    _phantom: PhantomData<(R, P)>,
}

impl<R> Default for RunSystem<R>
where
    R: Resource,
{
    fn default() -> Self {
        Self {
            input: (),
            _phantom: PhantomData,
        }
    }
}

impl<R, I> Default for RunSystem<R, I, In<I>>
where
    R: Resource,
    I: SystemInput + Send + Sync + Default + 'static,
    for<'a> I::Inner<'a>: Clone,
{
    fn default() -> Self {
        Self {
            input: I::default(),
            _phantom: PhantomData,
        }
    }
}

impl<R> RunSystem<R>
where
    R: Resource,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<R, I> RunSystem<R, I, In<I>>
where
    R: Resource,
    I: Clone + Send + Sync + 'static,
    In<I>: SystemInput + Send + Sync + 'static,
{
    pub fn with_input(input: I) -> Self {
        Self {
            input,
            _phantom: PhantomData,
        }
    }
}

impl<R, I, P> Action for RunSystem<R, I, P>
where
    R: SystemInResource<Input = P>,
    I: Clone + Send + Sync + 'static,
    for<'i> P: SystemInput<Inner<'i> = I> + Send + Sync + 'static,
{
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, world: &mut World) -> bool {
        world
            .run_system_with_input(
                world.get_resource::<R>().unwrap().system_id(),
                self.input.clone(),
            )
            .unwrap();

        true
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}

#[derive(Debug)]
pub struct ChainActions<const N: usize> {
    actions: [BoxedAction; N],
    index: usize,
    canceled: bool,
}

impl<const N: usize> ChainActions<N> {
    pub fn new(actions: [BoxedAction; N]) -> Self {
        Self {
            actions,
            index: 0,
            canceled: false,
        }
    }
}

impl<const N: usize> Action for ChainActions<N> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.actions[self.index].is_finished(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_add(agent, world));
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        self.actions[self.index].on_start(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        self.actions[self.index].on_stop(agent, world, reason);
        self.canceled = reason == StopReason::Canceled
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        self.actions[self.index].on_remove(agent, world);
    }

    fn on_drop(mut self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        self.index += 1;

        if self.index >= N {
            return;
        }

        if self.canceled || reason != DropReason::Done {
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
            return;
        }

        let Some(agent) = agent else { return };

        world
            .actions(agent)
            .start(false)
            .order(AddOrder::Front)
            .add(self as BoxedAction);
    }

    // fn type_name(&self) -> &'static str {
    //     format!("[{}]", self.actions.iter().map(|a| a.type_name()).collect::<Vec<_>>().join(", ")).as_str()
    // }
}
