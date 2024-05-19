use bevy::{
    math::{mat3, vec2, vec3, Mat3},
    prelude::*,
};

/// Determines how map coordinates are related to world coordinates.
#[derive(Debug, Clone, Copy)]
pub struct TileProjection {
    /// Projection matrix for converting map coordinates to world coordinates.
    /// This is normalized to the tile dimensions, ie. 1.0 means full tile width/height.
    pub projection: Mat3,

    /// Relative anchor point into a tile.
    /// `(0.0, 0.0)` is top left, `(1.0, 1.0)` is bottom-right
    pub tile_anchor_point: Vec2,
}

/// Default projection that renders every tile as-is in a rectangular grid.
pub const IDENTITY: TileProjection = TileProjection {
    // By default flip y so tiles are rendered right side up
    projection: mat3(
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    ),
    tile_anchor_point: vec2(0.0, 0.0),
};
