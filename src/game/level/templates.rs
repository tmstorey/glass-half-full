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
pub fn create_random_linear_segment(platform_count: usize, seed: u64) -> PlatformGraph {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let start_id = NodeId(0);
    let goal_id = NodeId(platform_count - 1);
    let mut graph = PlatformGraph::new(start_id, goal_id);

    let mut platforms = Vec::new();
    let mut current_x = TILE_SIZE * 2.0;
    let mut current_y = TILE_SIZE * 0.0; // Start at ground level

    // Generate platforms
    for _ in 0..platform_count {
        platforms.push(PlatformNode::new(
            Vec2::new(current_x, current_y),
            TILE_SIZE * PLATFORM_WIDTH_TILES,
        ));

        // Randomize next platform position
        let x_spacing =
            rng.random_range(MIN_HORIZONTAL_SPACING_TILES..=MAX_HORIZONTAL_SPACING_TILES);
        let y_delta = rng.random_range(MIN_HEIGHT_DELTA_TILES..=MAX_HEIGHT_DELTA_TILES);

        current_x += x_spacing * TILE_SIZE;
        current_y = (current_y + y_delta * TILE_SIZE).max(0.0); // Ensure y >= 0

        // Validate the jump would be possible
        if platforms.len() > 1 {
            let from = platforms[platforms.len() - 2].position;
            let to = Vec2::new(current_x, current_y);

            // If jump is invalid, adjust height to make it valid
            if !is_jump_valid(from, to) {
                // Try bringing it closer to the previous platform's height
                current_y = from.y + (MAX_HEIGHT_DELTA_TILES - 0.5) * TILE_SIZE;
                current_y = current_y.max(0.0);
            }
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

            // Verify all platforms are at y >= 0
            for node in &graph.nodes {
                assert!(
                    node.position.y >= 0.0,
                    "Platform below y=0 at y={}",
                    node.position.y
                );
            }
        }

        // Test with different seeds to ensure variety
        let graph1 = create_random_linear_segment(5, 111);
        let graph2 = create_random_linear_segment(5, 222);

        // Platforms should be in different positions (except the first one)
        assert_ne!(
            graph1.nodes[1].position, graph2.nodes[1].position,
            "Different seeds should produce different layouts"
        );
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
