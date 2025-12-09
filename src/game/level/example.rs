#![allow(dead_code)]

use super::generator::{CausalityGenerator, Difficulty, GeneratorConfig};
use super::templates::create_linear_template;

/// Example function that demonstrates the level generation system
pub fn generate_example_level() {
    println!("\n=== Procedural Level Generation Example ===\n");

    // Create a simple linear platform graph
    let mut graph = create_linear_template();
    println!("Created platform graph with {} nodes", graph.nodes.len());
    println!("Start: {:?}, Goal: {:?}", graph.start, graph.goal);

    // Validate the graph
    match graph.validate() {
        Ok(_) => println!("✓ Graph is valid (goal reachable from start)"),
        Err(e) => {
            println!("✗ Graph validation failed: {}", e);
            return;
        }
    }

    // Generate a causality chain for easy difficulty
    let config = GeneratorConfig {
        difficulty: Difficulty::Easy,
        seed: 42,
    };

    let mut generator = CausalityGenerator::new(config);
    println!("\nGenerating causality chain (Difficulty: Easy)...");

    match generator.generate_chain(&graph) {
        Ok(chain) => {
            println!("✓ Generated chain with {} steps", chain.nodes.len());

            // Validate the chain
            match chain.validate() {
                Ok(_) => println!("✓ Chain is valid (all causes satisfied)"),
                Err(e) => {
                    println!("✗ Chain validation failed: {}", e);
                    return;
                }
            }

            // Print the chain in forward order (start to goal)
            println!("\nCausality chain (player perspective):");
            for (i, node) in chain.forward_order().enumerate() {
                println!("  {}. {:?} at {:?}", i + 1, node.effect, node.location);
                println!("     Using: {:?}", node.terrain);
            }

            // Apply the chain to the graph
            match generator.apply_chain_to_graph(&chain, &mut graph) {
                Ok(_) => {
                    println!("\n✓ Applied chain to graph");

                    // Show what terrain was placed on each platform
                    println!("\nPlatform terrain layout:");
                    for (i, node) in graph.nodes.iter().enumerate() {
                        if !node.terrain_objects.is_empty() {
                            println!("  Platform {} at {:?}:", i, node.position);
                            for terrain in &node.terrain_objects {
                                println!("    - {:?}", terrain);
                            }
                        }
                    }
                }
                Err(e) => println!("✗ Failed to apply chain: {}", e),
            }
        }
        Err(e) => println!("✗ Chain generation failed: {}", e),
    }

    println!("\n=== Example Complete ===\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_generates() {
        // Just ensure the example runs without panicking
        generate_example_level();
    }
}
