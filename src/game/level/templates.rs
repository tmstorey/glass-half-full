#![allow(dead_code)]

use super::graph::{ConnectionType, LayoutDirection, NodeId, PlatformGraph, PlatformNode};
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

/// Creates a linear platform layout with random directional variation
///
/// Generates a sequence of platforms with randomly varying directions (Right or RightUp).
/// If seed is provided, randomly generates platform count (5-8) and chooses random directions.
/// If seed is not provided, creates 5 platforms with all Right directions (non-randomized).
///
/// # Arguments
/// * `seed` - Optional seed for randomization (if None, uses 5 platforms with Right directions)
pub fn create_linear_template(seed: Option<u64>) -> PlatformGraph {
    use super::graph::PlatformType;
    use rand::{Rng, SeedableRng};

    // Determine platform count based on seed
    let count = if let Some(s) = seed {
        let mut count_rng = rand::rngs::StdRng::seed_from_u64(s);
        count_rng.random_range(5..=8)
    } else {
        5
    };
    let start_id = NodeId(0);
    let goal_id = NodeId(count - 1);
    let mut graph = PlatformGraph::new(start_id, goal_id);

    let mut platforms = Vec::new();

    // Generate platforms (positions will be calculated later by generate_layout)
    for i in 0..count {
        if i == 0 {
            // First platform is Start type
            platforms.push(PlatformNode::with_type(PlatformType::Start));
        } else if i == count - 1 {
            // Last platform is Goal type
            platforms.push(PlatformNode::with_type(PlatformType::Goal));
        } else {
            // Middle platforms are regular
            platforms.push(PlatformNode::new());
        }
    }

    // Add all platforms to the graph
    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Create RNG if seed provided
    let mut rng = seed.map(rand::rngs::StdRng::seed_from_u64);

    // Connect platforms in sequence with bidirectional edges
    for i in 0..ids.len() - 1 {
        // Randomly choose between Right and RightUp if seed provided
        let forward_direction = if let Some(ref mut rng) = rng {
            if rng.random_bool(0.5) {
                LayoutDirection::RightUp
            } else {
                LayoutDirection::Right
            }
        } else {
            LayoutDirection::Right
        };

        // Determine backward direction based on forward direction
        let backward_direction = match forward_direction {
            LayoutDirection::RightUp => LayoutDirection::LeftDown,
            _ => LayoutDirection::Left,
        };

        // Forward connection
        graph.get_node_mut(ids[i]).unwrap().add_edge(
            ids[i + 1],
            ConnectionType::Jump {
                direction: forward_direction,
            },
        );

        // Backward connection (for backtracking)
        graph.get_node_mut(ids[i + 1]).unwrap().add_edge(
            ids[i],
            ConnectionType::Jump {
                direction: backward_direction,
            },
        );
    }

    graph
}

/// Creates a branching platform layout with two paths
pub fn create_branching_template() -> PlatformGraph {
    use super::graph::PlatformType;

    let mut graph = PlatformGraph::new(NodeId(0), NodeId(6));

    // Create platforms (positions will be calculated later by generate_layout)
    let platforms = vec![
        PlatformNode::with_type(PlatformType::Start), // Start
        PlatformNode::new(),                          // Upper path 1
        PlatformNode::new(),                          // Upper path 2
        PlatformNode::new(),                          // Lower path 1
        PlatformNode::new(),                          // Lower path 2
        PlatformNode::new(),                          // Converge
        PlatformNode::with_type(PlatformType::Goal),  // Goal
    ];

    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Start connects to both upper and lower paths
    graph.get_node_mut(ids[0]).unwrap().add_edge(
        ids[1],
        ConnectionType::Jump {
            direction: LayoutDirection::RightUp,
        },
    );
    graph.get_node_mut(ids[0]).unwrap().add_edge(
        ids[3],
        ConnectionType::Jump {
            direction: LayoutDirection::RightDown,
        },
    );

    // Upper path
    graph.get_node_mut(ids[1]).unwrap().add_edge(
        ids[2],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[2]).unwrap().add_edge(
        ids[5],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );

    // Lower path
    graph.get_node_mut(ids[3]).unwrap().add_edge(
        ids[4],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[4]).unwrap().add_edge(
        ids[5],
        ConnectionType::Jump {
            direction: LayoutDirection::RightUp,
        },
    );

    // Converge to goal
    graph.get_node_mut(ids[5]).unwrap().add_edge(
        ids[6],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );

    // Allow backtracking on main path
    graph.get_node_mut(ids[1]).unwrap().add_edge(
        ids[0],
        ConnectionType::Jump {
            direction: LayoutDirection::LeftDown,
        },
    );
    graph.get_node_mut(ids[3]).unwrap().add_edge(
        ids[0],
        ConnectionType::Jump {
            direction: LayoutDirection::LeftUp,
        },
    );
    graph.get_node_mut(ids[5]).unwrap().add_edge(
        ids[2],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );
    graph.get_node_mut(ids[5]).unwrap().add_edge(
        ids[4],
        ConnectionType::Jump {
            direction: LayoutDirection::LeftDown,
        },
    );

    graph
}

/// Creates a platform layout with cul-de-sacs
pub fn create_cul_de_sac_template() -> PlatformGraph {
    use super::graph::PlatformType;

    let mut graph = PlatformGraph::new(NodeId(0), NodeId(4));

    let platforms = vec![
        PlatformNode::with_type(PlatformType::Start), // Start
        PlatformNode::new(),                          // Main path 1
        PlatformNode::new(),                          // Cul-de-sac 1 (above)
        PlatformNode::new(),                          // Main path 2
        PlatformNode::with_type(PlatformType::Goal),  // Goal
        PlatformNode::new(),                          // Cul-de-sac 2 (above main path 2)
    ];

    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Main path
    graph.get_node_mut(ids[0]).unwrap().add_edge(
        ids[1],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[1]).unwrap().add_edge(
        ids[3],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[3]).unwrap().add_edge(
        ids[4],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );

    // Cul-de-sac 1 (connected to platform 1, going up)
    graph.get_node_mut(ids[1]).unwrap().add_edge(
        ids[2],
        ConnectionType::Jump {
            direction: LayoutDirection::RightUp,
        },
    );
    graph.get_node_mut(ids[2]).unwrap().add_edge(
        ids[1],
        ConnectionType::Jump {
            direction: LayoutDirection::LeftDown,
        },
    );

    // Cul-de-sac 2 (connected to platform 3, going up)
    graph.get_node_mut(ids[3]).unwrap().add_edge(
        ids[5],
        ConnectionType::Jump {
            direction: LayoutDirection::RightUp,
        },
    );
    graph.get_node_mut(ids[5]).unwrap().add_edge(
        ids[3],
        ConnectionType::Jump {
            direction: LayoutDirection::LeftDown,
        },
    );

    // Backtracking on main path
    graph.get_node_mut(ids[1]).unwrap().add_edge(
        ids[0],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );
    graph.get_node_mut(ids[3]).unwrap().add_edge(
        ids[1],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );

    graph
}

/// Creates a zigzag pattern with platforms alternating up and down
pub fn create_zigzag_template() -> PlatformGraph {
    use super::graph::PlatformType;

    let mut graph = PlatformGraph::new(NodeId(0), NodeId(7));

    // Create platforms: Start, then zigzag pattern (up, down, up, down, up), then Goal
    let platforms = vec![
        PlatformNode::with_type(PlatformType::Start),
        PlatformNode::new(), // Up
        PlatformNode::new(), // Down
        PlatformNode::new(), // Up
        PlatformNode::new(), // Down
        PlatformNode::new(), // Up
        PlatformNode::new(), // Down (connecting to goal)
        PlatformNode::with_type(PlatformType::Goal),
    ];

    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Connect with alternating directions
    for i in 0..ids.len() - 1 {
        let direction = if i % 2 == 0 {
            // Even index: go right and up
            LayoutDirection::RightUp
        } else {
            // Odd index: go right and down
            LayoutDirection::RightDown
        };

        // Forward connection
        graph
            .get_node_mut(ids[i])
            .unwrap()
            .add_edge(ids[i + 1], ConnectionType::Jump { direction });

        // Backward connection (opposite direction)
        let back_direction = match direction {
            LayoutDirection::RightUp => LayoutDirection::LeftDown,
            LayoutDirection::RightDown => LayoutDirection::LeftUp,
            _ => LayoutDirection::Left,
        };

        graph.get_node_mut(ids[i + 1]).unwrap().add_edge(
            ids[i],
            ConnectionType::Jump {
                direction: back_direction,
            },
        );
    }

    graph
}

/// Creates a layout with both grounded and floating platforms
pub fn create_ground_and_floating_template() -> PlatformGraph {
    use super::graph::PlatformType;

    let mut graph = PlatformGraph::new(NodeId(0), NodeId(7));

    // Create platforms: Start (grounded), 2 grounded, 3 floating, 1 grounded, Goal (grounded)
    let platforms = vec![
        PlatformNode::with_type(PlatformType::Start), // Grounded
        PlatformNode::with_type(PlatformType::Grounded), // Grounded
        PlatformNode::with_type(PlatformType::Grounded), // Grounded
        PlatformNode::with_type(PlatformType::Floating), // Floating (up)
        PlatformNode::with_type(PlatformType::Floating), // Floating
        PlatformNode::with_type(PlatformType::Floating), // Floating (down)
        PlatformNode::with_type(PlatformType::Grounded), // Grounded
        PlatformNode::with_type(PlatformType::Goal),  // Grounded
    ];

    let ids: Vec<NodeId> = platforms.into_iter().map(|p| graph.add_node(p)).collect();

    // Connect platforms
    // Start to grounded 1
    graph.get_node_mut(ids[0]).unwrap().add_edge(
        ids[1],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[1]).unwrap().add_edge(
        ids[0],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );

    // Grounded 1 to grounded 2
    graph.get_node_mut(ids[1]).unwrap().add_edge(
        ids[2],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[2]).unwrap().add_edge(
        ids[1],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );

    // Grounded 2 to floating 1 (up)
    graph.get_node_mut(ids[2]).unwrap().add_edge(
        ids[3],
        ConnectionType::Jump {
            direction: LayoutDirection::RightUp,
        },
    );
    graph.get_node_mut(ids[3]).unwrap().add_edge(
        ids[2],
        ConnectionType::Jump {
            direction: LayoutDirection::LeftDown,
        },
    );

    // Floating 1 to floating 2
    graph.get_node_mut(ids[3]).unwrap().add_edge(
        ids[4],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[4]).unwrap().add_edge(
        ids[3],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );

    // Floating 2 to floating 3
    graph.get_node_mut(ids[4]).unwrap().add_edge(
        ids[5],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[5]).unwrap().add_edge(
        ids[4],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );

    // Floating 3 to grounded 3 (down)
    graph.get_node_mut(ids[5]).unwrap().add_edge(
        ids[6],
        ConnectionType::Jump {
            direction: LayoutDirection::RightDown,
        },
    );
    graph.get_node_mut(ids[6]).unwrap().add_edge(
        ids[5],
        ConnectionType::Jump {
            direction: LayoutDirection::LeftUp,
        },
    );

    // Grounded 3 to goal
    graph.get_node_mut(ids[6]).unwrap().add_edge(
        ids[7],
        ConnectionType::Jump {
            direction: LayoutDirection::Right,
        },
    );
    graph.get_node_mut(ids[7]).unwrap().add_edge(
        ids[6],
        ConnectionType::Jump {
            direction: LayoutDirection::Left,
        },
    );

    graph
}

/// Merges multiple platform graphs into one by connecting their goal and start nodes
///
/// This takes multiple template graphs and combines them into a single longer graph.
/// The goal node of each graph (except the last) is converted to a regular platform,
/// and the start node of each graph (except the first) is converted to a regular platform.
/// These converted platforms are then connected with bidirectional edges.
///
/// # Arguments
/// * `graphs` - A vector of platform graphs to merge (must have at least 1 graph)
///
/// # Returns
/// A single merged platform graph, or the original graph if only one was provided
pub fn merge_graphs(mut graphs: Vec<PlatformGraph>) -> PlatformGraph {
    use super::graph::PlatformType;

    if graphs.is_empty() {
        panic!("Cannot merge empty list of graphs");
    }

    if graphs.len() == 1 {
        return graphs.pop().unwrap();
    }

    // Start with the first graph as the base
    let mut merged = graphs.remove(0);
    let mut node_offset = merged.nodes.len();

    // Process each remaining graph
    for mut next_graph in graphs {
        // Convert the current merged graph's goal to a regular platform
        if let Some(goal_node) = merged.get_node_mut(merged.goal) {
            goal_node.platform_type = PlatformType::Floating;
        }

        // Convert the next graph's start to a regular platform
        if let Some(start_node) = next_graph.get_node_mut(next_graph.start) {
            start_node.platform_type = PlatformType::Floating;
        }

        // Store the old goal and start IDs before merging
        let old_merged_goal = merged.goal;
        let old_next_start = next_graph.start;

        // Create a mapping of old NodeIds to new NodeIds for the next graph
        let mut id_mapping = std::collections::HashMap::new();
        for (old_idx, node) in next_graph.nodes.into_iter().enumerate() {
            let old_id = NodeId(old_idx);
            let mut new_node = node;

            // Remap all edges in this node
            for edge in &mut new_node.edges {
                let new_to = NodeId(edge.to.0 + node_offset);
                edge.to = new_to;
            }

            let new_id = merged.add_node(new_node);
            id_mapping.insert(old_id, new_id);
        }

        // Connect the old merged goal to the old next start
        // (both have been remapped to regular platforms)
        let remapped_next_start = id_mapping[&old_next_start];

        // Add bidirectional edges between the connection point
        merged.get_node_mut(old_merged_goal).unwrap().add_edge(
            remapped_next_start,
            ConnectionType::Jump {
                direction: LayoutDirection::Right,
            },
        );

        merged.get_node_mut(remapped_next_start).unwrap().add_edge(
            old_merged_goal,
            ConnectionType::Jump {
                direction: LayoutDirection::Left,
            },
        );

        // Update the merged graph's goal to the remapped goal from next_graph
        merged.goal = id_mapping[&next_graph.goal];

        // Update node_offset for the next iteration
        node_offset = merged.nodes.len();
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_template() {
        // Test default (no randomization, 5 platforms)
        let graph = create_linear_template(None);
        assert!(graph.validate().is_ok());
        assert_eq!(graph.nodes.len(), 5);

        // Test various seeds with randomization
        for seed in [12345, 54321, 99999] {
            let graph = create_linear_template(Some(seed));
            assert!(
                graph.validate().is_ok(),
                "Graph validation failed for seed {}",
                seed
            );
            // Platform count should be between 5 and 8
            assert!(graph.nodes.len() >= 5 && graph.nodes.len() <= 8);

            // Generate layout and verify all platforms are at y >= 0
            let layouts = graph.generate_layout(seed);
            for layout in layouts.values() {
                assert!(
                    layout.grid_y >= 0,
                    "Platform below y=0 at grid_y={}",
                    layout.grid_y
                );
            }
        }

        // Test with different seeds to ensure variety
        let graph1 = create_linear_template(Some(111));
        let graph2 = create_linear_template(Some(222));

        let layout1 = graph1.generate_layout(111);
        let layout2 = graph2.generate_layout(222);

        // Both should have valid layouts
        assert!(layout1.len() >= 5 && layout1.len() <= 8);
        assert!(layout2.len() >= 5 && layout2.len() <= 8);
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

    #[test]
    fn test_zigzag_template() {
        let graph = create_zigzag_template();
        assert!(graph.validate().is_ok());
        assert_eq!(graph.nodes.len(), 8);
    }

    #[test]
    fn test_ground_and_floating_template() {
        let graph = create_ground_and_floating_template();
        assert!(graph.validate().is_ok());
        assert_eq!(graph.nodes.len(), 8);
    }

    #[test]
    fn test_merge_two_graphs() {
        let graph1 = create_linear_template(None); // 5 nodes
        let graph2 = create_linear_template(None); // 5 nodes

        let merged = merge_graphs(vec![graph1, graph2]);

        // Should have 10 nodes total (5 + 5)
        assert_eq!(merged.nodes.len(), 10);

        // Merged graph should still be valid
        assert!(merged.validate().is_ok());

        // Start should be from first graph (NodeId(0))
        assert_eq!(merged.start, NodeId(0));

        // Goal should be from second graph (originally NodeId(4), now NodeId(9))
        assert_eq!(merged.goal, NodeId(9));

        // The connection point (old goal of graph1 and old start of graph2)
        // should both be regular platforms now
        use super::super::graph::PlatformType;
        let old_goal_node = merged.get_node(NodeId(4)).unwrap();
        assert_eq!(old_goal_node.platform_type, PlatformType::Floating);

        let old_start_node = merged.get_node(NodeId(5)).unwrap();
        assert_eq!(old_start_node.platform_type, PlatformType::Floating);
    }

    #[test]
    fn test_merge_three_graphs() {
        let graph1 = create_linear_template(None); // 5 nodes
        let graph2 = create_linear_template(None); // 5 nodes
        let graph3 = create_linear_template(None); // 5 nodes

        let merged = merge_graphs(vec![graph1, graph2, graph3]);

        // Should have 15 nodes total (5 + 5 + 5)
        assert_eq!(merged.nodes.len(), 15);

        // Merged graph should still be valid
        assert!(merged.validate().is_ok());

        // Start should be from first graph
        assert_eq!(merged.start, NodeId(0));

        // Goal should be from third graph (originally NodeId(4), now NodeId(14))
        assert_eq!(merged.goal, NodeId(14));
    }

    #[test]
    fn test_merge_single_graph() {
        let graph = create_linear_template(None);
        let original_len = graph.nodes.len();

        let merged = merge_graphs(vec![graph]);

        // Should be unchanged
        assert_eq!(merged.nodes.len(), original_len);
        assert!(merged.validate().is_ok());
    }
}
