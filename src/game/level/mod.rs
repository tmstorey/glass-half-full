#![allow(unused_imports)]

use bevy::prelude::*;

use super::interactions::LevelCompleteMessage;
use super::parallax::{parallax_background, scroll_parallax};
use super::tiles::{GridPosition, TerrainTile};
use crate::{
    PausableSystems,
    game::{CompletedYear, GameLevel, Season},
    screens::Screen,
};

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
pub use graph::{
    ConnectionType, Edge, GROUND_LEVEL, NodeId, PlatformGraph, PlatformLayout, PlatformNode,
    PlatformType, SmartTerrain, WALL_HEIGHT,
};
pub use spawn::{PlayerSpawnPoint, spawn_level_from_graph, update_player_spawn_point};
pub use templates::{
    create_branching_template, create_cul_de_sac_template, create_ground_and_floating_template,
    create_linear_template, create_zigzag_template, merge_graphs,
};

pub fn plugin(app: &mut App) {
    app.init_resource::<PlayerSpawnPoint>();
    app.init_resource::<BucketContent>();
    app.add_plugins(objects::plugin);
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
    app.add_systems(
        Update,
        (scroll_parallax, handle_level_complete)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

pub fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    season: Res<Season>,
    completed_year: Res<CompletedYear>,
    game_level: Res<GameLevel>,
    mut spawn_point: ResMut<PlayerSpawnPoint>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![parallax_background(*season, asset_server.clone())],
    ));

    // Use season and level to create unique seed
    // If year is completed, add randomness for variety on subsequent playthroughs
    let seed = if completed_year.0 {
        use rand::Rng;
        let base_seed = (*season as u64) * 1000 + game_level.0 as u64;
        let random_offset = rand::rng().random_range(0..1000000);
        base_seed.wrapping_add(random_offset)
    } else {
        (*season as u64) * 1000 + game_level.0 as u64
    };

    // Calculate difficulty based on season and completion
    let difficulty = if completed_year.0 {
        // After completing a year, stay on Hard
        Difficulty::Hard
    } else {
        match *season {
            Season::Summer => Difficulty::Easy,
            Season::Autumn => Difficulty::Medium,
            Season::Winter => Difficulty::Medium,
            Season::Spring => Difficulty::Hard,
        }
    };

    // Generate graph(s) based on difficulty
    // Easy: Single graph
    // Medium: Merge 2 graphs
    // Hard: Merge 3 graphs
    let mut graph = match difficulty {
        Difficulty::Easy => {
            // Single template graph with randomization
            create_linear_template(Some(seed))
        }
        Difficulty::Medium => {
            // Merge 2 template graphs
            use rand::{Rng, SeedableRng};
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

            let templates: [fn(u64) -> PlatformGraph; 5] = [
                |s| create_linear_template(Some(s)),
                |_| create_branching_template(),
                |_| create_cul_de_sac_template(),
                |_| create_zigzag_template(),
                |_| create_ground_and_floating_template(),
            ];

            let graph1 = templates[rng.random_range(0..templates.len())](seed.wrapping_add(1));
            let graph2 = templates[rng.random_range(0..templates.len())](seed.wrapping_add(2));

            merge_graphs(vec![graph1, graph2])
        }
        Difficulty::Hard => {
            // Merge 3 template graphs
            use rand::{Rng, SeedableRng};
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

            let templates: [fn(u64) -> PlatformGraph; 5] = [
                |s| create_linear_template(Some(s)),
                |_| create_branching_template(),
                |_| create_cul_de_sac_template(),
                |_| create_zigzag_template(),
                |_| create_ground_and_floating_template(),
            ];

            let graph1 = templates[rng.random_range(0..templates.len())](seed.wrapping_add(1));
            let graph2 = templates[rng.random_range(0..templates.len())](seed.wrapping_add(2));
            let graph3 = templates[rng.random_range(0..templates.len())](seed.wrapping_add(3));

            merge_graphs(vec![graph1, graph2, graph3])
        }
    };

    let config = GeneratorConfig {
        difficulty,
        seed,
        season: *season,
        completed_year: completed_year.0,
    };

    let mut generator = CausalityGenerator::new(config);

    if let Ok(chain) = generator.generate_chain(&graph)
        && chain.validate().is_ok()
    {
        if generator.apply_chain_to_graph(&chain, &mut graph).is_ok() {
            // Generate concrete layout from the abstract graph (Phase 2: Layout generation)
            let layouts = graph.generate_layout(seed);

            // Update spawn point based on generated level
            update_player_spawn_point(&graph, &layouts, &mut spawn_point);

            // Spawn entire level in one pass
            spawn_level_from_graph(&mut commands, &asset_server, &graph, &layouts);

            info!("Successfully generated procedural level");
        }
    } else {
        warn!("Failed to generate procedural level, falling back to flat level");
        spawn_flat_level(&mut commands, 40, -2, 6);
    }
}

/// System to handle level completion and transition to victory screen
fn handle_level_complete(
    mut level_complete_reader: MessageReader<LevelCompleteMessage>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    for _ in level_complete_reader.read() {
        info!("Level complete! Showing victory screen");

        // Transition to Victory screen
        // The victory screen will handle incrementing the level and transitioning to Gameplay
        next_state.set(Screen::Victory);
    }
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
