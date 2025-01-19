use avian3d::prelude::Collider;
use bevy::{
    app,
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<ColliderLoader>()
            .init_asset::<ColliderWrapper>();
    }
}

#[derive(Debug, Asset, TypePath)]
#[allow(dead_code)]
pub(crate) struct ColliderWrapper(pub Collider);

#[derive(Debug, Default)]
pub(crate) struct ColliderLoader;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Failed to read collider: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to unpack collider: {0:?}")]
    MessagePackError(#[from] rmp_serde::decode::Error),
}

impl AssetLoader for ColliderLoader {
    type Asset = ColliderWrapper;
    type Settings = ();
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<ColliderWrapper, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let collider = rmp_serde::from_slice(&bytes)?;
        Ok(ColliderWrapper(collider))
    }

    fn extensions(&self) -> &[&str] {
        &["msp"]
    }
}
