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

    // Try to generate a valid level, with up to 10 retries
    let max_retries = 10;
    for attempt in 0..max_retries {
        let attempt_seed = seed.wrapping_add(attempt as u64 * 10000);

        // Generate graph(s) based on difficulty
        // Easy: Single graph (varied templates except first level)
        // Medium: Merge 2 graphs
        // Hard: Merge 3 graphs
        let mut graph = match difficulty {
            Difficulty::Easy => {
                // First level (Summer level 1) always uses linear template
                // All other easy levels use varied templates
                let is_first_level = *season == Season::Summer && game_level.0 == 1;

                if is_first_level {
                    create_linear_template(Some(attempt_seed))
                } else {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand::rngs::StdRng::seed_from_u64(attempt_seed);

                    let templates: [fn(u64) -> PlatformGraph; 3] = [
                        |s| create_linear_template(Some(s)),
                        |_| create_zigzag_template(),
                        |_| create_ground_and_floating_template(),
                    ];

                    templates[rng.random_range(0..templates.len())](attempt_seed)
                }
            }
            Difficulty::Medium => {
                // Merge 2 template graphs
                use rand::{Rng, SeedableRng};
                let mut rng = rand::rngs::StdRng::seed_from_u64(attempt_seed);

                let templates: [fn(u64) -> PlatformGraph; 5] = [
                    |s| create_linear_template(Some(s)),
                    |_| create_branching_template(),
                    |_| create_cul_de_sac_template(),
                    |_| create_zigzag_template(),
                    |_| create_ground_and_floating_template(),
                ];

                let graph1 =
                    templates[rng.random_range(0..templates.len())](attempt_seed.wrapping_add(1));
                let graph2 =
                    templates[rng.random_range(0..templates.len())](attempt_seed.wrapping_add(2));

                merge_graphs(vec![graph1, graph2])
            }
            Difficulty::Hard => {
                // Merge 3 template graphs
                use rand::{Rng, SeedableRng};
                let mut rng = rand::rngs::StdRng::seed_from_u64(attempt_seed);

                let templates: [fn(u64) -> PlatformGraph; 5] = [
                    |s| create_linear_template(Some(s)),
                    |_| create_branching_template(),
                    |_| create_cul_de_sac_template(),
                    |_| create_zigzag_template(),
                    |_| create_ground_and_floating_template(),
                ];

                let graph1 =
                    templates[rng.random_range(0..templates.len())](attempt_seed.wrapping_add(1));
                let graph2 =
                    templates[rng.random_range(0..templates.len())](attempt_seed.wrapping_add(2));
                let graph3 =
                    templates[rng.random_range(0..templates.len())](attempt_seed.wrapping_add(3));

                merge_graphs(vec![graph1, graph2, graph3])
            }
        };

        let config = GeneratorConfig {
            difficulty,
            seed: attempt_seed,
            season: *season,
            completed_year: completed_year.0,
        };

        let mut generator = CausalityGenerator::new(config);

        if let Ok(chain) = generator.generate_chain(&graph)
            && chain.validate().is_ok()
            && generator.apply_chain_to_graph(&chain, &mut graph).is_ok()
        {
            // Generate concrete layout from the abstract graph (Phase 2: Layout generation)
            let layouts = graph.generate_layout(attempt_seed);

            // Update spawn point based on generated level
            update_player_spawn_point(&graph, &layouts, &mut spawn_point);

            // Spawn entire level in one pass
            spawn_level_from_graph(&mut commands, &asset_server, &graph, &layouts);

            info!(
                "Successfully generated procedural level (attempt {})",
                attempt + 1
            );
            return; // Success! Exit the system
        }

        warn!(
            "Level generation attempt {} failed, retrying...",
            attempt + 1
        );
    }

    // If we get here, all retries failed - this should be very rare
    error!(
        "Failed to generate level after {} attempts! Using fallback linear template",
        max_retries
    );

    // Last resort: use basic linear template without any smart terrain
    let graph = create_linear_template(Some(seed));
    let layouts = graph.generate_layout(seed);
    update_player_spawn_point(&graph, &layouts, &mut spawn_point);
    spawn_level_from_graph(&mut commands, &asset_server, &graph, &layouts);
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
