use derive_more::From;

use crate::{PausableSystems, asset_tracking::LoadResource, game::Season, screens::Screen};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

pub const TILE_SIZE: f32 = 32.0;

pub const TILESET_COLUMNS: u32 = 8;
pub const TILESET_ROWS: u32 = 3;

pub fn plugin(app: &mut App) {
    app.load_resource::<TilesetAtlases>();
    app.add_systems(
        Update,
        (
            update_dual_tiles,
            sync_grid_to_transform,
            spawn_terrain_on_click,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Terrain tiles on the primary grid that should generate dual tiles
#[derive(
    Component, Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect, Resource,
)]
pub enum TerrainTile {
    #[default]
    Grass,
    Dirt,
}

/// Grid alignment mode - primary grid or dual grid (offset by 0.5, 0.5)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash, Reflect)]
pub enum GridAlignment {
    /// Aligned to primary grid at integer coordinates (for game objects)
    #[default]
    Primary,
    /// Aligned to dual grid, offset by (0.5, 0.5) (for corners/edges)
    Dual,
}

/// Position on the grid system
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash, Reflect)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
    pub alignment: GridAlignment,
}

impl GridPosition {
    /// Convert grid position to world coordinates
    pub fn to_world(self, tile_size: f32) -> Vec2 {
        let offset = match self.alignment {
            GridAlignment::Primary => 0.0,
            GridAlignment::Dual => 0.5,
        };
        Vec2::new(
            (self.x as f32 + offset) * tile_size,
            (self.y as f32 + offset) * tile_size,
        )
    }

    /// Create a primary grid position
    pub fn primary(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            alignment: GridAlignment::Primary,
        }
    }

    /// Create a dual grid position
    pub fn dual(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            alignment: GridAlignment::Dual,
        }
    }
}

/// Corner flags for tile autotiling
/// Each bit represents whether a tile exists at one of the 4 adjacent primary grid cells
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Ord, PartialOrd, From, Hash, Reflect)]
pub struct CornerMask(u8);

#[allow(dead_code)]
impl CornerMask {
    pub const NORTH_WEST: u8 = 0b1000;
    pub const NORTH_EAST: u8 = 0b0100;
    pub const SOUTH_EAST: u8 = 0b0010;
    pub const SOUTH_WEST: u8 = 0b0001;

    pub const EMPTY: Self = Self(0b0000);
    pub const FULL: Self = Self(0b1111);

    /// Create a new corner mask from the 4 corner flags
    pub fn new(nw: bool, ne: bool, se: bool, sw: bool) -> Self {
        let mut mask = 0;
        if nw {
            mask |= Self::NORTH_WEST;
        }
        if ne {
            mask |= Self::NORTH_EAST;
        }
        if se {
            mask |= Self::SOUTH_EAST;
        }
        if sw {
            mask |= Self::SOUTH_WEST;
        }
        Self(mask)
    }

    /// Create from raw byte value
    pub fn from_bits(bits: u8) -> Self {
        Self(bits & 0b1111)
    }

    /// Get the raw byte value (0-15)
    pub fn bits(&self) -> u8 {
        self.0
    }

    /// Check if a specific corner is set
    pub fn has_corner(&self, corner: u8) -> bool {
        (self.0 & corner) != 0
    }

    /// Count number of corners set
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    /// Calculate corner mask for a dual grid position based on neighboring primary grid tiles
    pub fn calculate(dual_x: i32, dual_y: i32, has_tile: impl Fn(i32, i32) -> bool) -> CornerMask {
        // A dual grid position at (dual_x, dual_y) is surrounded by 4 primary grid cells:
        // NW: (dual_x - 1, dual_y)
        // NE: (dual_x, dual_y)
        // SW: (dual_x - 1, dual_y - 1)
        // SE: (dual_x, dual_y - 1)

        let nw = has_tile(dual_x - 1, dual_y);
        let ne = has_tile(dual_x, dual_y);
        let sw = has_tile(dual_x - 1, dual_y - 1);
        let se = has_tile(dual_x, dual_y - 1);

        CornerMask::new(nw, ne, se, sw)
    }
}

/// Dual tile variant types
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash, Reflect)]
pub enum DualVariant {
    #[default]
    Dirt,
    Grass,
    DirtToGrass,
    GrassToDirt,
    Rock,
    Stalactite,
}

/// Dual grid tile component - represents a tile on the dual grid
#[derive(Component, Clone, Copy, Debug)]
pub struct DualTile {
    /// Which corners have tiles adjacent to them
    pub corner_mask: CornerMask,
    /// Which variant to use
    pub variant: DualVariant,
}

impl DualTile {
    /// Create a new dual tile with the given corner configuration and variant
    pub fn new(corner_mask: CornerMask, variant: DualVariant) -> Self {
        Self {
            corner_mask,
            variant,
        }
    }

    /// Get the tileset index for this tile, if it exists
    pub fn tileset_index(&self) -> Option<usize> {
        find_tile_index(self.corner_mask.bits(), self.variant)
    }

    /// Get texture atlas index
    pub fn atlas_index(&self) -> Option<usize> {
        self.tileset_index()
    }
}

#[rustfmt::skip]
pub const DUAL_TILESET: [DualTile; 24] = [
    DualTile { corner_mask: CornerMask(0b0110), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b1011), variant: DualVariant::Grass },
    DualTile { corner_mask: CornerMask(0b0011), variant: DualVariant::Grass },
    DualTile { corner_mask: CornerMask(0b0011), variant: DualVariant::GrassToDirt },
    DualTile { corner_mask: CornerMask(0b0111), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b1001), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b0010), variant: DualVariant::Grass },
    DualTile { corner_mask: CornerMask(0b0001), variant: DualVariant::Grass },

    DualTile { corner_mask: CornerMask(0b0110), variant: DualVariant::Rock },
    DualTile { corner_mask: CornerMask(0b1111), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b1101), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b1100), variant: DualVariant::Stalactite },
    DualTile { corner_mask: CornerMask(0b1100), variant: DualVariant::Stalactite },
    DualTile { corner_mask: CornerMask(0b1010), variant: DualVariant::Grass },
    DualTile { corner_mask: CornerMask(0b0111), variant: DualVariant::Grass },
    DualTile { corner_mask: CornerMask(0b1001), variant: DualVariant::Rock },

    DualTile { corner_mask: CornerMask(0b0100), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b1110), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b1011), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b0011), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b0011), variant: DualVariant::DirtToGrass },
    DualTile { corner_mask: CornerMask(0b0101), variant: DualVariant::Grass },
    DualTile { corner_mask: CornerMask(0b1100), variant: DualVariant::Dirt },
    DualTile { corner_mask: CornerMask(0b1000), variant: DualVariant::Dirt },
];

/// Find the tileset index for a given corner mask and variant
pub fn find_tile_index(corner_mask: u8, variant: DualVariant) -> Option<usize> {
    DUAL_TILESET
        .iter()
        .enumerate()
        .find(|(_, def)| def.corner_mask == corner_mask.into() && def.variant == variant)
        .map(|(index, _)| index)
}

/// Get all available variants for a given corner mask
pub fn get_variants_for_mask(corner_mask: u8) -> Vec<DualVariant> {
    let mut variants = Vec::new();
    for def in &DUAL_TILESET {
        if def.corner_mask == corner_mask.into() && !variants.contains(&def.variant) {
            variants.push(def.variant);
        }
    }
    variants
}

/// Resource to hold season tileset texture atlases
#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TilesetAtlases {
    #[dependency]
    pub layout: Handle<TextureAtlasLayout>,
    #[dependency]
    pub summer: Handle<Image>,
    #[dependency]
    pub autumn: Handle<Image>,
    #[dependency]
    pub winter: Handle<Image>,
}

impl FromWorld for TilesetAtlases {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(TILE_SIZE as u32, TILE_SIZE as u32),
            TILESET_COLUMNS,
            TILESET_ROWS,
            None,
            None,
        );
        let layout = assets.add(layout);
        Self {
            layout,
            summer: assets.load("images/tiles/summer.epng"),
            autumn: assets.load("images/tiles/autumn.epng"),
            winter: assets.load("images/tiles/winter.epng"),
        }
    }
}

impl TilesetAtlases {
    /// Get the texture handle for a specific season
    pub fn get_texture(&self, season: Season) -> Handle<Image> {
        match season {
            Season::Summer => self.summer.clone(),
            Season::Autumn => self.autumn.clone(),
            Season::Winter => self.winter.clone(),
        }
    }
}

/// Sync grid positions to transform components
pub fn sync_grid_to_transform(
    mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
) {
    for (grid_pos, mut transform) in &mut query {
        let world_pos = grid_pos.to_world(TILE_SIZE);
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}

/// Automatically generates/updates dual tiles based on terrain tiles
pub fn update_dual_tiles(
    mut commands: Commands,
    terrain_query: Query<(Ref<TerrainTile>, Ref<GridPosition>), Without<DualTile>>,
    dual_query: Query<(Entity, &GridPosition), With<DualTile>>,
    tileset_atlases: If<Res<TilesetAtlases>>,
    season: Res<Season>,
) {
    // Check if any terrain tiles changed
    let has_changes = terrain_query
        .iter()
        .any(|(tile, pos)| tile.is_changed() || pos.is_changed());

    if !has_changes {
        return;
    }

    // Build a spatial map of all terrain tiles for fast lookups
    let mut terrain_map: HashMap<(i32, i32), TerrainTile> = HashMap::new();
    let mut dual_updated = HashSet::new();

    for (terrain, pos) in terrain_query.iter() {
        if pos.alignment == GridAlignment::Primary {
            terrain_map.insert((pos.x, pos.y), *terrain);

            // If this terrain tile changed, mark its 4 adjacent dual corners for update
            if pos.is_changed() {
                // Each terrain tile at (x, y) affects 4 dual grid positions:
                // (x, y), (x+1, y), (x, y+1), (x+1, y+1)
                dual_updated.insert((pos.x, pos.y));
                dual_updated.insert((pos.x + 1, pos.y));
                dual_updated.insert((pos.x, pos.y + 1));
                dual_updated.insert((pos.x + 1, pos.y + 1));
            }
        }
    }

    // Remove existing dual tiles at positions that need updating
    for (entity, pos) in dual_query.iter() {
        if pos.alignment == GridAlignment::Dual && dual_updated.contains(&(pos.x, pos.y)) {
            commands.entity(entity).despawn();
        }
    }

    // Regenerate dual tiles at marked positions
    for (dual_x, dual_y) in dual_updated {
        let corner_mask =
            CornerMask::calculate(dual_x, dual_y, |x, y| terrain_map.contains_key(&(x, y)));

        // Only spawn a dual tile if at least one corner is filled
        if corner_mask.count() == 0 {
            continue;
        }

        let sw_terrain = terrain_map.get(&(dual_x - 1, dual_y - 1));
        let se_terrain = terrain_map.get(&(dual_x, dual_y - 1));

        let variants = get_variants_for_mask(corner_mask.bits());

        let variant = match (sw_terrain, se_terrain) {
            (Some(TerrainTile::Grass), Some(TerrainTile::Grass)) => DualVariant::Grass,
            (None, Some(TerrainTile::Grass)) => DualVariant::Grass,
            (Some(TerrainTile::Grass), None) => DualVariant::Grass,
            (Some(TerrainTile::Dirt), Some(TerrainTile::Grass)) => DualVariant::GrassToDirt,
            (Some(TerrainTile::Grass), Some(TerrainTile::Dirt)) => DualVariant::DirtToGrass,
            _ => DualVariant::Dirt,
        };
        let variant = if variants.contains(&variant) {
            variant
        } else {
            DualVariant::Dirt
        };

        let dual_tile = DualTile::new(corner_mask, variant);

        if let Some(atlas_index) = dual_tile.atlas_index() {
            let position = GridPosition::dual(dual_x, dual_y);
            let world_pos = position.to_world(TILE_SIZE);

            commands.spawn((
                Name::new(format!(
                    "DualTile {:?} at ({}, {})",
                    variant, dual_x, dual_y
                )),
                position,
                dual_tile,
                Sprite::from_atlas_image(
                    tileset_atlases.get_texture(*season),
                    TextureAtlas {
                        layout: tileset_atlases.layout.clone(),
                        index: atlas_index,
                    },
                ),
                Transform::from_translation(world_pos.extend(0.0)),
                DespawnOnExit(Screen::Gameplay),
            ));
        }
    }
}

/// Spawn or remove terrain tiles on mouse click
pub fn spawn_terrain_on_click(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::pixel_camera::MainCamera>>,
    window_query: Query<&Window>,
    terrain_query: Query<(Entity, &GridPosition), With<TerrainTile>>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left)
        && !mouse_buttons.just_pressed(MouseButton::Right)
    {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let grid_x = (world_pos.x / TILE_SIZE).floor() as i32;
    let grid_y = (world_pos.y / TILE_SIZE).floor() as i32;
    let grid_pos = GridPosition::primary(grid_x, grid_y);

    // Left click: spawn grass, Right click: remove
    if mouse_buttons.just_pressed(MouseButton::Left) {
        // Check if there's already a terrain tile at this position
        let tile_exists = terrain_query.iter().any(|(_, pos)| *pos == grid_pos);

        if !tile_exists {
            commands.spawn((
                Name::new(format!("TerrainTile Grass at ({}, {})", grid_x, grid_y)),
                grid_pos,
                TerrainTile::Grass,
            ));
        }
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        // Remove terrain tile at this position
        for (entity, pos) in terrain_query.iter() {
            if *pos == grid_pos {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corner_mask() {
        let mask = CornerMask::new(true, true, false, false);
        assert_eq!(mask.bits(), 0b1100);
        assert_eq!(mask.count(), 2);
        assert!(mask.has_corner(CornerMask::NORTH_WEST));
        assert!(mask.has_corner(CornerMask::NORTH_EAST));
        assert!(!mask.has_corner(CornerMask::SOUTH_WEST));
    }

    #[test]
    fn test_grid_position_conversion() {
        let primary = GridPosition::primary(2, 3);
        let world = primary.to_world(16.0);
        assert_eq!(world, Vec2::new(32.0, 48.0));

        let dual = GridPosition::dual(2, 3);
        let world = dual.to_world(16.0);
        assert_eq!(world, Vec2::new(40.0, 56.0));
    }

    #[test]
    fn test_dual_tile_index() {
        let tile = DualTile::new(CornerMask::from_bits(0b1001), DualVariant::Dirt);
        let index = tile.tileset_index();
        assert_eq!(index, Some(5));
    }
}
