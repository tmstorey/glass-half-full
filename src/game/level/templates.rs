#![allow(dead_code)]

use super::graph::{ConnectionType, NodeId, PlatformGraph, PlatformNode};
use crate::game::tiles::TILE_SIZE;
use bevy::prelude::*;

// Player can jump ~4 tiles high and ~6 tiles across
const MAX_JUMP_HEIGHT_TILES: f32 = 3.5;
const MAX_JUMP_DISTANCE_TILES: f32 = 5.5;
const PLATFORM_WIDTH_TILES: f32 = 4.0;

/// Creates a simple linear platform layout
pub fn create_linear_template() -> PlatformGraph {
    let mut graph = PlatformGraph::new(NodeId(0), NodeId(4));

    // Create 5 platforms in a line with reasonable jump distances
    // All platforms at y >= 0, with varied heights for interest
    let platforms = vec![
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 2.0, TILE_SIZE * 0.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ),
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 8.0, TILE_SIZE * 2.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ),
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 14.0, TILE_SIZE * 3.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ),
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 20.0, TILE_SIZE * 1.5),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ),
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 26.0, TILE_SIZE * 0.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ),
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

    // Create platforms with tile-based positioning
    let platforms = vec![
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 2.0, TILE_SIZE * 2.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Start
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 8.0, TILE_SIZE * 4.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Upper path 1
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 14.0, TILE_SIZE * 5.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Upper path 2
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 8.0, TILE_SIZE * 0.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Lower path 1
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 14.0, TILE_SIZE * 0.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Lower path 2
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 20.0, TILE_SIZE * 2.5),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Converge
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 26.0, TILE_SIZE * 2.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Goal
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
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 2.0, TILE_SIZE * 1.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Start
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 8.0, TILE_SIZE * 1.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Main path 1
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 8.0, TILE_SIZE * 4.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Cul-de-sac 1 (above)
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 14.0, TILE_SIZE * 1.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Main path 2
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 20.0, TILE_SIZE * 1.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Goal
        PlatformNode::new(
            Vec2::new(TILE_SIZE * 14.0, TILE_SIZE * 4.0),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ), // Cul-de-sac 2 (above main path 2)
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
