use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use super::character::Character;
use super::controls::Action;
use super::level::BucketContent;
use super::level::objects::{Container, Water};
use crate::PausableSystems;
use crate::screens::Screen;

/// Message triggered when a level is completed
#[derive(Message)]
pub struct LevelCompleteMessage;

pub fn plugin(app: &mut App) {
    app.add_message::<LevelCompleteMessage>();
    app.add_systems(
        Update,
        (interact_with_water, interact_with_container)
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
