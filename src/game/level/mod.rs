use bevy::prelude::*;

use super::tiles::{GridPosition, TerrainTile};
use crate::screens::Screen;

mod objects;

pub fn plugin(app: &mut App) {
    app.add_plugins(objects::plugin);
}

/// Spawns a flat grass level
pub fn spawn_flat_level(commands: &mut Commands, width: i32, ground_y: i32, thickness: i32) {
    for x in 0..width {
        for y in 0..thickness {
            let grid_pos = GridPosition::primary(x, ground_y - y);

            commands.spawn((
                Name::new(format!("Terrain at ({}, {})", x, ground_y - y)),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }
}
