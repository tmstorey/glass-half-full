#![allow(dead_code)]

use bevy::prelude::*;

/// Unique identifier for a platform node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Layout direction hint for platform placement
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    /// Horizontal jump to the right
    #[default]
    Right,
    /// Horizontal jump to the left
    Left,
    /// Jump diagonally up and to the right
    RightUp,
    /// Jump diagonally up and to the left
    LeftUp,
    /// Fall/jump diagonally down and to the right
    RightDown,
    /// Fall/jump diagonally down and to the left
    LeftDown,
}

/// Represents the type of connection between platforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    /// A jump that's always possible
    Jump { direction: LayoutDirection },
    /// A one-way fall (can go down but not up)
    Fall { direction: LayoutDirection },
    /// Requires a moving platform to be active
    MovingPlatform {
        platform_entity: Option<Entity>,
        required: bool,
        direction: LayoutDirection,
    },
}

impl ConnectionType {
    /// Extract the layout direction from this connection type
    pub fn direction(&self) -> LayoutDirection {
        match self {
            ConnectionType::Jump { direction } => *direction,
            ConnectionType::Fall { direction } => *direction,
            ConnectionType::MovingPlatform { direction, .. } => *direction,
        }
    }
}

/// An edge connecting two platform nodes
#[derive(Debug, Clone)]
pub struct Edge {
    pub to: NodeId,
    pub connection_type: ConnectionType,
}

/// Ground level for grounded platforms (in world units)
pub const GROUND_LEVEL: f32 = -64.0;

/// Height of boundary walls on start/goal platforms (in tiles)
pub const WALL_HEIGHT: i32 = 5;

/// Type of platform structure
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum PlatformType {
    /// Floating platform (just the platform tiles)
    #[default]
    Floating,
    /// Grounded platform (dirt extends down to GROUND_LEVEL)
    Grounded,
    /// Start platform (grounded with left wall and water source)
    Start,
    /// Goal platform (grounded with right wall and goal container)
    Goal,
}

/// A node in the platform graph representing a platform section
#[derive(Debug, Clone)]
pub struct PlatformNode {
    /// Connections to other platforms
    pub edges: Vec<Edge>,
    /// Smart terrain objects placed on this platform
    pub terrain_objects: Vec<SmartTerrain>,
    /// Type of platform structure
    pub platform_type: PlatformType,
}

impl PlatformNode {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            terrain_objects: Vec::new(),
            platform_type: PlatformType::default(),
        }
    }

    /// Creates a new platform node with a specific type
    pub fn with_type(platform_type: PlatformType) -> Self {
        Self {
            edges: Vec::new(),
            terrain_objects: Vec::new(),
            platform_type,
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

    /// Calculates the width this platform should have based on its terrain or type
    pub fn calculate_width(&self) -> f32 {
        use crate::game::tiles::TILE_SIZE;

        // Start and Goal platforms are always wide enough for water
        if matches!(self.platform_type, PlatformType::Start | PlatformType::Goal) {
            return TILE_SIZE * 9.0;
        }

        if self
            .terrain_objects
            .iter()
            .any(|t| matches!(t, SmartTerrain::WaterSource))
        {
            TILE_SIZE * 9.0 // Water platforms need 9+ tiles
        } else {
            TILE_SIZE * 4.0 // Standard platform width
        }
    }

    /// Calculates the height this platform should have based on its terrain or type
    pub fn calculate_height(&self) -> u32 {
        // Start and Goal platforms are always 2 tiles high
        if matches!(self.platform_type, PlatformType::Start | PlatformType::Goal) {
            return 2;
        }

        if self
            .terrain_objects
            .iter()
            .any(|t| matches!(t, SmartTerrain::WaterSource))
        {
            2 // Water platforms are 2 tiles high
        } else {
            1 // Standard platforms are 1 tile high
        }
    }
}

/// Concrete layout information for a platform in grid coordinates
#[derive(Debug, Clone, Copy)]
pub struct PlatformLayout {
    /// Grid position (left edge, bottom of platform)
    pub grid_x: i32,
    pub grid_y: i32,
    /// Width in tiles
    pub width_tiles: i32,
    /// Height in tiles
    pub height_tiles: i32,
}

impl PlatformLayout {
    /// Get the center position in world coordinates
    pub fn center_world(&self) -> Vec2 {
        use crate::game::tiles::TILE_SIZE;
        Vec2::new(
            (self.grid_x as f32 + self.width_tiles as f32 / 2.0) * TILE_SIZE,
            (self.grid_y as f32 + self.height_tiles as f32 / 2.0) * TILE_SIZE,
        )
    }

    /// Get the left edge in world coordinates
    pub fn left_edge_world(&self) -> f32 {
        use crate::game::tiles::TILE_SIZE;
        self.grid_x as f32 * TILE_SIZE
    }

    /// Get the right edge in world coordinates
    pub fn right_edge_world(&self) -> f32 {
        use crate::game::tiles::TILE_SIZE;
        (self.grid_x + self.width_tiles) as f32 * TILE_SIZE
    }

    /// Get the bottom edge in world coordinates
    pub fn bottom_world(&self) -> f32 {
        use crate::game::tiles::TILE_SIZE;
        self.grid_y as f32 * TILE_SIZE
    }

    /// Get the top edge in world coordinates
    pub fn top_world(&self) -> f32 {
        use crate::game::tiles::TILE_SIZE;
        (self.grid_y + self.height_tiles) as f32 * TILE_SIZE
    }
}

/// Smart terrain types that can be placed in the level
#[derive(Debug, Clone, PartialEq)]
pub enum SmartTerrain {
    /// Infinite water source
    WaterSource,
    /// Infinite snow source
    SnowSource,
    /// Fire that blocks passage with a wall (must be extinguished to pass)
    BlockingFire { extinguished: bool },
    /// Fire used to melt snow into water (no wall)
    SnowMeltFire { extinguished: bool },
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
        use rand::{Rng, SeedableRng};
        use std::collections::{HashMap, HashSet, VecDeque};

        const MIN_HORIZONTAL_SPACING_TILES: i32 = 2;
        const MAX_HORIZONTAL_SPACING_TILES: i32 = 4;
        const MIN_HEIGHT_DELTA_TILES: i32 = -2;
        const MAX_HEIGHT_DELTA_TILES: i32 = 2;
        const MAX_JUMP_HEIGHT_TILES: i32 = 3;
        const MAX_JUMP_DISTANCE_TILES: i32 = 6;

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut layouts = HashMap::new();

        // Start node at grid origin (2, 0)
        let start_node = self.get_node(self.start).unwrap();
        let start_width_tiles = (start_node.calculate_width() / 32.0) as i32; // Assuming TILE_SIZE = 32
        let start_height_tiles = start_node.calculate_height() as i32;

        layouts.insert(
            self.start,
            PlatformLayout {
                grid_x: 2,
                grid_y: 0,
                width_tiles: start_width_tiles,
                height_tiles: start_height_tiles,
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
                    let next_width_tiles = (next_node.calculate_width() / 32.0) as i32;
                    let next_height_tiles = next_node.calculate_height() as i32;

                    // Get direction hint from edge
                    let direction = edge.connection_type.direction();

                    // Determine horizontal direction and vertical bias
                    let (x_direction, y_bias) = match direction {
                        LayoutDirection::Right => (1, 0),      // Move right, neutral height
                        LayoutDirection::Left => (-1, 0),      // Move left, neutral height
                        LayoutDirection::RightUp => (1, 1),    // Move right and prefer upward
                        LayoutDirection::LeftUp => (-1, 1),    // Move left and prefer upward
                        LayoutDirection::RightDown => (1, -1), // Move right and prefer downward
                        LayoutDirection::LeftDown => (-1, -1), // Move left and prefer downward
                    };

                    // Calculate edge-to-edge spacing (gap between platforms)
                    let gap_tiles = rng
                        .random_range(MIN_HORIZONTAL_SPACING_TILES..=MAX_HORIZONTAL_SPACING_TILES);

                    // Position next platform based on direction
                    let next_grid_x = if x_direction > 0 {
                        // Moving right: place after current platform
                        current_layout.grid_x + current_layout.width_tiles + gap_tiles
                    } else {
                        // Moving left: place before current platform
                        current_layout.grid_x - gap_tiles - next_width_tiles
                    };

                    // Calculate vertical position with directional bias
                    let y_delta = if y_bias != 0 {
                        // Biased direction: prefer up or down
                        rng.random_range(0..=MAX_HEIGHT_DELTA_TILES) * y_bias
                    } else {
                        // Neutral direction: randomize
                        rng.random_range(MIN_HEIGHT_DELTA_TILES..=MAX_HEIGHT_DELTA_TILES)
                    };
                    let mut next_grid_y = (current_layout.grid_y + y_delta).max(0);

                    // Validate jump is possible (using grid distances)
                    let horizontal_dist = (next_grid_x - current_layout.grid_x).abs();
                    let vertical_dist = next_grid_y - current_layout.grid_y;

                    // Adjust if jump is invalid
                    if horizontal_dist > MAX_JUMP_DISTANCE_TILES
                        || vertical_dist > MAX_JUMP_HEIGHT_TILES
                    {
                        // Bring it closer to current platform's height
                        next_grid_y = current_layout.grid_y + (MAX_HEIGHT_DELTA_TILES - 1);
                        next_grid_y = next_grid_y.max(0);
                    }

                    let next_layout = PlatformLayout {
                        grid_x: next_grid_x,
                        grid_y: next_grid_y,
                        width_tiles: next_width_tiles,
                        height_tiles: next_height_tiles,
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

        graph.get_node_mut(id1).unwrap().add_edge(
            id2,
            ConnectionType::Jump {
                direction: LayoutDirection::Right,
            },
        );
        graph.get_node_mut(id2).unwrap().add_edge(
            id3,
            ConnectionType::Jump {
                direction: LayoutDirection::Right,
            },
        );

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
