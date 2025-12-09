use super::graph::{PlatformGraph, SmartTerrain};
use super::objects::{
    ContainerState, FireState, WaterType, spawn_container, spawn_fire, spawn_snow, spawn_water,
};
use crate::game::tiles::{GridPosition, TILE_SIZE, TerrainTile};
use crate::screens::Screen;
use bevy::prelude::*;

/// Resource that defines where the player spawns
#[derive(Resource, Debug)]
pub struct PlayerSpawnPoint {
    pub position: Vec3,
}

impl Default for PlayerSpawnPoint {
    fn default() -> Self {
        Self {
            position: Vec3::new(TILE_SIZE * 5.0, TILE_SIZE * 4.0, 0.0),
        }
    }
}

/// Spawns platforms from a generated graph
pub fn spawn_platforms_from_graph(commands: &mut Commands, graph: &PlatformGraph) {
    for (node_idx, node) in graph.nodes.iter().enumerate() {
        let platform_width_tiles = (node.width / TILE_SIZE) as i32;
        let platform_x_tiles = (node.position.x / TILE_SIZE) as i32;
        let platform_y_tiles = (node.position.y / TILE_SIZE) as i32;

        // Spawn platform tiles
        for x_offset in 0..platform_width_tiles {
            let grid_pos = GridPosition::primary(platform_x_tiles + x_offset, platform_y_tiles);

            commands.spawn((
                Name::new(format!(
                    "Platform {} tile at ({}, {})",
                    node_idx,
                    platform_x_tiles + x_offset,
                    platform_y_tiles
                )),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }
}

/// Spawns terrain objects from a generated graph
pub fn spawn_terrain_objects_from_graph(
    commands: &mut Commands,
    asset_server: &AssetServer,
    graph: &PlatformGraph,
) {
    for node in &graph.nodes {
        for terrain in &node.terrain_objects {
            // Calculate world position - center objects on the platform
            let world_pos = Vec3::new(
                node.position.x + node.width / 2.0,
                node.position.y + TILE_SIZE * 2., // Place above platform
                0.0,
            );

            match terrain {
                SmartTerrain::WaterSource { active } => {
                    // Spawn as a waterfall or water pool
                    let water_type = if *active {
                        WaterType::WaterMiddle
                    } else {
                        WaterType::WaterfallTop
                    };

                    super::objects::spawn_water(commands, asset_server, world_pos, water_type);
                }
                SmartTerrain::SnowSource => {
                    super::objects::spawn_snow(commands, asset_server, world_pos);
                }
                SmartTerrain::Fire { extinguished } => {
                    let fire_state = if *extinguished {
                        FireState::Extinguished
                    } else {
                        FireState::Active
                    };

                    super::objects::spawn_fire(commands, asset_server, world_pos, fire_state);
                }
                SmartTerrain::GoalContainer { .. } => {
                    spawn_container(commands, asset_server, world_pos, ContainerState::Empty);
                }
                SmartTerrain::SwitchContainer { .. } => {
                    // For now, spawn as a regular container
                    // TODO: Differentiate visually or add switch logic
                    spawn_container(commands, asset_server, world_pos, ContainerState::Empty);
                }
                SmartTerrain::Switch { .. } => {
                    // TODO: Implement switch object
                    // For now, just log that we need this
                    info!("Switch terrain not yet implemented at {:?}", world_pos);
                }
                SmartTerrain::MovingPlatform { .. } => {
                    // TODO: Implement moving platform
                    info!("Moving platform not yet implemented at {:?}", world_pos);
                }
            }
        }
    }
}

/// Updates the player spawn point based on the graph's start node
pub fn update_player_spawn_point(graph: &PlatformGraph, spawn_point: &mut PlayerSpawnPoint) {
    if let Some(start_node) = graph.get_node(graph.start) {
        spawn_point.position = Vec3::new(
            start_node.position.x + start_node.width / 2.0,
            start_node.position.y + TILE_SIZE * 4.0,
            0.0,
        );
    }
}
