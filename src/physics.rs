use avian3d::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum GameLayer {
    #[default]
    Default,
    PhysicsObject,
    MouseObject,
}
