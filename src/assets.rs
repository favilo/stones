use bevy::{app, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, _app: &mut App) {
        // app;
    }
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "board_scene")]
    pub board_scene: Handle<Scene>,

    #[asset(key = "stone_mesh")]
    pub stone_mesh: Handle<Mesh>,

    #[asset(key = "stone_materials", collection(typed))]
    pub stone_materials: Vec<Handle<StandardMaterial>>,

    #[asset(key = "main_font")]
    pub main_font: Handle<Font>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Failed to read collider: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to unpack collider: {0:?}")]
    MessagePackError(#[from] rmp_serde::decode::Error),
}
