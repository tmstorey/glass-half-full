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
    /// World position of this platform
    pub position: Vec2,
    /// Width of the platform in world units
    pub width: f32,
    /// Connections to other platforms
    pub edges: Vec<Edge>,
    /// Smart terrain objects placed on this platform
    pub terrain_objects: Vec<SmartTerrain>,
}

impl PlatformNode {
    pub fn new(position: Vec2, width: f32) -> Self {
        Self {
            position,
            width,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_graph() {
        let mut graph = PlatformGraph::new(NodeId(0), NodeId(2));

        let node1 = PlatformNode::new(Vec2::new(0.0, 0.0), 100.0);
        let node2 = PlatformNode::new(Vec2::new(150.0, 50.0), 100.0);
        let node3 = PlatformNode::new(Vec2::new(300.0, 0.0), 100.0);

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
        let node1 = PlatformNode::new(Vec2::new(0.0, 0.0), 100.0);
        let node2 = PlatformNode::new(Vec2::new(200.0, 0.0), 100.0);

        graph.add_node(node1);
        graph.add_node(node2);

        assert!(graph.validate().is_err());
    }
}
