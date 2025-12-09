#![allow(unused_imports)]

use bevy::prelude::*;

use super::tiles::{GridPosition, TerrainTile};
use crate::screens::Screen;

mod causality;
mod example;
mod generator;
mod graph;
pub mod objects;
mod spawn;
mod templates;

pub use causality::{BucketContent, CausalityChain, CausalityNode, Cause, Effect};
pub use example::generate_example_level;
pub use generator::{CausalityGenerator, Difficulty, GeneratorConfig};
pub use graph::{ConnectionType, Edge, NodeId, PlatformGraph, PlatformNode, SmartTerrain};
pub use spawn::{
    PlayerSpawnPoint, spawn_platforms_from_graph, spawn_terrain_objects_from_graph,
    update_player_spawn_point,
};
pub use templates::{
    create_branching_template, create_cul_de_sac_template, create_linear_template,
    create_random_linear_segment,
};

pub fn plugin(app: &mut App) {
    app.init_resource::<PlayerSpawnPoint>();
    app.init_resource::<BucketContent>();
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
