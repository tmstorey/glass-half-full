use crate::{PausableSystems, screens::Screen};
use bevy::prelude::*;
use bevy_pkv::prelude::*;
use strum::Display;

pub mod character;
pub mod controls;
pub mod level;
mod parallax;
mod physics;
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect, Resource)]
pub struct PlayerLevel(pub u8);

pub fn plugin(app: &mut App) {
    app.insert_resource(PkvStore::new("tmstorey", "glass-half-full"));
    app.init_resource::<Season>();
    app.init_resource::<PlayerLevel>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
    app.add_systems(
        Update,
        scroll_parallax
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_plugins(tiles::plugin);
    app.add_plugins(character::plugin);
    app.add_plugins(controls::plugin);
    app.add_plugins(physics::plugin);
}

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![parallax_background(Season::Summer, asset_server)],
    ));

    // Spawn the flat level
    level::spawn_flat_level(&mut commands, 40, -2, 6);
}
