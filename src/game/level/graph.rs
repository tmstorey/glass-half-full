#![allow(dead_code)]

use bevy::prelude::*;

/// Unique identifier for a platform node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Represents the type of connection between platforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    /// A jump that's always possible
    Jump,
    /// A one-way fall (can go down but not up)
    Fall,
    /// Requires a moving platform to be active
    MovingPlatform {
        platform_entity: Option<Entity>,
        required: bool,
    },
}

/// An edge connecting two platform nodes
#[derive(Debug, Clone)]
pub struct Edge {
    pub to: NodeId,
    pub connection_type: ConnectionType,
}

/// A node in the platform graph representing a platform section
#[derive(Debug, Clone)]
pub struct PlatformNode {
    /// Connections to other platforms
    pub edges: Vec<Edge>,
    /// Smart terrain objects placed on this platform
    pub terrain_objects: Vec<SmartTerrain>,
}

impl PlatformNode {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            terrain_objects: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, to: NodeId, connection_type: ConnectionType) {
        self.edges.push(Edge {
            to,
            connection_type,
        });
    }

    pub fn add_terrain(&mut self, terrain: SmartTerrain) {
        self.terrain_objects.push(terrain);
    }

    /// Calculates the width this platform should have based on its terrain
    pub fn calculate_width(&self) -> f32 {
        use crate::game::tiles::TILE_SIZE;

        if self
            .terrain_objects
            .iter()
            .any(|t| matches!(t, SmartTerrain::WaterSource { .. }))
        {
            TILE_SIZE * 9.0 // Water platforms need 9+ tiles
        } else {
            TILE_SIZE * 4.0 // Standard platform width
        }
    }

    /// Calculates the height this platform should have based on its terrain
    pub fn calculate_height(&self) -> u32 {
        if self
            .terrain_objects
            .iter()
            .any(|t| matches!(t, SmartTerrain::WaterSource { .. }))
        {
            2 // Water platforms are 2 tiles high
        } else {
            1 // Standard platforms are 1 tile high
        }
    }
}

/// Concrete layout information for a platform
#[derive(Debug, Clone, Copy)]
pub struct PlatformLayout {
    pub position: Vec2,
    pub width: f32,
    pub height: u32,
}

/// Smart terrain types that can be placed in the level
#[derive(Debug, Clone, PartialEq)]
pub enum SmartTerrain {
    /// Infinite water source
    WaterSource { active: bool },
    /// Infinite snow source
    SnowSource,
    /// Fire that can be extinguished
    Fire { extinguished: bool },
    /// The goal container that needs to be filled twice
    GoalContainer { fill_count: u8, target: u8 },
    /// A container that activates something when filled
    SwitchContainer { filled: bool, activates: NodeId },
    /// A switch that activates something when pressed
    Switch { activated: bool, activates: NodeId },
    /// A moving platform
    MovingPlatform { active: bool },
}

/// The complete platform graph for a level
#[derive(Debug)]
pub struct PlatformGraph {
    pub nodes: Vec<PlatformNode>,
    pub start: NodeId,
    pub goal: NodeId,
}

impl PlatformGraph {
    pub fn new(start: NodeId, goal: NodeId) -> Self {
        Self {
            nodes: Vec::new(),
            start,
            goal,
        }
    }

    pub fn add_node(&mut self, node: PlatformNode) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&PlatformNode> {
        self.nodes.get(id.0)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut PlatformNode> {
        self.nodes.get_mut(id.0)
    }

    /// Returns all nodes reachable from the given node
    pub fn reachable_from(&self, node: NodeId) -> Vec<NodeId> {
        let mut reachable = Vec::new();
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            if visited[current.0] {
                continue;
            }
            visited[current.0] = true;
            reachable.push(current);

            if let Some(node) = self.get_node(current) {
                for edge in &node.edges {
                    if !visited[edge.to.0] {
                        stack.push(edge.to);
                    }
                }
            }
        }

        reachable
    }

    /// Validates that the graph is connected and the goal is reachable from start
    pub fn validate(&self) -> Result<(), String> {
        let reachable = self.reachable_from(self.start);

        if !reachable.contains(&self.goal) {
            return Err("Goal is not reachable from start".to_string());
        }

        Ok(())
    }

    /// Generates concrete layout positions for all platforms based on graph connectivity and terrain
    #[allow(clippy::map_entry)]
    pub fn generate_layout(&self, seed: u64) -> std::collections::HashMap<NodeId, PlatformLayout> {
        use crate::game::tiles::TILE_SIZE;
        use rand::{Rng, SeedableRng};
        use std::collections::{HashMap, HashSet, VecDeque};

        const MIN_HORIZONTAL_SPACING_TILES: f32 = 2.0;
        const MAX_HORIZONTAL_SPACING_TILES: f32 = 4.0;
        const MIN_HEIGHT_DELTA_TILES: f32 = -2.;
        const MAX_HEIGHT_DELTA_TILES: f32 = 2.;
        const MAX_JUMP_HEIGHT_TILES: f32 = 3.5;
        const MAX_JUMP_DISTANCE_TILES: f32 = 5.;

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut layouts = HashMap::new();

        // Start node at origin
        let start_node = self.get_node(self.start).unwrap();
        let start_width = start_node.calculate_width();
        let start_height = start_node.calculate_height();

        layouts.insert(
            self.start,
            PlatformLayout {
                position: Vec2::new(TILE_SIZE * 2.0, 0.0),
                width: start_width,
                height: start_height,
            },
        );

        // Traverse graph and calculate positions
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.start);

        while let Some(current_id) = queue.pop_front() {
            if !visited.insert(current_id) {
                continue;
            }

            let current_layout = layouts[&current_id];
            let current_node = self.get_node(current_id).unwrap();

            for edge in &current_node.edges {
                if layouts.contains_key(&edge.to) {
                    // Already positioned - this is a converging path
                    // For now, we'll just skip it (could extend platform here if needed)
                    continue;
                } else {
                    // Calculate position for next platform
                    let next_node = self.get_node(edge.to).unwrap();
                    let next_width = next_node.calculate_width();
                    let next_height = next_node.calculate_height();

                    // Calculate edge-to-edge spacing (gap between platforms)
                    let gap = rng
                        .random_range(MIN_HORIZONTAL_SPACING_TILES..=MAX_HORIZONTAL_SPACING_TILES)
                        * TILE_SIZE;

                    // Position next platform: current center + half current width + gap + half next width
                    let next_x = current_layout.position.x
                        + (current_layout.width / 2.0)
                        + gap
                        + (next_width / 2.0);

                    // Randomize vertical position
                    let y_delta = rng.random_range(MIN_HEIGHT_DELTA_TILES..=MAX_HEIGHT_DELTA_TILES)
                        * TILE_SIZE;
                    let mut next_y = (current_layout.position.y + y_delta).max(0.0);

                    // Validate jump is possible
                    let horizontal_dist = (next_x - current_layout.position.x).abs();
                    let vertical_dist = next_y - current_layout.position.y;

                    // Adjust if jump is invalid
                    if horizontal_dist > MAX_JUMP_DISTANCE_TILES * TILE_SIZE
                        || vertical_dist > MAX_JUMP_HEIGHT_TILES * TILE_SIZE
                    {
                        // Bring it closer to current platform's height
                        next_y =
                            current_layout.position.y + (MAX_HEIGHT_DELTA_TILES - 0.5) * TILE_SIZE;
                        next_y = next_y.max(0.0);
                    }

                    let next_layout = PlatformLayout {
                        position: Vec2::new(next_x, next_y),
                        width: next_width,
                        height: next_height,
                    };

                    layouts.insert(edge.to, next_layout);
                    queue.push_back(edge.to);
                }
            }
        }

        layouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_graph() {
        let mut graph = PlatformGraph::new(NodeId(0), NodeId(2));

        let node1 = PlatformNode::new();
        let node2 = PlatformNode::new();
        let node3 = PlatformNode::new();

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

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_unreachable_goal() {
        let mut graph = PlatformGraph::new(NodeId(0), NodeId(1));

        // Add two nodes but don't connect them
        let node1 = PlatformNode::new();
        let node2 = PlatformNode::new();

        graph.add_node(node1);
        graph.add_node(node2);

        assert!(graph.validate().is_err());
    }
}
