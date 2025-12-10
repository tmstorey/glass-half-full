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
    use super::graph::PlatformType;

    for (i, node) in graph.nodes.iter().enumerate() {
        let node_id = NodeId(i);
        let layout = match layouts.get(&node_id) {
            Some(l) => l,
            None => {
                warn!("No layout found for node {:?}, skipping", node_id);
                continue;
            }
        };

        // Dispatch based on platform type
        match node.platform_type {
            PlatformType::Start => {
                spawn_start_platform(commands, asset_server, node, layout);
            }
            PlatformType::Goal => {
                spawn_goal_platform(commands, asset_server, node, layout);
            }
            _ => {
                // Check what terrain this platform contains
                let has_water_source = node
                    .terrain_objects
                    .iter()
                    .any(|t| matches!(t, SmartTerrain::WaterSource));

                let has_fire = node.terrain_objects.iter().any(|t| {
                    matches!(
                        t,
                        SmartTerrain::BlockingFire { .. } | SmartTerrain::SnowMeltFire { .. }
                    )
                });

                if has_water_source && !has_fire {
                    // Spawn specialized water platform (2 tiles high, with water integrated)
                    spawn_water_platform(commands, asset_server, node, layout);
                } else {
                    // Spawn standard grass platform
                    spawn_standard_platform(commands, node, layout);
                }

                // Spawn other terrain objects (fire, snow, containers, etc.)
                spawn_other_terrain_objects(commands, asset_server, node, layout);
            }
        }
    }
}

/// Helper: Spawns a platform with integrated water source
fn spawn_water_platform(
    commands: &mut Commands,
    asset_server: &AssetServer,
    node: &PlatformNode,
    layout: &PlatformLayout,
) {
    use super::graph::{GROUND_LEVEL, PlatformType};
    use rand::Rng;
    let mut rng = rand::rng();

    let platform_width_tiles = layout.width_tiles;
    let platform_x_tiles = layout.grid_x;
    let platform_y_tiles = layout.grid_y;

    // Generate water layout (random 1-4 tiles wide)
    let total_water_width = rng.random_range(1..=4);
    let water_start_x = platform_x_tiles + ((platform_width_tiles - total_water_width) / 2);

    // Spawn platform tiles (2 tiles high)
    for y_offset in 0..layout.height_tiles {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let tile_y = platform_y_tiles + y_offset;
            let grid_pos = GridPosition::primary(tile_x, tile_y);

            // Skip top layer where water will be
            if y_offset == layout.height_tiles - 1 {
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
    for x_offset in 0..=total_water_width + 2 {
        let z = if x_offset <= 0 || x_offset > total_water_width + 1 {
            -0.1
        } else {
            20.
        };

        let tile_x = water_start_x + x_offset;
        let water_grid_pos = GridPosition::primary(tile_x, platform_y_tiles + layout.height_tiles);

        spawn_water(
            commands,
            asset_server,
            water_grid_pos,
            WaterType::WaterMiddle,
            z,
        );
    }

    // If grounded, spawn dirt tiles extending down to GROUND_LEVEL
    if node.platform_type == PlatformType::Grounded {
        let ground_y_tiles = (GROUND_LEVEL / TILE_SIZE) as i32;
        let platform_bottom_y = platform_y_tiles;

        // Spawn dirt from bottom of platform down to ground level
        for y in ground_y_tiles..platform_bottom_y {
            for x_offset in 0..platform_width_tiles {
                let tile_x = platform_x_tiles + x_offset;
                let grid_pos = GridPosition::primary(tile_x, y);

                commands.spawn((
                    Name::new(format!("Ground support at ({}, {})", tile_x, y)),
                    grid_pos,
                    TerrainTile::Dirt,
                    DespawnOnExit(Screen::Gameplay),
                ));
            }
        }
    }
}

/// Helper: Spawns a standard grass platform
fn spawn_standard_platform(commands: &mut Commands, node: &PlatformNode, layout: &PlatformLayout) {
    use super::graph::{GROUND_LEVEL, PlatformType};

    let platform_width_tiles = layout.width_tiles;
    let platform_x_tiles = layout.grid_x;
    let platform_y_tiles = layout.grid_y;

    // Spawn platform tiles based on height
    for y_offset in 0..layout.height_tiles {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let tile_y = platform_y_tiles + y_offset;

            let grid_pos = GridPosition::primary(tile_x, tile_y);
            commands.spawn((
                Name::new(format!("Platform tile at ({}, {})", tile_x, tile_y)),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }

    // If grounded, spawn dirt tiles extending down to GROUND_LEVEL
    if node.platform_type == PlatformType::Grounded {
        let ground_y_tiles = (GROUND_LEVEL / TILE_SIZE) as i32;
        let platform_bottom_y = platform_y_tiles;

        // Spawn dirt from bottom of platform down to ground level
        for y in ground_y_tiles..platform_bottom_y {
            for x_offset in 0..platform_width_tiles {
                let tile_x = platform_x_tiles + x_offset;
                let grid_pos = GridPosition::primary(tile_x, y);

                commands.spawn((
                    Name::new(format!("Ground support at ({}, {})", tile_x, y)),
                    grid_pos,
                    TerrainTile::Dirt,
                    DespawnOnExit(Screen::Gameplay),
                ));
            }
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
        let center_world = layout.center_world();
        let world_pos = Vec3::new(center_world.x, layout.top_world() + TILE_SIZE, 0.0);

        match terrain {
            SmartTerrain::WaterSource => {
                // Already handled in spawn_water_platform
                continue;
            }
            SmartTerrain::SnowSource => {
                super::objects::spawn_snow(commands, asset_server, world_pos);
            }
            SmartTerrain::BlockingFire { extinguished } => {
                let fire_state = if *extinguished {
                    FireState::Extinguished
                } else {
                    FireState::Active
                };
                spawn_fire(commands, asset_server, world_pos, fire_state);

                // Spawn blocking wall above fire (only if not extinguished)
                if !extinguished {
                    spawn_fire_wall(commands, world_pos);
                }
            }
            SmartTerrain::SnowMeltFire { extinguished } => {
                let fire_state = if *extinguished {
                    FireState::Extinguished
                } else {
                    FireState::Active
                };
                spawn_fire(commands, asset_server, world_pos, fire_state);
            }
            SmartTerrain::GoalContainer { .. } => {
                let container_pos = Vec3::new(center_world.x, layout.top_world() + TILE_SIZE, 0.0);
                spawn_container(commands, asset_server, container_pos, ContainerState::Empty);
            }
            SmartTerrain::SwitchContainer { .. } => {
                // For now, spawn as a regular container
                // TODO: Differentiate visually or add switch logic
                let container_pos = Vec3::new(center_world.x, layout.top_world() + TILE_SIZE, 0.0);
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

/// Helper: Spawns a start platform (grounded with left wall and water source)
fn spawn_start_platform(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _node: &PlatformNode,
    layout: &PlatformLayout,
) {
    use super::graph::{GROUND_LEVEL, WALL_HEIGHT};
    use rand::Rng;
    let mut rng = rand::rng();

    let platform_width_tiles = layout.width_tiles;
    let platform_x_tiles = layout.grid_x;
    let platform_y_tiles = layout.grid_y;

    // Generate water layout: 2-3 tiles wide, starting right next to the left wall
    let water_width = rng.random_range(2..=3);
    let water_start_x = platform_x_tiles + 2; // Start right at the platform edge (next to wall)

    // Spawn platform tiles (2 tiles high)
    for y_offset in 0..layout.height_tiles {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let tile_y = platform_y_tiles + y_offset;
            let grid_pos = GridPosition::primary(tile_x, tile_y);

            // Skip top layer where water will be
            if y_offset == layout.height_tiles - 1
                && tile_x >= water_start_x
                && tile_x < water_start_x + water_width - 1
            {
                continue;
            }

            let terrain = if y_offset == layout.height_tiles - 1 {
                TerrainTile::Grass
            } else {
                TerrainTile::Dirt
            };
            commands.spawn((
                Name::new(format!("Start platform tile at ({}, {})", tile_x, tile_y)),
                grid_pos,
                terrain,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }

    // Spawn water tiles (extend right from the wall)
    let water_y = platform_y_tiles + layout.height_tiles;
    for x_offset in 0..=water_width {
        let z = if x_offset <= 0 || x_offset > water_width - 1 {
            -0.1
        } else {
            20.
        };
        let tile_x = water_start_x + x_offset;
        let water_grid_pos = GridPosition::primary(tile_x, water_y);

        // Leftmost water tile is the waterfall base
        let water_type = if x_offset == 1 {
            WaterType::WaterfallBase
        } else {
            WaterType::WaterMiddle
        };

        spawn_water(commands, asset_server, water_grid_pos, water_type, z);
    }

    // Spawn waterfall tiles above the leftmost water tile
    let waterfall_x = water_start_x + 1;
    let waterfall_middle_count = rng.random_range(1..=2);

    // WaterfallLower (1 tile above base)
    spawn_water(
        commands,
        asset_server,
        GridPosition::primary(waterfall_x, water_y + 1),
        WaterType::WaterfallLower,
        20.0,
    );

    // WaterfallMiddle (1-2 tiles)
    for i in 0..waterfall_middle_count {
        spawn_water(
            commands,
            asset_server,
            GridPosition::primary(waterfall_x, water_y + 2 + i),
            WaterType::WaterfallMiddle,
            20.0,
        );
    }

    // WaterfallTop (at the top)
    spawn_water(
        commands,
        asset_server,
        GridPosition::primary(waterfall_x, water_y + 2 + waterfall_middle_count),
        WaterType::WaterfallTop,
        20.0,
    );

    // Spawn ground support (dirt to GROUND_LEVEL)
    let ground_y_tiles = (GROUND_LEVEL / TILE_SIZE) as i32;
    let platform_bottom_y = platform_y_tiles;
    for y in ground_y_tiles..platform_bottom_y {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let grid_pos = GridPosition::primary(tile_x, y);
            commands.spawn((
                Name::new(format!("Start ground support at ({}, {})", tile_x, y)),
                grid_pos,
                TerrainTile::Dirt,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }

    // Spawn left wall (5 tiles high, 3 tiles wide from platform top)
    for wall_y_offset in -10..=WALL_HEIGHT {
        for wall_x_offset in -10..2 {
            let tile_x = platform_x_tiles + wall_x_offset;
            let tile_y = platform_y_tiles + wall_y_offset;
            let grid_pos = GridPosition::primary(tile_x, tile_y);
            commands.spawn((
                Name::new(format!("Start left wall at ({}, {})", tile_x, tile_y)),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }
}

/// Helper: Spawns a goal platform (grounded with right wall and goal container)
fn spawn_goal_platform(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _node: &PlatformNode,
    layout: &PlatformLayout,
) {
    use super::graph::{GROUND_LEVEL, WALL_HEIGHT};

    let platform_width_tiles = layout.width_tiles;
    let platform_x_tiles = layout.grid_x;
    let platform_y_tiles = layout.grid_y;

    // Spawn platform tiles (2 tiles high)
    for y_offset in 0..layout.height_tiles {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let tile_y = platform_y_tiles + y_offset;
            let grid_pos = GridPosition::primary(tile_x, tile_y);

            let terrain = if y_offset == layout.height_tiles - 1 {
                TerrainTile::Grass
            } else {
                TerrainTile::Dirt
            };
            commands.spawn((
                Name::new(format!("Goal platform tile at ({}, {})", tile_x, tile_y)),
                grid_pos,
                terrain,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }

    // Spawn goal container in the middle
    let center_world = layout.center_world();
    let container_pos = Vec3::new(center_world.x, layout.top_world() + TILE_SIZE, 0.0);
    spawn_container(commands, asset_server, container_pos, ContainerState::Empty);

    // Spawn ground support (dirt to GROUND_LEVEL)
    let ground_y_tiles = (GROUND_LEVEL / TILE_SIZE) as i32;
    let platform_bottom_y = platform_y_tiles;
    for y in ground_y_tiles..platform_bottom_y {
        for x_offset in 0..platform_width_tiles {
            let tile_x = platform_x_tiles + x_offset;
            let grid_pos = GridPosition::primary(tile_x, y);
            commands.spawn((
                Name::new(format!("Goal ground support at ({}, {})", tile_x, y)),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }

    // Spawn right wall
    let right_edge_start = platform_x_tiles + platform_width_tiles - 1;
    for wall_y_offset in -10..=WALL_HEIGHT {
        for wall_x_offset in 0..10 {
            let tile_x = right_edge_start + wall_x_offset;
            let tile_y = platform_y_tiles + wall_y_offset;
            let grid_pos = GridPosition::primary(tile_x, tile_y);
            commands.spawn((
                Name::new(format!("Goal right wall at ({}, {})", tile_x, tile_y)),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }
}

/// Spawns a 3x5 blocking wall above a fire
fn spawn_fire_wall(commands: &mut Commands, fire_world_pos: Vec3) {
    // Convert fire world position to grid coordinates
    let fire_grid_x = (fire_world_pos.x / TILE_SIZE) as i32;
    let fire_grid_y = (fire_world_pos.y / TILE_SIZE) as i32;

    // Spawn 3-tile wide, 5-tile high wall above the fire
    for y_offset in 1..=5 {
        for x_offset in -1..=1 {
            let grid_pos = GridPosition::primary(fire_grid_x + x_offset, fire_grid_y + y_offset);
            commands.spawn((
                Name::new(format!("Fire wall at ({}, {})", grid_pos.x, grid_pos.y)),
                grid_pos,
                TerrainTile::Grass,
                DespawnOnExit(Screen::Gameplay),
            ));
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
        // Position player 6 tiles to the right of the left edge of the platform
        let spawn_x = (start_layout.grid_x + 6) as f32 * TILE_SIZE;
        spawn_point.position = Vec3::new(spawn_x, start_layout.top_world() + TILE_SIZE * 4.0, 0.0);
    }
}
