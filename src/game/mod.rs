use crate::{PausableSystems, screens::Screen};
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
use interactions::LevelCompleteMessage;
use parallax::{parallax_background, scroll_parallax};

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
pub struct GameLevel(pub u8);

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
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
    app.add_systems(
        Update,
        (scroll_parallax, handle_level_complete)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_plugins(tiles::plugin);
    app.add_plugins(character::plugin);
    app.add_plugins(controls::plugin);
    app.add_plugins(physics::plugin);
    app.add_plugins(level::plugin);
    app.add_plugins(ui::plugin);
    app.add_plugins(interactions::plugin);
}

pub fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    season: Res<Season>,
    game_level: Res<GameLevel>,
    mut spawn_point: ResMut<level::PlayerSpawnPoint>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![parallax_background(*season, asset_server.clone())],
    ));

    // Calculate platform count based on level (start with 5, increase gradually)
    let platform_count = (5 + (game_level.0 / 2) as usize).min(10);

    // Use season and level to create unique seed
    let seed = (*season as u64) * 1000 + game_level.0 as u64;

    // Generate a randomized procedural level
    let mut graph = level::create_random_linear_segment(platform_count, seed);

    let config = level::GeneratorConfig {
        difficulty: level::Difficulty::Easy,
        seed,
    };

    let mut generator = level::CausalityGenerator::new(config);

    if let Ok(chain) = generator.generate_chain(&graph)
        && chain.validate().is_ok()
    {
        if generator.apply_chain_to_graph(&chain, &mut graph).is_ok() {
            // Update spawn point based on generated level
            level::update_player_spawn_point(&graph, &mut spawn_point);

            // Spawn terrain objects first to get exclusion map
            let exclusions =
                level::spawn_terrain_objects_from_graph(&mut commands, &asset_server, &graph);

            // Then spawn platforms, excluding positions with water/dirt
            level::spawn_platforms_from_graph(&mut commands, &graph, &exclusions);

            info!("Successfully generated procedural level");
        }
    } else {
        warn!("Failed to generate procedural level, falling back to flat level");
        level::spawn_flat_level(&mut commands, 40, -2, 6);
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
