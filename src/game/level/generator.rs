#![allow(dead_code)]

use super::causality::{BucketContent, CausalityChain, CausalityNode, Cause, Effect};
use super::graph::{NodeId, PlatformGraph, SmartTerrain};
use bevy::prelude::*;
use rand::Rng;

/// Configuration for level generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Target difficulty (affects chain length)
    pub difficulty: Difficulty,
    /// Random seed for reproducibility
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Returns the target chain length range for this difficulty
    pub fn chain_length_range(&self) -> (usize, usize) {
        match self {
            Difficulty::Easy => (2, 4),
            Difficulty::Medium => (4, 7),
            Difficulty::Hard => (7, 12),
        }
    }

    /// Returns the maximum number of fires for this difficulty
    pub fn max_fires(&self) -> usize {
        match self {
            Difficulty::Easy => 1,
            Difficulty::Medium => 2,
            Difficulty::Hard => 4,
        }
    }
}

/// Generator for creating causality chains
pub struct CausalityGenerator {
    config: GeneratorConfig,
    rng: rand::rngs::StdRng,
}

impl CausalityGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        use rand::SeedableRng;
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(config.seed),
            config,
        }
    }

    /// Generates a causality chain for filling a container twice
    pub fn generate_chain(&mut self, graph: &PlatformGraph) -> Result<CausalityChain, String> {
        let mut chain = CausalityChain::new(Effect::ContainerFilled(1)); // Second fill

        // Goal: Fill container twice
        // We'll work backwards from the second fill to the first fill to the start

        let goal_node = graph.goal;

        // Step 1: Second fill of the container
        chain.add_node(CausalityNode {
            effect: Effect::ContainerFilled(1),
            cause: Cause::BucketAt {
                content: BucketContent::Water,
                location: goal_node,
            },
            terrain: SmartTerrain::GoalContainer {
                fill_count: 0,
                target: 2,
            },
            location: goal_node,
        });

        // Step 2: Get water for second fill
        self.add_water_source_step(&mut chain, graph)?;

        // Step 3: First fill of the container
        chain.add_node(CausalityNode {
            effect: Effect::ContainerFilled(0),
            cause: Cause::BucketAt {
                content: BucketContent::Water,
                location: goal_node,
            },
            terrain: SmartTerrain::GoalContainer {
                fill_count: 0,
                target: 2,
            },
            location: goal_node,
        });

        // Step 4: Get water for first fill
        self.add_water_source_step(&mut chain, graph)?;

        Ok(chain)
    }

    /// Adds a step to get water (either from a water source or snow + fire)
    fn add_water_source_step(
        &mut self,
        chain: &mut CausalityChain,
        graph: &PlatformGraph,
    ) -> Result<(), String> {
        // Choose between direct water source or snow + fire conversion
        let use_fire = self.rng.random_bool(0.3); // 30% chance to use fire mechanic

        if use_fire && self.config.difficulty != Difficulty::Easy {
            // Add fire + snow conversion
            self.add_fire_conversion_step(chain, graph)?;
        } else {
            // Add simple water source
            self.add_simple_water_source(chain, graph)?;
        }

        Ok(())
    }

    /// Adds a simple water source (player picks up water directly)
    fn add_simple_water_source(
        &mut self,
        chain: &mut CausalityChain,
        graph: &PlatformGraph,
    ) -> Result<(), String> {
        // Find a node that's not the goal to place the water source
        let available_nodes: Vec<NodeId> = graph
            .reachable_from(graph.start)
            .into_iter()
            .filter(|&n| n != graph.goal)
            .collect();

        if available_nodes.is_empty() {
            return Err("No available nodes for water source".to_string());
        }

        let node = available_nodes[self.rng.random_range(0..available_nodes.len())];

        chain.add_node(CausalityNode {
            effect: Effect::WaterBucket,
            cause: Cause::Player,
            terrain: SmartTerrain::WaterSource { active: true },
            location: node,
        });

        Ok(())
    }

    /// Adds a fire conversion step (snow + fire = water)
    fn add_fire_conversion_step(
        &mut self,
        chain: &mut CausalityChain,
        graph: &PlatformGraph,
    ) -> Result<(), String> {
        let available_nodes: Vec<NodeId> = graph
            .reachable_from(graph.start)
            .into_iter()
            .filter(|&n| n != graph.goal)
            .collect();

        if available_nodes.len() < 2 {
            return Err("Not enough nodes for fire conversion".to_string());
        }

        // Pick two different nodes for snow source and fire
        let snow_node = available_nodes[self.rng.random_range(0..available_nodes.len())];
        let mut fire_node = available_nodes[self.rng.random_range(0..available_nodes.len())];

        // Ensure fire is not at the same location as snow
        while fire_node == snow_node && available_nodes.len() > 1 {
            fire_node = available_nodes[self.rng.random_range(0..available_nodes.len())];
        }

        // Step 1: Get snow
        chain.add_node(CausalityNode {
            effect: Effect::SnowBucket,
            cause: Cause::Player,
            terrain: SmartTerrain::SnowSource,
            location: snow_node,
        });

        // Step 2: Convert snow to water at fire
        chain.add_node(CausalityNode {
            effect: Effect::WaterBucket,
            cause: Cause::BucketAt {
                content: BucketContent::Snow,
                location: fire_node,
            },
            terrain: SmartTerrain::Fire {
                extinguished: false,
            },
            location: fire_node,
        });

        Ok(())
    }

    /// Applies the causality chain to the platform graph by placing terrain objects
    pub fn apply_chain_to_graph(
        &self,
        chain: &CausalityChain,
        graph: &mut PlatformGraph,
    ) -> Result<(), String> {
        use crate::game::tiles::TILE_SIZE;

        let terrain_map = chain.terrain_by_location();

        for (node_id, terrain_objects) in terrain_map {
            let node = graph
                .get_node_mut(node_id)
                .ok_or_else(|| format!("Node {:?} not found in graph", node_id))?;

            // Check if this node will have a water source
            let has_water_source = terrain_objects
                .iter()
                .any(|t| matches!(t, SmartTerrain::WaterSource { .. }));

            if has_water_source {
                // Ensure platform is 2 tiles high
                node.height = 2;

                // Ensure platform is at least 4 tiles wider than max water width (5 tiles)
                // Water can be 3-5 tiles wide, so we need at least 9 tiles (5 + 4 margin)
                let min_width_tiles = 9.0;
                let min_width = min_width_tiles * TILE_SIZE;
                if node.width < min_width {
                    node.width = min_width;
                }
            }

            for terrain in terrain_objects {
                // Avoid adding duplicate goal containers
                if matches!(terrain, SmartTerrain::GoalContainer { .. }) {
                    if !node
                        .terrain_objects
                        .iter()
                        .any(|t| matches!(t, SmartTerrain::GoalContainer { .. }))
                    {
                        node.add_terrain(terrain);
                    }
                } else {
                    node.add_terrain(terrain);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::level::graph::{ConnectionType, PlatformNode};

    fn create_simple_graph() -> PlatformGraph {
        let mut graph = PlatformGraph::new(NodeId(0), NodeId(2));

        let node1 = PlatformNode::new(Vec2::new(0.0, 0.0), 100.0);
        let node2 = PlatformNode::new(Vec2::new(200.0, 0.0), 100.0);
        let node3 = PlatformNode::new(Vec2::new(400.0, 0.0), 100.0);

        let id1 = graph.add_node(node1);
        let id2 = graph.add_node(node2);
        let id3 = graph.add_node(node3);

        graph
            .get_node_mut(id1)
            .unwrap()
            .add_edge(id2, ConnectionType::Jump);
        graph
            .get_node_mut(id2)
            .unwrap()
            .add_edge(id3, ConnectionType::Jump);

        graph
    }

    #[test]
    fn test_generate_simple_chain() {
        let graph = create_simple_graph();
        let config = GeneratorConfig {
            difficulty: Difficulty::Easy,
            seed: 42,
        };

        let mut generator = CausalityGenerator::new(config);
        let chain = generator.generate_chain(&graph);

        assert!(chain.is_ok());
        let chain = chain.unwrap();
        assert!(chain.validate().is_ok());
        println!("Generated chain with {} nodes", chain.nodes.len());
    }

    #[test]
    fn test_apply_chain_to_graph() {
        let mut graph = create_simple_graph();
        let config = GeneratorConfig {
            difficulty: Difficulty::Easy,
            seed: 42,
        };

        let mut generator = CausalityGenerator::new(config);
        let chain = generator.generate_chain(&graph).unwrap();

        let result = generator.apply_chain_to_graph(&chain, &mut graph);
        assert!(result.is_ok());

        // Check that terrain was placed
        let has_terrain = graph.nodes.iter().any(|n| !n.terrain_objects.is_empty());
        assert!(has_terrain, "No terrain objects were placed");
    }
}
