
use bevy::prelude::*;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use crate::{
    screens::Screen,
};


pub fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            Sprite::from_image(asset_server.load_with_settings(
                "images/bg/summer-1.epng",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            )),
        ],
    ));
}
