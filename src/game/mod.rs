
use bevy::prelude::*;
//use bevy::image::{ImageLoaderSettings, ImageSampler};
use crate::{
    screens::Screen,
};


mod parallax;
use parallax::{parallax_background, Season};


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
            parallax_background(Season::Summer, asset_server)
        ],
    ));
}
