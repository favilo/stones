use avian3d::{math::Quaternion, prelude::Collider};
use bevy::{app, ecs::system::SystemState, prelude::*};

use crate::GameAssets;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, _app: &mut App) {
        // app;
    }
}

// #[derive(Resource, Debug, Clone, Deref, DerefMut)]
/// Construct a board collider from the `obj` file exported from CoACD
// pub(crate) struct BoardCollider(pub Collider);

// impl FromWorld for BoardCollider {
//     fn from_world(world: &mut World) -> Self {
//         let mut outer_state =
//             SystemState::<(Res<GameAssets>, ResMut<Assets<Scene>>, Res<Assets<Mesh>>)>::new(world);
//         let (game_assets, mut scenes, meshes) = outer_state.get_mut(world);
//         let collider_scene = game_assets.board_collider.clone();
//         let scene: &mut Scene = scenes.get_mut(&collider_scene).unwrap();

//         let mut mesh_query = scene.world.query::<&Mesh3d>();
//         let colliders = mesh_query
//             .iter(&scene.world)
//             .map(|mesh| meshes.get(&mesh.0).unwrap())
//             .map(|mesh| Collider::convex_hull_from_mesh(mesh).unwrap())
//             .map(|collider| (Vec3::ZERO, Quaternion::IDENTITY, collider))
//             .collect();

//         let collider = Collider::compound(colliders);
//         Self(collider)
//     }
// }

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Failed to read collider: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to unpack collider: {0:?}")]
    MessagePackError(#[from] rmp_serde::decode::Error),
}
