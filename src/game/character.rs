#![allow(dead_code)]

use crate::{PausableSystems, screens::Screen};
use bevy::prelude::*;
use std::sync::LazyLock;
use std::time::Duration;
use strum::Display;

/// Component for character facing direction
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub enum Direction {
    Left,
    Right,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Right
    }
}

pub static ROWS: u32 = 7;
pub static COLUMNS: u32 = 10;

pub static HAIR_COLOURS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["blonde", "red", "light", "dark", "black"]);
pub static CLOTHING_COLOURS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["blue", "green", "orange", "purple", "red"]);

pub fn plugin(app: &mut App) {
    app.register_type::<CharacterLayers>();
    app.register_type::<CharacterAnimation>();
    app.register_type::<AnimationState>();
    app.register_type::<Direction>();

    app.add_systems(OnEnter(Screen::Gameplay), spawn_character);

    app.add_systems(Update, (
        update_character_animation_timer,
        sync_layer_animations,
    )
    .in_set(PausableSystems)
    .run_if(in_state(Screen::Gameplay))
    .chain());
}

/// Component that defines which sprite layers make up a character
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct CharacterLayers {
    /// Ordered list of sprite layers (bottom to top)
    pub layers: Vec<CharacterLayer>,
}

impl CharacterLayers {
    /// Creates a new CharacterLayers from a vec of layers.
    pub fn new(mut layers: Vec<CharacterLayer>) -> Self {
        // Ensure Body layer is present
        if !layers.iter().any(|layer| layer.layer_type == LayerType::Body) {
            layers.push(CharacterLayer {
                layer_type: LayerType::Body,
                item_name: None,
                variant: Some(LayerVariant::Variant(5)),
            });
        }

        // Ensure Underclothes layer is present
        if !layers.iter().any(|layer| layer.layer_type == LayerType::Underclothes) {
            layers.push(CharacterLayer {
                layer_type: LayerType::Underclothes,
                item_name: Some("bodice1".to_string()),
                variant: Some(LayerVariant::ClothingColour(0)),
            });
        }

        layers.sort_by_key(|layer| layer.layer_type);

        Self { layers }
    }
}

impl Default for CharacterLayers {
    fn default() -> Self {
        Self {
            layers: vec![
                CharacterLayer {
                    layer_type: LayerType::Body,
                    item_name: None,
                    variant: Some(LayerVariant::Variant(3)),
                },
                CharacterLayer {
                    layer_type: LayerType::Hair,
                    item_name: Some("1".to_string()),
                    variant: Some(LayerVariant::HairColour(4)),
                },
                CharacterLayer {
                    layer_type: LayerType::Underclothes,
                    item_name: Some("bodice1".to_string()),
                    variant: Some(LayerVariant::ClothingColour(0)),
                },
                CharacterLayer {
                    layer_type: LayerType::Footwear,
                    item_name: Some("boots".to_string()),
                    variant: None,
                },
                CharacterLayer {
                    layer_type: LayerType::Clothes,
                    item_name: Some("sleeve-dress".to_string()),
                    variant: None,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Reflect)]
pub enum LayerVariant {
    HairColour(u8),
    ClothingColour(u8),
    Variant(u8),
}

impl LayerVariant {
    pub fn to_path_string(&self) -> String {
        match self {
            _ => format!(""),
        }
    }
}

/// Represents a single sprite layer in the character
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct CharacterLayer {
    /// Type of layer
    pub layer_type: LayerType,
    /// Name of the item
    pub item_name: Option<String>,
    /// Colour or variant (1-5)
    pub variant: Option<LayerVariant>,
}

impl CharacterLayer {
    pub fn texture_path(&self) -> Result<String, ()> {
        let mut path = format!(
            "images/character/{}",
            self.layer_type.to_string().to_lowercase()
        );
        if let Some(item_name) = self.item_name.clone() {
            path.push('/');
            path.push_str(&item_name);
        }
        if let Some(variant) = self.variant {
            path.push('/');
            let variant_string = match variant {
                LayerVariant::HairColour(index) => (*HAIR_COLOURS)[index as usize].to_string(),
                LayerVariant::ClothingColour(index) => {
                    (*CLOTHING_COLOURS)[index as usize].to_string()
                }
                LayerVariant::Variant(number) => number.to_string(),
            };
            path.push_str(&variant_string);
        }
        path.push_str(".epng");
        Ok(path)
    }
}

/// The type of layer. Variants are in bottom-to-top order
#[derive(Clone, Copy, Debug, Display, Reflect, PartialEq, Eq, Ord, PartialOrd)]
pub enum LayerType {
    Cape,
    Body,
    Hair,
    Underclothes,
    Footwear,
    Clothes,
    Gloves,
    Headwear,
}

impl LayerType {
    pub fn available_items(&self) -> Vec<CharacterLayer> {
        let mut items = vec![];
        match self {
            LayerType::Body => {
                for index in 1..=5 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: None,
                        variant: Some(LayerVariant::Variant(index)),
                    });
                }
            }
            LayerType::Cape | LayerType::Gloves => {
                for index in 0..=4 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: None,
                        variant: Some(LayerVariant::ClothingColour(index)),
                    });
                }
            }
            LayerType::Hair => {
                let item_names = vec!["1", "2", "3", "4", "5", "6"];
                for name in item_names {
                    for index in 0..=4 {
                        items.push(CharacterLayer {
                            layer_type: *self,
                            item_name: Some(name.to_string()),
                            variant: Some(LayerVariant::HairColour(index)),
                        });
                    }
                }
            }
            LayerType::Underclothes => {
                items.push(CharacterLayer {
                    layer_type: *self,
                    item_name: Some("armored-corset".to_string()),
                    variant: None,
                });
                let item_names = vec![
                    "bodice1",
                    "bodice2",
                    "bodice3",
                    "corset1",
                    "corset2",
                    "corset3",
                    "corset4",
                    "bikini",
                    "underwear",
                ];
                for name in item_names {
                    for index in 0..=4 {
                        items.push(CharacterLayer {
                            layer_type: *self,
                            item_name: Some(name.to_string()),
                            variant: Some(LayerVariant::ClothingColour(index)),
                        });
                    }
                }
            }
            LayerType::Footwear => {
                items.push(CharacterLayer {
                    layer_type: *self,
                    item_name: Some("boots".to_string()),
                    variant: None,
                });
                for index in 0..=4 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some("socks".to_string()),
                        variant: Some(LayerVariant::ClothingColour(index)),
                    });
                }
                for index in 1..=5 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some("thighhighs".to_string()),
                        variant: Some(LayerVariant::Variant(index)),
                    });
                }
            }
            LayerType::Clothes => {
                for index in 0..=4 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some("dress".to_string()),
                        variant: Some(LayerVariant::ClothingColour(index)),
                    });
                }
                let item_names = vec![
                    "sleeve-dress",
                    "fancy-dress",
                    "queen-dress",
                    "skirt",
                    "short-skirt",
                ];
                for name in item_names {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some(name.to_string()),
                        variant: None,
                    });
                }
            }
            LayerType::Headwear => {
                for index in 0..=4 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some("hat".to_string()),
                        variant: Some(LayerVariant::ClothingColour(index)),
                    });
                }
                for index in 0..=4 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some("cap".to_string()),
                        variant: Some(LayerVariant::ClothingColour(index)),
                    });
                }

                let item_names = vec!["farming-hat", "mining-helmet", "witch-hat", "santa-hat"];
                for name in item_names {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some(name.to_string()),
                        variant: None,
                    });
                }

                for index in 1..=5 {
                    items.push(CharacterLayer {
                        layer_type: *self,
                        item_name: Some("bunnyears".to_string()),
                        variant: Some(LayerVariant::Variant(index)),
                    });
                }
            }
        };
        items
    }
}

/// Tracks animation state for a layered character
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterAnimation {
    timer: Timer,
    current_frame: usize,
    state: AnimationState,
}

#[derive(Reflect, PartialEq, Clone, Copy, Debug, Display)]
pub enum AnimationState {
    Idle,
    Walk,
    Run,
    Jump,
    Fall,
    Attack,
    Death,
}

impl AnimationState {
    /// Returns (row_index, frame_count, frame_duration)
    pub fn get_animation_config(&self) -> (usize, usize, Duration) {
        match self {
            AnimationState::Idle => (0, 5, Duration::from_millis(200)),
            AnimationState::Walk => (1, 8, Duration::from_millis(100)),
            AnimationState::Run => (2, 8, Duration::from_millis(80)),
            AnimationState::Jump => (3, 4, Duration::from_millis(100)),
            AnimationState::Fall => (4, 4, Duration::from_millis(100)),
            AnimationState::Attack => (5, 6, Duration::from_millis(60)),
            AnimationState::Death => (6, 10, Duration::from_millis(100)),
        }
    }
}

impl CharacterAnimation {
    pub fn new(state: AnimationState) -> Self {
        let (_, _, duration) = state.get_animation_config();
        Self {
            timer: Timer::new(duration, TimerMode::Repeating),
            current_frame: 0,
            state,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.just_finished() {
            let (_, frame_count, _) = self.state.get_animation_config();
            self.current_frame = (self.current_frame + 1) % frame_count;
        }
    }

    pub fn set_state(&mut self, new_state: AnimationState) {
        if self.state != new_state {
            let (_, _, duration) = new_state.get_animation_config();
            self.state = new_state;
            self.current_frame = 0;
            self.timer = Timer::new(duration, TimerMode::Repeating);
        }
    }

    pub fn get_atlas_index(&self) -> usize {
        let (row, _, _) = self.state.get_animation_config();
        row * 100 + self.current_frame // Assuming max 100 frames per row
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn state(&self) -> AnimationState {
        self.state
    }

    pub fn just_changed(&self) -> bool {
        self.timer.just_finished()
    }
}

/// Marker component for the root character entity
#[derive(Component)]
pub struct Character;

/// Spawns a layered character with all its sprite layers as children
pub fn spawn_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let animation = CharacterAnimation::new(AnimationState::Idle);
    let character_layers = CharacterLayers::default();
    let position = Vec3::new(5., 5., 0.);

    // Create the root character entity
    let character_id = commands
        .spawn((
            Name::new("Character"),
            Character,
            character_layers.clone(),
            animation,
            Direction::default(),
            Transform::from_translation(position),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    // Spawn each layer as a child sprite
    for layer in character_layers.layers.iter() {
        let texture = asset_server.load(&layer.texture_path().unwrap());

        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(80, 64),
            COLUMNS,
            ROWS,
            None,
            None,
        );

        let layer_entity = commands
            .spawn((
                Name::new(format!("{:?} Layer", layer.layer_type)),
                Sprite {
                    image: texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: asset_server.add(layout),
                        index: 0,
                    }),
                    flip_x: true,
                    ..default()
                },
                layer.clone(),
                Transform::default(),
            ))
            .id();

        commands.entity(character_id).add_child(layer_entity);
    }
}

/// System that updates animation timers
pub fn update_character_animation_timer(
    time: Res<Time>,
    mut query: Query<&mut CharacterAnimation>,
) {
    for mut animation in &mut query {
        animation.update(time.delta());
    }
}

/// System that syncs all layer sprites to the character's current animation frame and direction
pub fn sync_layer_animations(
    character_query: Query<(&CharacterAnimation, &Direction, &Children), With<Character>>,
    mut layer_query: Query<&mut Sprite, With<CharacterLayer>>,
) {
    for (animation, direction, children) in &character_query {
        if !animation.just_changed() {
            continue;
        }

        let (row, _, _) = animation.state().get_animation_config();
        let atlas_index = row * (COLUMNS as usize) + animation.current_frame();

        // Sprites face left in the sheet, so flip when facing right
        let should_flip = *direction == Direction::Right;

        for child in children.iter() {
            if let Ok(mut sprite) = layer_query.get_mut(child) {
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = atlas_index;
                }
                sprite.flip_x = should_flip;
            }
        }
    }
}

/// Example system to change animation based on velocity (you'd integrate this with your movement)
pub fn update_character_animation_from_movement(
    mut _query: Query<(&Transform, &mut CharacterAnimation), With<Character>>,
) {
    // This is a stub - integrate with your actual movement system
    // Example:
    // for (transform, mut animation) in &mut query {
    //     if velocity.length() > RUN_THRESHOLD {
    //         animation.set_state(AnimationState::Run);
    //     } else if velocity.length() > WALK_THRESHOLD {
    //         animation.set_state(AnimationState::Walk);
    //     } else {
    //         animation.set_state(AnimationState::Idle);
    //     }
    // }
}
