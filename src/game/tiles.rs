#![allow(dead_code)]
use derive_more::From;

use crate::game::Season;
use bevy::prelude::*;

pub const TILE_SIZE: f32 = 32.0;

pub const TILESET_COLUMNS: u32 = 8;
pub const TILESET_ROWS: u32 = 3;

/// Terrain tiles on the primary grid that should generate dual tiles
#[derive(Component, Clone, Copy, Debug)]
pub enum TerrainTile {
    Grass,
    Dirt,
}

/// Grid alignment mode - primary grid or dual grid (offset by 0.5, 0.5)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridAlignment {
    /// Aligned to primary grid at integer coordinates (for game objects)
    Primary,
    /// Aligned to dual grid, offset by (0.5, 0.5) (for corners/edges)
    Dual,
}

/// Position on the grid system
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, From)]
pub struct CornerMask(u8);

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
}

/// Dual tile variant types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum DualVariant {
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
#[derive(Resource)]
pub struct TilesetAtlases {
    pub layout: Handle<TextureAtlasLayout>,
    pub summer: Handle<Image>,
    pub autumn: Handle<Image>,
    pub winter: Handle<Image>,
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

/// System to sync grid positions to transform components
pub fn sync_grid_to_transform(
    mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
) {
    for (grid_pos, mut transform) in &mut query {
        let world_pos = grid_pos.to_world(TILE_SIZE);
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}

/// Calculate corner mask for a dual grid position based on neighboring primary grid tiles
pub fn calculate_corner_mask(
    dual_x: i32,
    dual_y: i32,
    has_tile: impl Fn(i32, i32) -> bool,
) -> CornerMask {
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
