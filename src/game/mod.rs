use crate::{PausableSystems, screens::Screen};
use bevy::prelude::*;
use strum::Display;

mod character;
mod parallax;
mod tiles;
use parallax::{parallax_background, scroll_parallax};

#[derive(
    Clone, Copy, Debug, Default, Display, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect, Resource,
)]
pub enum Season {
    #[default]
    Summer,
    Autumn,
    Winter,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Season>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
    app.add_systems(
        Update,
        scroll_parallax
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_plugins(tiles::plugin);
    app.add_plugins(character::plugin);
}

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![parallax_background(Season::Summer, asset_server)],
    ));
}
