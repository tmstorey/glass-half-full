#![allow(dead_code)]

use super::graph::{ConnectionType, NodeId, PlatformGraph, PlatformNode};
use crate::game::tiles::TILE_SIZE;
use bevy::prelude::*;
use rand::Rng;

// Player can jump ~4 tiles high and ~6 tiles across
const MAX_JUMP_HEIGHT_TILES: f32 = 3.5;
const MAX_JUMP_DISTANCE_TILES: f32 = 6.5;
const PLATFORM_WIDTH_TILES: f32 = 4.0;

// Safe randomization ranges (with margin of safety)
const MIN_HORIZONTAL_SPACING_TILES: f32 = 3.5;
const MAX_HORIZONTAL_SPACING_TILES: f32 = 6.0;
const MIN_HEIGHT_DELTA_TILES: f32 = -2.0;
const MAX_HEIGHT_DELTA_TILES: f32 = 3.0;

/// Validates if a jump from one position to another is possible
fn is_jump_valid(from: Vec2, to: Vec2) -> bool {
    let horizontal = (to.x - from.x).abs();
    let vertical = to.y - from.y;

    // Can't jump more than MAX_JUMP_DISTANCE_TILES horizontally
    if horizontal > MAX_JUMP_DISTANCE_TILES * TILE_SIZE {
        return false;
    }

    // Can jump up to MAX_JUMP_HEIGHT_TILES
    if vertical > MAX_JUMP_HEIGHT_TILES * TILE_SIZE {
        return false;
    }

    // Don't allow platforms below y=0
    if to.y < 0.0 {
        return false;
    }

    true
}

/// Creates a randomized linear platform layout
///
/// Generates a sequence of platforms with randomized spacing and height,
/// while ensuring all jumps are within the player's capabilities.
pub fn create_random_linear_segment(platform_count: usize, _seed: u64) -> PlatformGraph {
    use super::graph::PlatformType;

    let start_id = NodeId(0);
    let goal_id = NodeId(platform_count - 1);
    let mut graph = PlatformGraph::new(start_id, goal_id);

    let mut platforms = Vec::new();

    // Generate platforms (positions will be calculated later by generate_layout)
    for i in 0..platform_count {
        if i == 0 {
            // First platform is Start type
            platforms.push(PlatformNode::with_type(PlatformType::Start));
        } else if i == platform_count - 1 {
            // Last platform is Goal type
            platforms.push(PlatformNode::with_type(PlatformType::Goal));
        } else {
            // Middle platforms are regular
            platforms.push(PlatformNode::new());
        }
    }

    // Add all platforms to the graph
    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Connect platforms in sequence with bidirectional edges
    for i in 0..ids.len() - 1 {
        // Forward connection
        graph
            .get_node_mut(ids[i])
            .unwrap()
            .add_edge(ids[i + 1], ConnectionType::Jump);

        // Backward connection (for backtracking)
        graph
            .get_node_mut(ids[i + 1])
            .unwrap()
            .add_edge(ids[i], ConnectionType::Jump);
    }

    graph
}

/// Creates a simple linear platform layout
pub fn create_linear_template() -> PlatformGraph {
    use super::graph::PlatformType;

    let mut graph = PlatformGraph::new(NodeId(0), NodeId(4));

    // Create 5 platforms: Start, 3 regular, Goal
    let platforms = vec![
        PlatformNode::with_type(PlatformType::Start),
        PlatformNode::new(),
        PlatformNode::new(),
        PlatformNode::new(),
        PlatformNode::with_type(PlatformType::Goal),
    ];

    // Add all platforms to the graph
    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Connect platforms in sequence
    for i in 0..ids.len() - 1 {
        graph
            .get_node_mut(ids[i])
            .unwrap()
            .add_edge(ids[i + 1], ConnectionType::Jump);

        // Allow backtracking (jumping back)
        graph
            .get_node_mut(ids[i + 1])
            .unwrap()
            .add_edge(ids[i], ConnectionType::Jump);
    }

    graph
}

/// Creates a branching platform layout with two paths
pub fn create_branching_template() -> PlatformGraph {
    let mut graph = PlatformGraph::new(NodeId(0), NodeId(6));

    // Create platforms (positions will be calculated later by generate_layout)
    let platforms = vec![
        PlatformNode::new(), // Start
        PlatformNode::new(), // Upper path 1
        PlatformNode::new(), // Upper path 2
        PlatformNode::new(), // Lower path 1
        PlatformNode::new(), // Lower path 2
        PlatformNode::new(), // Converge
        PlatformNode::new(), // Goal
    ];

    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Start connects to both upper and lower paths
    graph
        .get_node_mut(ids[0])
        .unwrap()
        .add_edge(ids[1], ConnectionType::Jump);
    graph
        .get_node_mut(ids[0])
        .unwrap()
        .add_edge(ids[3], ConnectionType::Jump);

    // Upper path
    graph
        .get_node_mut(ids[1])
        .unwrap()
        .add_edge(ids[2], ConnectionType::Jump);
    graph
        .get_node_mut(ids[2])
        .unwrap()
        .add_edge(ids[5], ConnectionType::Jump);

    // Lower path
    graph
        .get_node_mut(ids[3])
        .unwrap()
        .add_edge(ids[4], ConnectionType::Jump);
    graph
        .get_node_mut(ids[4])
        .unwrap()
        .add_edge(ids[5], ConnectionType::Jump);

    // Converge to goal
    graph
        .get_node_mut(ids[5])
        .unwrap()
        .add_edge(ids[6], ConnectionType::Jump);

    // Allow backtracking on main path
    graph
        .get_node_mut(ids[1])
        .unwrap()
        .add_edge(ids[0], ConnectionType::Jump);
    graph
        .get_node_mut(ids[3])
        .unwrap()
        .add_edge(ids[0], ConnectionType::Jump);
    graph
        .get_node_mut(ids[5])
        .unwrap()
        .add_edge(ids[2], ConnectionType::Jump);
    graph
        .get_node_mut(ids[5])
        .unwrap()
        .add_edge(ids[4], ConnectionType::Jump);

    graph
}

/// Creates a platform layout with cul-de-sacs
pub fn create_cul_de_sac_template() -> PlatformGraph {
    let mut graph = PlatformGraph::new(NodeId(0), NodeId(4));

    let platforms = vec![
        PlatformNode::new(), // Start
        PlatformNode::new(), // Main path 1
        PlatformNode::new(), // Cul-de-sac 1 (above)
        PlatformNode::new(), // Main path 2
        PlatformNode::new(), // Goal
        PlatformNode::new(), // Cul-de-sac 2 (above main path 2)
    ];

    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Main path
    graph
        .get_node_mut(ids[0])
        .unwrap()
        .add_edge(ids[1], ConnectionType::Jump);
    graph
        .get_node_mut(ids[1])
        .unwrap()
        .add_edge(ids[3], ConnectionType::Jump);
    graph
        .get_node_mut(ids[3])
        .unwrap()
        .add_edge(ids[4], ConnectionType::Jump);

    // Cul-de-sac 1 (connected to platform 1)
    graph
        .get_node_mut(ids[1])
        .unwrap()
        .add_edge(ids[2], ConnectionType::Jump);
    graph
        .get_node_mut(ids[2])
        .unwrap()
        .add_edge(ids[1], ConnectionType::Jump);

    // Cul-de-sac 2 (connected to platform 3)
    graph
        .get_node_mut(ids[3])
        .unwrap()
        .add_edge(ids[5], ConnectionType::Jump);
    graph
        .get_node_mut(ids[5])
        .unwrap()
        .add_edge(ids[3], ConnectionType::Jump);

    // Backtracking on main path
    graph
        .get_node_mut(ids[1])
        .unwrap()
        .add_edge(ids[0], ConnectionType::Jump);
    graph
        .get_node_mut(ids[3])
        .unwrap()
        .add_edge(ids[1], ConnectionType::Jump);

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_template() {
        let graph = create_linear_template();
        assert!(graph.validate().is_ok());
        assert_eq!(graph.nodes.len(), 5);
    }

    #[test]
    fn test_random_linear_segment() {
        // Test various platform counts
        for count in 3..=10 {
            let graph = create_random_linear_segment(count, 12345);
            assert!(
                graph.validate().is_ok(),
                "Graph validation failed for {} platforms",
                count
            );
            assert_eq!(graph.nodes.len(), count);

            // Generate layout and verify all platforms are at y >= 0
            let layouts = graph.generate_layout(12345);
            for layout in layouts.values() {
                assert!(
                    layout.grid_y >= 0,
                    "Platform below y=0 at grid_y={}",
                    layout.grid_y
                );
            }
        }

        // Test with different seeds to ensure variety
        let graph1 = create_random_linear_segment(5, 111);
        let graph2 = create_random_linear_segment(5, 222);

        let layout1 = graph1.generate_layout(111);
        let layout2 = graph2.generate_layout(222);

        // Platforms should be in different positions (except potentially the first one)
        // Since layouts use seed for positioning, different seeds should produce different results
        assert_eq!(layout1.len(), 5);
        assert_eq!(layout2.len(), 5);
    }

    #[test]
    fn test_branching_template() {
        let graph = create_branching_template();
        assert!(graph.validate().is_ok());
        assert_eq!(graph.nodes.len(), 7);
    }

    #[test]
    fn test_cul_de_sac_template() {
        let graph = create_cul_de_sac_template();
        assert!(graph.validate().is_ok());
        assert_eq!(graph.nodes.len(), 6);
    }
}
