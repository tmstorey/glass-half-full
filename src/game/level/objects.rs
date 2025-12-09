#![allow(dead_code)]

use bevy::prelude::*;

use crate::{PausableSystems, screens::Screen};

pub fn plugin(app: &mut App) {
    app.register_type::<Fire>();
    app.register_type::<FireState>();
    app.register_type::<FireAnimation>();
    app.register_type::<Snow>();
    app.register_type::<Platform>();
    app.register_type::<Water>();
    app.register_type::<WaterType>();
    app.register_type::<WaterAnimation>();
    app.register_type::<Container>();
    app.register_type::<ContainerState>();

    app.add_systems(
        Update,
        (
            update_fire_animation,
            sync_fire_animation,
            update_fire_state,
            update_water_animation,
            sync_water_animation,
            update_container_state,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Component representing a fire that can be active or extinguished
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Fire {
    pub state: FireState,
}

/// The state of a fire
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum FireState {
    Active,
    Extinguished,
}

impl Fire {
    pub fn new(state: FireState) -> Self {
        Self { state }
    }

    pub fn is_active(&self) -> bool {
        self.state == FireState::Active
    }

    pub fn extinguish(&mut self) {
        self.state = FireState::Extinguished;
    }

    pub fn ignite(&mut self) {
        self.state = FireState::Active;
    }
}

/// Animation component for fire sprites
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FireAnimation {
    timer: Timer,
    current_frame: usize,
    frame_count: usize,
}

impl FireAnimation {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
            frame_count: 40, // 5 columns � 8 rows
        }
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
        if self.timer.just_finished() {
            self.current_frame = (self.current_frame + 1) % self.frame_count;
        }
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn just_changed(&self) -> bool {
        self.timer.just_finished()
    }
}

/// Spawns a fire at the specified position
pub fn spawn_fire(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
    state: FireState,
) -> Entity {
    let texture = match state {
        FireState::Active => asset_server.load("images/objects/fire.epng"),
        FireState::Extinguished => asset_server.load("images/objects/fire-extinguished.epng"),
    };

    let layout = match state {
        FireState::Active => TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            5, // columns
            8, // rows
            None,
            None,
        ),
        FireState::Extinguished => TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            1, // single tile
            1,
            None,
            None,
        ),
    };

    let mut entity_commands = commands.spawn((
        Name::new("Fire"),
        Fire::new(state),
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: asset_server.add(layout),
                index: 0,
            }),
            ..default()
        },
        Transform::from_translation(position),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
    ));

    // Only add animation component for active fires
    if state == FireState::Active {
        entity_commands.insert(FireAnimation::new());
    }

    entity_commands.id()
}

/// System to update fire animations
pub fn update_fire_animation(time: Res<Time>, mut query: Query<&mut FireAnimation>) {
    for mut animation in &mut query {
        animation.update(time.delta());
    }
}

/// System to sync fire sprite with animation state
pub fn sync_fire_animation(mut query: Query<(&FireAnimation, &mut Sprite), With<Fire>>) {
    for (animation, mut sprite) in &mut query {
        if animation.just_changed()
            && let Some(atlas) = sprite.texture_atlas.as_mut()
        {
            atlas.index = animation.current_frame();
        }
    }
}

/// System to update fire sprite when state changes
pub fn update_fire_state(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut query: Query<(Entity, &Fire, &mut Sprite), Changed<Fire>>,
) {
    for (entity, fire, mut sprite) in &mut query {
        let texture = match fire.state {
            FireState::Active => asset_server.load("images/objects/fire.epng"),
            FireState::Extinguished => asset_server.load("images/objects/fire-extinguished.epng"),
        };

        let layout = match fire.state {
            FireState::Active => TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 8, None, None),
            FireState::Extinguished => {
                TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None)
            }
        };

        sprite.image = texture;
        sprite.texture_atlas = Some(TextureAtlas {
            layout: asset_server.add(layout),
            index: 0,
        });

        // Add or remove animation component based on state
        if fire.state == FireState::Active {
            commands.entity(entity).insert(FireAnimation::new());
        } else {
            commands.entity(entity).remove::<FireAnimation>();
        }
    }
}

/// Marker component for snow objects
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Snow;

/// Spawns a snow sprite at the specified position
pub fn spawn_snow(commands: &mut Commands, asset_server: &AssetServer, position: Vec3) -> Entity {
    commands
        .spawn((
            Name::new("Snow"),
            Snow,
            Sprite {
                image: asset_server.load("images/objects/snow.epng"),
                custom_size: Some(Vec2::new(64.0, 32.0)),
                ..default()
            },
            Transform::from_translation(position),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id()
}

/// Marker component for platform objects
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Platform;

/// Spawns a platform sprite at the specified position
pub fn spawn_platform(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
) -> Entity {
    commands
        .spawn((
            Name::new("Platform"),
            Platform,
            Sprite {
                image: asset_server.load("images/objects/platform.epng"),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_translation(position),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id()
}

/// Component representing a water object
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Water {
    pub water_type: WaterType,
}

/// The type of water object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum WaterType {
    WaterLeft,
    WaterRight,
    WaterMiddle,
    WaterfallTop,
    WaterfallMiddle,
    WaterfallLower,
    WaterfallBase,
}

impl WaterType {
    /// Returns the row index for this water type in the sprite sheet
    pub fn row_index(&self) -> usize {
        match self {
            WaterType::WaterLeft => 0,
            WaterType::WaterRight => 1,
            WaterType::WaterMiddle => 2,
            WaterType::WaterfallTop => 3,
            WaterType::WaterfallMiddle => 4,
            WaterType::WaterfallLower => 5,
            WaterType::WaterfallBase => 6,
        }
    }
}

impl Water {
    pub fn new(water_type: WaterType) -> Self {
        Self { water_type }
    }
}

/// Animation component for water sprites
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct WaterAnimation {
    timer: Timer,
    current_frame: usize,
    frame_count: usize,
    row: usize,
}

impl WaterAnimation {
    pub fn new(water_type: WaterType) -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
            frame_count: 20, // 20 columns
            row: water_type.row_index(),
        }
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
        if self.timer.just_finished() {
            self.current_frame = (self.current_frame + 1) % self.frame_count;
        }
    }

    pub fn current_atlas_index(&self) -> usize {
        self.row * self.frame_count + self.current_frame
    }

    pub fn just_changed(&self) -> bool {
        self.timer.just_finished()
    }
}

/// Spawns a water object at the specified position
pub fn spawn_water(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
    water_type: WaterType,
) -> Entity {
    let texture = asset_server.load("images/objects/water.epng");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        20, // columns
        7,  // rows
        None,
        Some(UVec2::splat(1)),
    );

    commands
        .spawn((
            Name::new(format!("Water {:?}", water_type)),
            Water::new(water_type),
            WaterAnimation::new(water_type),
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas {
                    layout: asset_server.add(layout),
                    index: water_type.row_index() * 20, // Start at first frame of the row
                }),
                ..default()
            },
            Transform::from_translation(position),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id()
}

/// System to update water animations
pub fn update_water_animation(time: Res<Time>, mut query: Query<&mut WaterAnimation>) {
    for mut animation in &mut query {
        animation.update(time.delta());
    }
}

/// System to sync water sprite with animation state
pub fn sync_water_animation(mut query: Query<(&WaterAnimation, &mut Sprite), With<Water>>) {
    for (animation, mut sprite) in &mut query {
        if animation.just_changed()
            && let Some(atlas) = sprite.texture_atlas.as_mut()
        {
            atlas.index = animation.current_atlas_index();
        }
    }
}

/// Component representing a container with different fill states
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Container {
    pub state: ContainerState,
}

/// The fill state of a container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ContainerState {
    Empty,
    HalfFull,
    Full,
}

impl ContainerState {
    /// Returns the column index for this container state in the sprite sheet
    pub fn column_index(&self) -> usize {
        match self {
            ContainerState::Empty => 0,
            ContainerState::HalfFull => 1,
            ContainerState::Full => 2,
        }
    }
}

impl Container {
    pub fn new(state: ContainerState) -> Self {
        Self { state }
    }

    pub fn is_empty(&self) -> bool {
        self.state == ContainerState::Empty
    }

    pub fn is_full(&self) -> bool {
        self.state == ContainerState::Full
    }

    pub fn fill(&mut self) {
        self.state = match self.state {
            ContainerState::Empty => ContainerState::HalfFull,
            ContainerState::HalfFull => ContainerState::Full,
            ContainerState::Full => ContainerState::Full,
        };
    }

    pub fn empty(&mut self) {
        self.state = match self.state {
            ContainerState::Empty => ContainerState::Empty,
            ContainerState::HalfFull => ContainerState::Empty,
            ContainerState::Full => ContainerState::HalfFull,
        };
    }
}

/// Spawns a container at the specified position
pub fn spawn_container(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
    state: ContainerState,
) -> Entity {
    let texture = asset_server.load("images/objects/container.epng");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        3, // columns
        1, // row
        None,
        None,
    );

    commands
        .spawn((
            Name::new(format!("Container {:?}", state)),
            Container::new(state),
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas {
                    layout: asset_server.add(layout),
                    index: state.column_index(),
                }),
                ..default()
            },
            Transform::from_translation(position),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id()
}

/// System to update container sprite when state changes
pub fn update_container_state(
    asset_server: Res<AssetServer>,
    mut query: Query<(&Container, &mut Sprite), Changed<Container>>,
) {
    for (container, mut sprite) in &mut query {
        let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 3, 1, None, None);

        sprite.image = asset_server.load("images/objects/container.epng");
        sprite.texture_atlas = Some(TextureAtlas {
            layout: asset_server.add(layout),
            index: container.state.column_index(),
        });
    }
}
