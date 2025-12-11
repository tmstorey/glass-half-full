use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use super::character::Character;
use super::controls::Action;
use super::level::BucketContent;
use super::level::PlayerSpawnPoint;
use super::level::objects::{Container, Fire, Snow, Water};
use crate::PausableSystems;
use crate::screens::Screen;

/// Message triggered when a level is completed
#[derive(Message)]
pub struct LevelCompleteMessage;

pub fn plugin(app: &mut App) {
    app.add_message::<LevelCompleteMessage>();
    app.add_systems(
        Update,
        (
            interact_with_water,
            interact_with_container,
            interact_with_fire,
            interact_with_snow,
            touch_active_fire,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

const INTERACTION_RANGE: f32 = 64.0; // How close player needs to be to interact

/// System to handle picking up water
fn interact_with_water(
    action_query: Query<&ActionState<Action>>,
    character_query: Query<&Transform, With<Character>>,
    water_query: Query<&Transform, With<Water>>,
    mut bucket_content: ResMut<BucketContent>,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };

    // Only check when Use is just pressed
    if !action_state.just_pressed(&Action::Use) {
        return;
    }

    let Ok(character_transform) = character_query.single() else {
        return;
    };

    let character_pos = character_transform.translation;

    // Check if there's a water source nearby
    for water_transform in &water_query {
        let water_pos = water_transform.translation;
        let distance = character_pos.distance(water_pos);

        if distance <= INTERACTION_RANGE {
            // Fill bucket with water
            *bucket_content = BucketContent::Water;
            info!("Picked up water!");
            return; // Only interact with one water source at a time
        }
    }
}

/// System to handle pouring water into containers
fn interact_with_container(
    action_query: Query<&ActionState<Action>>,
    character_query: Query<&Transform, With<Character>>,
    mut container_query: Query<(&Transform, &mut Container)>,
    mut bucket_content: ResMut<BucketContent>,
    mut level_complete_writer: MessageWriter<LevelCompleteMessage>,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };

    // Only check when Use is just pressed
    if !action_state.just_pressed(&Action::Use) {
        return;
    };

    // Can only pour if bucket has water
    if *bucket_content != BucketContent::Water {
        return;
    }

    let Ok(character_transform) = character_query.single() else {
        return;
    };

    let character_pos = character_transform.translation;

    // Check if there's a container nearby
    for (container_transform, mut container) in &mut container_query {
        let container_pos = container_transform.translation;
        let distance = character_pos.distance(container_pos);

        if distance <= INTERACTION_RANGE {
            // Pour water into container
            container.fill();
            *bucket_content = BucketContent::Empty;
            info!(
                "Poured water into container! Container is now {:?}",
                container.state
            );

            // Check if container is full (level complete!)
            if container.is_full() {
                info!("Container is full! Level complete!");
                level_complete_writer.write(LevelCompleteMessage);
            }

            return; // Only interact with one container at a time
        }
    }
}

/// System to handle interacting with fires (extinguish or melt snow)
fn interact_with_fire(
    action_query: Query<&ActionState<Action>>,
    character_query: Query<&Transform, With<Character>>,
    mut fire_query: Query<(&Transform, &mut Fire)>,
    mut bucket_content: ResMut<BucketContent>,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };

    // Only check when Use is just pressed
    if !action_state.just_pressed(&Action::Use) {
        return;
    }

    let Ok(character_transform) = character_query.single() else {
        return;
    };

    let character_pos = character_transform.translation;

    // Check if there's a fire nearby
    for (fire_transform, mut fire) in &mut fire_query {
        let fire_pos = fire_transform.translation;
        let distance = character_pos.distance(fire_pos);

        if distance <= INTERACTION_RANGE {
            // Handle different bucket contents
            match *bucket_content {
                BucketContent::Water => {
                    // Extinguish fire with water
                    if fire.is_active() {
                        fire.extinguish();
                        *bucket_content = BucketContent::Empty;
                        info!("Extinguished fire!");
                        return; // Only interact with one fire at a time
                    }
                }
                BucketContent::Snow => {
                    // Melt snow to water at fire (only if fire is active)
                    if fire.is_active() {
                        *bucket_content = BucketContent::Water;
                        info!("Melted snow into water!");
                        return; // Only interact with one fire at a time
                    }
                }
                BucketContent::Empty => {
                    // Can't do anything with empty bucket
                }
            }
        }
    }
}

/// System to handle picking up snow
fn interact_with_snow(
    action_query: Query<&ActionState<Action>>,
    character_query: Query<&Transform, With<Character>>,
    snow_query: Query<&Transform, With<Snow>>,
    mut bucket_content: ResMut<BucketContent>,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };

    // Only check when Use is just pressed
    if !action_state.just_pressed(&Action::Use) {
        return;
    }

    let Ok(character_transform) = character_query.single() else {
        return;
    };

    let character_pos = character_transform.translation;

    // Check if there's snow nearby
    for snow_transform in &snow_query {
        let snow_pos = snow_transform.translation;
        let distance = character_pos.distance(snow_pos);

        if distance <= INTERACTION_RANGE {
            // Fill bucket with snow
            if *bucket_content != BucketContent::Water {
                *bucket_content = BucketContent::Snow;
                info!("Picked up snow!");
            }
            return; // Only interact with one snow source at a time
        }
    }
}

/// System to handle player touching active fire (reset to spawn point)
fn touch_active_fire(
    mut fire_query: Query<(&Transform, &mut Fire), Without<Character>>,
    spawn_point: Res<PlayerSpawnPoint>,
    mut character_transform_query: Query<&mut Transform, With<Character>>,
    mut bucket_content: ResMut<BucketContent>,
) {
    let Ok(character_transform) = character_transform_query.single() else {
        return;
    };

    let character_pos = character_transform.translation;

    // Check if player is touching any active fire
    for (fire_transform, fire) in &fire_query {
        if !fire.is_active() {
            continue;
        }

        let fire_pos = fire_transform.translation;
        let distance = character_pos.distance(fire_pos);

        // Use a smaller distance for actual collision (about 1 tile)
        const TOUCH_DISTANCE: f32 = 20.0;

        if distance <= TOUCH_DISTANCE {
            // Reset player to spawn point
            if let Ok(mut transform) = character_transform_query.single_mut() {
                transform.translation = spawn_point.position;

                // Reset all fires to Active state
                for (_, mut fire) in &mut fire_query {
                    fire.ignite();
                }

                // Reset bucket contents
                *bucket_content = BucketContent::Empty;

                info!("Player touched fire! Resetting to spawn point and level state.");
            }
            return; // Only handle one fire collision per frame
        }
    }
}
