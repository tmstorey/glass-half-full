use bevy::prelude::*;
use bevy_pkv::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

pub mod character;
pub mod controls;
mod interactions;
pub mod level;
mod parallax;
mod physics;
mod tiles;
mod ui;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Reflect,
    Resource,
    Serialize,
    Deserialize,
)]
pub enum Season {
    #[default]
    Summer,
    Autumn,
    Winter,
    Spring,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Reflect,
    Resource,
    Serialize,
    Deserialize,
)]
pub struct PlayerLevel(pub u8);

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Reflect,
    Resource,
    Serialize,
    Deserialize,
)]
pub struct GameLevel(pub u8);

impl Default for GameLevel {
    fn default() -> Self {
        GameLevel(1)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Reflect,
    Resource,
    Serialize,
    Deserialize,
)]
pub struct CompletedYear(pub bool);

impl Season {
    /// Get the next season in the cycle
    pub fn next(&self) -> Self {
        match self {
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
            Season::Spring => Season::Summer,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.insert_resource(PkvStore::new("tmstorey", "glass-half-full"));
    app.init_persistent_resource::<PlayerLevel>();
    app.init_persistent_resource::<GameLevel>();
    app.init_persistent_resource::<CompletedYear>();
    app.init_persistent_resource::<Season>();
    app.add_plugins(tiles::plugin);
    app.add_plugins(character::plugin);
    app.add_plugins(controls::plugin);
    app.add_plugins(physics::plugin);
    app.add_plugins(level::plugin);
    app.add_plugins(ui::plugin);
    app.add_plugins(interactions::plugin);
}
