use super::graph::{NodeId, PlatformGraph, PlatformLayout, PlatformNode, SmartTerrain};
use super::objects::{
    ContainerState, FireState, WaterType, spawn_container, spawn_fire, spawn_snow, spawn_water,
};
use crate::game::tiles::{GridPosition, TILE_SIZE, TerrainTile};
use crate::screens::Screen;
use bevy::prelude::*;
use std::collections::HashMap;

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

/// Spawns a complete level from a platform graph and layout map
pub fn spawn_level_from_graph(
    commands: &mut Commands,
    asset_server: &AssetServer,
    graph: &PlatformGraph,
    layouts: &HashMap<NodeId, PlatformLayout>,
) {
    for (i, node) in graph.nodes.iter().enumerate() {
        let node_id = NodeId(i);
        let layout = match layouts.get(&node_id) {
            Some(l) => l,
            None => {
                warn!("No layout found for node {:?}, skipping", node_id);
                continue;
            }
        };

        // Check what terrain this platform contains
        let has_water_source = node
            .terrain_objects
            .iter()
            .any(|t| matches!(t, SmartTerrain::WaterSource { active: true }));

        if has_water_source {
            // Spawn specialized water platform (2 tiles high, with water integrated)
            spawn_water_platform(commands, asset_server, node, layout);
            info!("water");
        } else {
            // Spawn standard grass platform
            spawn_standard_platform(commands, node, layout);
            info!("standard");
        }

        // Spawn other terrain objects (fire, snow, containers, etc.)
        spawn_other_terrain_objects(commands, asset_server, node, layout);
    }
}

/// Helper: Spawns a platform with integrated water source
fn spawn_water_platform(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _node: &PlatformNode,
    layout: &PlatformLayout,
) {
    use rand::Rng;
    let mut rng = rand::rng();

    let platform_width_tiles = (layout.width / TILE_SIZE) as i32;

    // Layout position is the CENTER of the platform, so calculate left edge
    let left_edge_x = layout.position.x - (layout.width / 2.0);
    let platform_x_tiles = (left_edge_x / TILE_SIZE) as i32;
    let platform_y_tiles = (layout.position.y / TILE_SIZE) as i32;

    // Generate water layout (random 3-5 tiles wide)
    let middle_count = rng.random_range(1..=3);
    let total_water_width = 2 + middle_count;
    let water_start_x = platform_x_tiles + ((platform_width_tiles - total_water_width) / 2);

    // Spawn platform tiles (2 tiles high)
    for y_offset in 0..2 {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let tile_y = platform_y_tiles - y_offset;
            let grid_pos = GridPosition::primary(tile_x, tile_y);

            // Skip top layer where water will be
            if y_offset == 0 {
                if tile_x >= water_start_x && tile_x < water_start_x + total_water_width + 1 {
                    continue;
                }

                commands.spawn((
                    Name::new(format!("Water platform tile at ({}, {})", tile_x, tile_y)),
                    grid_pos,
                    TerrainTile::Grass,
                    DespawnOnExit(Screen::Gameplay),
                ));
            } else {
                commands.spawn((
                    Name::new(format!("Water platform tile at ({}, {})", tile_x, tile_y)),
                    grid_pos,
                    TerrainTile::Dirt,
                    DespawnOnExit(Screen::Gameplay),
                ));
            }
        }
    }

    // Spawn water tiles on top layer
    for x_offset in 0..=total_water_width + 1 {
        let z = if x_offset <= 0 || x_offset > total_water_width {
            -0.1
        } else {
            20.
        };

        let tile_x = water_start_x + x_offset;
        let water_grid_pos = GridPosition::primary(tile_x, platform_y_tiles + 1);

        spawn_water(
            commands,
            asset_server,
            water_grid_pos,
            WaterType::WaterMiddle,
            z,
        );
    }
}

/// Helper: Spawns a standard grass platform
fn spawn_standard_platform(commands: &mut Commands, _node: &PlatformNode, layout: &PlatformLayout) {
    let platform_width_tiles = (layout.width / TILE_SIZE) as i32;

    // Layout position is the CENTER of the platform, so calculate left edge
    let left_edge_x = layout.position.x - (layout.width / 2.0);
    let platform_x_tiles = (left_edge_x / TILE_SIZE) as i32;
    let platform_y_tiles = (layout.position.y / TILE_SIZE) as i32;

    // Spawn platform tiles based on height
    for y_offset in 0..layout.height as i32 {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let tile_y = platform_y_tiles - y_offset;

            let grid_pos = GridPosition::primary(tile_x, tile_y);
            commands.spawn((
                Name::new(format!("Platform tile at ({}, {})", tile_x, tile_y)),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }
}

/// Helper: Spawns non-water terrain objects (fire, snow, containers, etc.)
fn spawn_other_terrain_objects(
    commands: &mut Commands,
    asset_server: &AssetServer,
    node: &PlatformNode,
    layout: &PlatformLayout,
) {
    for terrain in &node.terrain_objects {
        let world_pos = Vec3::new(
            layout.position.x + layout.width / 2.0,
            layout.position.y + TILE_SIZE,
            0.0,
        );

        match terrain {
            SmartTerrain::WaterSource { .. } => {
                // Already handled in spawn_water_platform
                continue;
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
                let container_pos = Vec3::new(
                    layout.position.x + layout.width / 2.0,
                    layout.position.y + TILE_SIZE * 2.0,
                    0.0,
                );
                spawn_container(commands, asset_server, container_pos, ContainerState::Empty);
            }
            SmartTerrain::SwitchContainer { .. } => {
                // For now, spawn as a regular container
                // TODO: Differentiate visually or add switch logic
                let container_pos = Vec3::new(
                    layout.position.x + layout.width / 2.0,
                    layout.position.y + TILE_SIZE,
                    0.0,
                );
                spawn_container(commands, asset_server, container_pos, ContainerState::Empty);
            }
            SmartTerrain::Switch { .. } => {
                // TODO: Implement switch object
                info!("Switch terrain not yet implemented at {:?}", world_pos);
            }
            SmartTerrain::MovingPlatform { .. } => {
                // TODO: Implement moving platform
                info!("Moving platform not yet implemented at {:?}", world_pos);
            }
        }
    }
}

/// Updates the player spawn point based on the graph's start node and layout
pub fn update_player_spawn_point(
    graph: &PlatformGraph,
    layouts: &HashMap<NodeId, PlatformLayout>,
    spawn_point: &mut PlayerSpawnPoint,
) {
    if let Some(start_layout) = layouts.get(&graph.start) {
        spawn_point.position = Vec3::new(
            start_layout.position.x + start_layout.width / 2.0,
            start_layout.position.y + TILE_SIZE * 4.0,
            0.0,
        );
    }
}
