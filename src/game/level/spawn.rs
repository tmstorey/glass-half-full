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

/// Spawns platforms from a generated graph with exclusion zones
pub fn spawn_platforms_from_graph(
    commands: &mut Commands,
    graph: &PlatformGraph,
    exclusions: &std::collections::HashSet<(i32, i32)>,
) {
    for (node_idx, node) in graph.nodes.iter().enumerate() {
        let platform_width_tiles = (node.width / TILE_SIZE) as i32;
        let platform_x_tiles = (node.position.x / TILE_SIZE) as i32;
        let platform_y_tiles = (node.position.y / TILE_SIZE) as i32;

        // Spawn platform tiles based on height
        for y_offset in 0..node.height as i32 {
            for x_offset in 0..platform_width_tiles {
                let tile_x = platform_x_tiles + x_offset;
                let tile_y = platform_y_tiles - y_offset;

                // Skip if this position is excluded (water placement)
                if exclusions.contains(&(tile_x, tile_y)) {
                    continue;
                }

                let grid_pos = GridPosition::primary(tile_x, tile_y);

                commands.spawn((
                    Name::new(format!(
                        "Platform {} tile at ({}, {})",
                        node_idx, tile_x, tile_y
                    )),
                    grid_pos,
                    TerrainTile::Grass,
                    DespawnOnExit(Screen::Gameplay),
                ));
            }
        }
    }
}

/// Spawns terrain objects from a generated graph and returns grid positions that should be excluded from platform spawning
pub fn spawn_terrain_objects_from_graph(
    commands: &mut Commands,
    asset_server: &AssetServer,
    graph: &PlatformGraph,
) -> std::collections::HashSet<(i32, i32)> {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut exclusions = std::collections::HashSet::new();

    for node in &graph.nodes {
        for terrain in &node.terrain_objects {
            // Calculate world position - center objects on the platform
            let world_pos = Vec3::new(
                node.position.x + node.width / 2.0,
                node.position.y + TILE_SIZE, // Place on platform level
                0.0,
            );

            match terrain {
                SmartTerrain::WaterSource { active } => {
                    if *active {
                        // Spawn a water pool with WaterLeft, WaterMiddle (1-3 tiles), WaterRight
                        let middle_count = rng.random_range(1..=3);
                        let total_width = 2 + middle_count; // left + middle + right

                        // Calculate starting X position to center the water on the platform
                        let start_x_tile = (node.position.x / TILE_SIZE) as i32
                            + ((node.width / TILE_SIZE) as i32 - total_width) / 2;
                        let y_tile = (node.position.y / TILE_SIZE) as i32 + 1; // One tile above platform

                        // Spawn water tiles
                        for x_offset in 0..total_width {
                            let water_type = if x_offset == 0 {
                                WaterType::WaterLeft
                            } else if x_offset == total_width - 1 {
                                WaterType::WaterRight
                            } else {
                                WaterType::WaterMiddle
                            };

                            let tile_x = start_x_tile + x_offset;
                            let water_grid_pos = GridPosition::primary(tile_x, y_tile);

                            spawn_water(commands, asset_server, water_grid_pos, water_type);

                            // Add water position to exclusions so grass doesn't spawn here
                            exclusions.insert((tile_x, y_tile));
                        }
                    } else {
                        // Inactive water source - could spawn as dry basin or nothing
                        // For now, skip spawning
                    }
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
                    // Place container on platform level
                    let container_pos = Vec3::new(
                        node.position.x + node.width / 2.0,
                        node.position.y + TILE_SIZE * 2.,
                        0.0,
                    );
                    spawn_container(commands, asset_server, container_pos, ContainerState::Empty);
                }
                SmartTerrain::SwitchContainer { .. } => {
                    // For now, spawn as a regular container
                    // TODO: Differentiate visually or add switch logic
                    let container_pos = Vec3::new(
                        node.position.x + node.width / 2.0,
                        node.position.y + TILE_SIZE,
                        0.0,
                    );
                    spawn_container(commands, asset_server, container_pos, ContainerState::Empty);
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

    exclusions
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
