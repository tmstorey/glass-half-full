
use bevy::prelude::*;
use crate::{
    screens::Screen,
    PausableSystems,
};

mod parallax;
use parallax::{parallax_background, scroll_parallax, Season};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
    app.add_systems(
        Update,
        scroll_parallax
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay))
    );
}

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
