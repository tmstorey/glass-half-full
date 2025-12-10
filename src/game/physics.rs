use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use super::character::{
    AnimationState, Character, CharacterAnimation, Direction, OneShotAnimation,
};
use super::controls::Action;
use super::level::PlayerSpawnPoint;
use super::tiles::{GridPosition, TILE_SIZE, TerrainTile};
use crate::PausableSystems;
use crate::pixel_camera::PixelCamera;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.register_type::<Velocity>();
    app.register_type::<CharacterController>();

    app.add_systems(
        Update,
        (
            character_movement,
            apply_gravity,
            apply_velocity,
            character_collision,
            decrement_oneshot_animation,
            update_character_animation,
            camera_follow_player,
            respawn_on_fall,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay))
            .chain(),
    );
}

/// Velocity component for physics
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Character controller configuration
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterController {
    pub max_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub jump_strength: f32,
    pub gravity: f32,
    pub is_grounded: bool,
    pub hitbox_width: f32,
    pub hitbox_height: f32,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            max_speed: 200.0,
            acceleration: 800.0,
            friction: 600.0,
            jump_strength: 320.0,
            gravity: 500.0,
            is_grounded: false,
            hitbox_width: 16.0,
            hitbox_height: 95.0,
        }
    }
}

/// System to handle character movement based on input
fn character_movement(
    time: Res<Time>,
    action_query: Query<&ActionState<Action>>,
    mut character_query: Query<
        (&mut Velocity, &mut Direction, &CharacterController),
        With<Character>,
    >,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };

    for (mut velocity, mut direction, controller) in &mut character_query {
        let dt = time.delta_secs();

        // Get horizontal input
        let input = action_state.axis_pair(&Action::Run).x;

        // Update facing direction
        if input > 0.1 {
            *direction = Direction::Right;
        } else if input < -0.1 {
            *direction = Direction::Left;
        }

        // Apply acceleration or friction
        if input.abs() > 0.1 {
            velocity.x += input * controller.acceleration * dt;
            velocity.x = velocity
                .x
                .clamp(-controller.max_speed, controller.max_speed);
        } else {
            // Apply friction
            let friction_force = controller.friction * dt;
            if velocity.x > 0.0 {
                velocity.x = (velocity.x - friction_force).max(0.0);
            } else if velocity.x < 0.0 {
                velocity.x = (velocity.x + friction_force).min(0.0);
            }
        }

        // Handle jumping
        if controller.is_grounded && action_state.just_pressed(&Action::Jump) {
            velocity.y = controller.jump_strength;
        }
    }
}

/// System to apply gravity
fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &CharacterController), With<Character>>,
) {
    for (mut velocity, controller) in &mut query {
        let dt = time.delta_secs();
        velocity.y -= controller.gravity * dt;
    }
}

/// System to apply velocity to transform
fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform), With<Character>>) {
    for (velocity, mut transform) in &mut query {
        let dt = time.delta_secs();
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}

/// System to handle collision with terrain tiles
fn character_collision(
    mut character_query: Query<
        (&mut Transform, &mut Velocity, &mut CharacterController),
        With<Character>,
    >,
    terrain_query: Query<&GridPosition, With<TerrainTile>>,
) {
    for (mut transform, mut velocity, mut controller) in &mut character_query {
        let char_x = transform.translation.x;
        let char_y = transform.translation.y;
        let half_width = controller.hitbox_width / 2.0;
        let half_height = controller.hitbox_height / 2.0;

        // Character bounds
        let char_left = char_x - half_width;
        let char_right = char_x + half_width;
        let char_bottom = char_y - half_height;
        let char_top = char_y + half_height - 50.;

        controller.is_grounded = false;

        // Check collision with each terrain tile
        for grid_pos in terrain_query.iter() {
            let tile_world_pos = grid_pos.to_world(TILE_SIZE);
            let tile_left = tile_world_pos.x;
            let tile_right = tile_world_pos.x + TILE_SIZE * 2.;
            let tile_bottom = tile_world_pos.y;
            let tile_top = tile_world_pos.y + TILE_SIZE;

            // Check if character overlaps with tile
            if char_right > tile_left
                && char_left < tile_right
                && char_top > tile_bottom
                && char_bottom < tile_top
            {
                // Calculate overlap amounts
                let overlap_left = char_right - tile_left;
                let overlap_right = tile_right - char_left;
                let overlap_bottom = char_top - tile_bottom;
                let overlap_top = tile_top - char_bottom;

                // Find the smallest overlap (that's the collision direction)
                let min_overlap = overlap_left
                    .min(overlap_right)
                    .min(overlap_bottom)
                    .min(overlap_top);

                // Resolve collision in the direction of smallest overlap
                if min_overlap == overlap_top && velocity.y <= 0.0 {
                    // Collision from below (character landing on tile)
                    transform.translation.y = tile_top + half_height;
                    velocity.y = 0.0;
                    controller.is_grounded = true;
                } else if min_overlap == overlap_bottom && velocity.y > 0.0 {
                    // Collision from above (character hitting ceiling)
                    transform.translation.y = tile_bottom - half_height + 50.;
                    velocity.y = 0.0;
                } else if min_overlap == overlap_left && velocity.x > 0.0 {
                    // Collision from left
                    transform.translation.x = tile_left - half_width;
                    velocity.x = 0.0;
                } else if min_overlap == overlap_right && velocity.x < 0.0 {
                    // Collision from right
                    transform.translation.x = tile_right + half_width;
                    velocity.x = 0.0;
                }
            }
        }
    }
}

/// System to decrement one-shot animation frame counters
fn decrement_oneshot_animation(
    mut query: Query<(&mut OneShotAnimation, &CharacterAnimation), With<Character>>,
) {
    for (mut oneshot, animation) in &mut query {
        if let Some(frames) = oneshot.frames_remaining
            && animation.just_changed()
            && frames > 0
        {
            oneshot.frames_remaining = Some(frames - 1);
        }
    }
}

/// System to update character animation based on velocity and input
fn update_character_animation(
    action_query: Query<&ActionState<Action>>,
    mut query: Query<
        (
            &Velocity,
            &CharacterController,
            &mut CharacterAnimation,
            &mut OneShotAnimation,
        ),
        With<Character>,
    >,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };

    for (velocity, controller, mut animation, mut oneshot) in &mut query {
        let horizontal_speed = velocity.x.abs();
        let vertical_speed = velocity.y;

        if controller.is_grounded && action_state.just_pressed(&Action::Use) {
            let (_, frame_count, _, _) = AnimationState::Use.get_animation_config();
            oneshot.frames_remaining = Some(frame_count);
        }

        let new_state = if oneshot.frames_remaining.is_some_and(|f| f > 0) {
            AnimationState::Use
        } else if !controller.is_grounded {
            if vertical_speed > 0.0 {
                AnimationState::Jump
            } else {
                AnimationState::Fall
            }
        } else if horizontal_speed > controller.max_speed * 0.7 {
            AnimationState::Run
        } else if horizontal_speed > 10.0 {
            AnimationState::Walk
        } else {
            AnimationState::Idle
        };

        animation.set_state(new_state);
    }
}

/// System to smoothly follow the player character with the camera
fn camera_follow_player(
    time: Res<Time>,
    character_query: Query<&Transform, (With<Character>, Without<PixelCamera>)>,
    mut camera_query: Query<&mut Transform, With<PixelCamera>>,
) {
    let Ok(character_transform) = character_query.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let smoothness = 10.0; // Higher = faster following
    let y_offset = 64.0;
    let dt = time.delta_secs();

    camera_transform.translation.x = camera_transform.translation.x
        + (character_transform.translation.x - camera_transform.translation.x) * smoothness * dt;
    camera_transform.translation.y = camera_transform.translation.y
        + (character_transform.translation.y + y_offset - camera_transform.translation.y)
            * smoothness
            * dt;
    camera_transform.translation.y = camera_transform.translation.y.max(-12.);
}

/// System to respawn the character if they fall off the level
fn respawn_on_fall(
    spawn_point: Res<PlayerSpawnPoint>,
    mut character_query: Query<(&mut Transform, &mut Velocity), With<Character>>,
) {
    let Ok((mut transform, mut velocity)) = character_query.single_mut() else {
        return;
    };

    let fall_threshold = -500.0; // Y-position below which character respawns

    if transform.translation.y < fall_threshold {
        // Reset to spawn position
        transform.translation = spawn_point.position;

        // Reset velocity to prevent continued falling
        velocity.x = 0.0;
        velocity.y = 0.0;
    }
}
