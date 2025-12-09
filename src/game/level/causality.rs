#![allow(dead_code)]

use super::graph::{NodeId, SmartTerrain};
use bevy::prelude::*;

/// Represents the state of the player's bucket
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub enum BucketContent {
    #[default]
    Empty,
    Water,
    Snow,
}

/// Effects that can be achieved in the level
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Player has water in bucket
    WaterBucket,
    /// Player has snow in bucket
    SnowBucket,
    /// Player has empty bucket
    EmptyBucket,
    /// A specific container has been filled
    ContainerFilled(usize), // index into which fill
    /// A fire has been extinguished
    FireExtinguished(NodeId),
    /// A switch has been activated
    SwitchActivated(NodeId),
    /// A path is now accessible (moving platform activated)
    PathAccessible(NodeId),
}

/// Causes that can trigger effects
#[derive(Debug, Clone, PartialEq)]
pub enum Cause {
    /// Player can do this directly (e.g., pick up water)
    Player,
    /// Requires specific bucket content at a location
    BucketAt {
        content: BucketContent,
        location: NodeId,
    },
    /// Requires one of several effects to be achieved first
    RequiresAny(Vec<Effect>),
    /// Requires all of several effects to be achieved first
    RequiresAll(Vec<Effect>),
}

/// A node in the causality chain representing a puzzle step
#[derive(Debug, Clone)]
pub struct CausalityNode {
    /// What this step achieves
    pub effect: Effect,
    /// What's needed to achieve this effect
    pub cause: Cause,
    /// The terrain object that enables this
    pub terrain: SmartTerrain,
    /// Where this terrain should be placed
    pub location: NodeId,
}

/// A complete causality chain representing a puzzle solution
#[derive(Debug, Clone)]
pub struct CausalityChain {
    /// The nodes in the chain, in order from goal to start
    pub nodes: Vec<CausalityNode>,
    /// The final goal this chain achieves
    pub goal: Effect,
}

impl CausalityChain {
    pub fn new(goal: Effect) -> Self {
        Self {
            nodes: Vec::new(),
            goal,
        }
    }

    pub fn add_node(&mut self, node: CausalityNode) {
        self.nodes.push(node);
    }

    /// Returns the chain in reverse order (start to goal)
    pub fn forward_order(&self) -> impl Iterator<Item = &CausalityNode> {
        self.nodes.iter().rev()
    }

    /// Validates that the chain is complete (all causes are satisfied)
    pub fn validate(&self) -> Result<(), String> {
        // Track which effects have been achieved
        let mut achieved: std::collections::HashSet<Effect> = std::collections::HashSet::new();

        // Process in forward order (start to goal)
        for node in self.forward_order() {
            // Check if the cause is satisfied
            match &node.cause {
                Cause::Player => {
                    // Always satisfied
                }
                Cause::BucketAt { .. } => {
                    // For now, assume bucket requirements are satisfiable
                    // In a full implementation, we'd track bucket state through the chain
                }
                Cause::RequiresAny(effects) => {
                    if !effects.iter().any(|e| achieved.contains(e)) {
                        return Err(format!(
                            "Effect {:?} requires one of {:?}, but none are achieved",
                            node.effect, effects
                        ));
                    }
                }
                Cause::RequiresAll(effects) => {
                    for effect in effects {
                        if !achieved.contains(effect) {
                            return Err(format!(
                                "Effect {:?} requires {:?}, which is not achieved",
                                node.effect, effect
                            ));
                        }
                    }
                }
            }

            // Mark this effect as achieved
            achieved.insert(node.effect.clone());
        }

        // Check that the goal is achieved
        if !achieved.contains(&self.goal) {
            return Err(format!("Goal {:?} is not achieved by the chain", self.goal));
        }

        Ok(())
    }

    /// Returns all terrain objects that need to be placed, grouped by location
    pub fn terrain_by_location(&self) -> std::collections::HashMap<NodeId, Vec<SmartTerrain>> {
        let mut terrain_map = std::collections::HashMap::new();

        for node in &self.nodes {
            terrain_map
                .entry(node.location)
                .or_insert_with(Vec::new)
                .push(node.terrain.clone());
        }

        terrain_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::level::graph::SmartTerrain;

    #[test]
    fn test_simple_chain() {
        let mut chain = CausalityChain::new(Effect::ContainerFilled(0));

        // Player gets water
        chain.add_node(CausalityNode {
            effect: Effect::WaterBucket,
            cause: Cause::Player,
            terrain: SmartTerrain::WaterSource { active: true },
            location: NodeId(0),
        });

        // Player fills container
        chain.add_node(CausalityNode {
            effect: Effect::ContainerFilled(0),
            cause: Cause::BucketAt {
                content: BucketContent::Water,
                location: NodeId(1),
            },
            terrain: SmartTerrain::GoalContainer {
                fill_count: 0,
                target: 2,
            },
            location: NodeId(1),
        });

        assert!(chain.validate().is_ok());
    }

    #[test]
    fn test_chain_with_fire() {
        let mut chain = CausalityChain::new(Effect::ContainerFilled(0));

        // Player gets snow
        chain.add_node(CausalityNode {
            effect: Effect::SnowBucket,
            cause: Cause::Player,
            terrain: SmartTerrain::SnowSource,
            location: NodeId(0),
        });

        // Snow + Fire = Water (fire extinguished)
        chain.add_node(CausalityNode {
            effect: Effect::WaterBucket,
            cause: Cause::BucketAt {
                content: BucketContent::Snow,
                location: NodeId(1),
            },
            terrain: SmartTerrain::Fire {
                extinguished: false,
            },
            location: NodeId(1),
        });

        // Fill container with water
        chain.add_node(CausalityNode {
            effect: Effect::ContainerFilled(0),
            cause: Cause::BucketAt {
                content: BucketContent::Water,
                location: NodeId(2),
            },
            terrain: SmartTerrain::GoalContainer {
                fill_count: 0,
                target: 2,
            },
            location: NodeId(2),
        });

        assert!(chain.validate().is_ok());
    }
}
