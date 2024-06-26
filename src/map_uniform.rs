use bevy::{
    math::{vec2, vec3, Vec3Swizzles},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
};

#[derive(ShaderType, Clone, Debug, Reflect, AsBindGroup)]
pub struct MapUniform {
    /// Size of the map, in tiles.
    /// Will be derived from underlying map texture.
    pub(crate) map_size: UVec2,

    /// Size of the tile atlas, in pixels.
    /// Will be derived from the tile atlas texture.
    pub(crate) atlas_size: Vec2,

    /// Size of each tile, in pixels.
    pub(crate) tile_size: Vec2,

    /// Relative anchor point position in a tile (in [0..1]^2)
    pub(crate) tile_anchor_point: Vec2,

    /// fractional 2d map index -> projected 2d "map index"
    pub(crate) projection: Mat3,

    pub(crate) global_transform_matrix: Mat3,
    pub(crate) global_transform_translation: Vec3,

    /// (derived) Size of the map in world units necessary to display
    /// all tiles according to projection.
    pub(crate) world_size: Vec2,

    /// (derived) Offset of the projected map in world coordinates
    pub(crate) world_offset: Vec2,

    /// (derived)
    pub(crate) n_tiles: UVec2,

    /// (derived) global world pos -> fractional 2d map index
    pub(crate) global_inverse_transform_matrix: Mat3,
    pub(crate) global_inverse_transform_translation: Vec3,
}

impl Default for MapUniform {
    fn default() -> Self {
        Self {
            map_size: default(),
            atlas_size: default(),
            tile_size: default(),
            tile_anchor_point: vec2(0.0, 0.0),
            projection: crate::PROJECTION,
            global_transform_matrix: default(),
            global_transform_translation: default(),
            world_size: default(),
            world_offset: default(),
            n_tiles: default(),
            global_inverse_transform_matrix: default(),
            global_inverse_transform_translation: default(),
        }
    }
}

impl MapUniform {
    pub(crate) fn map_size(&self) -> UVec2 {
        self.map_size
    }

    pub(crate) fn world_size(&self) -> Vec2 {
        self.world_size
    }

    pub(crate) fn map_to_local(&self, map_position: Vec3) -> Vec3 {
        (self.projection * map_position) * self.tile_size.extend(1.0)
            + self.world_offset.extend(0.0)
    }

    pub(crate) fn map_to_world(&self, map_position: Vec3) -> Vec3 {
        let local = self.map_to_local(map_position);
        self.global_transform_matrix * local + self.global_inverse_transform_translation
    }

    /// As of now, this will ignore `world`s z coordinate
    /// and always project to z=0 on the map.
    /// This behavior might change in the future
    pub(crate) fn local_to_map(&self, local: Vec3) -> Vec3 {
        (crate::INVERSE_PROJECTION * ((local.xy() - self.world_offset) / self.tile_size))
            .extend(0.0)
    }

    pub(crate) fn world_to_map(&self, world: Vec3) -> Vec3 {
        let local = self.global_inverse_transform_matrix * world
            + self.global_inverse_transform_translation;
        self.local_to_map(local)
    }

    pub(crate) fn update_world_size(&mut self) {
        // World Size
        //
        // Determine the bounding rectangle of the projected map (in order to construct the quad
        // that will hold the texture).
        //
        // There is probably a more elegant way to do this, but this
        // works and is simple enough:
        // 1. save coordinates for all 4 corners
        // 2. take maximum x- and y distances
        let mut low = self.map_to_local(vec3(0.0, 0.0, 0.0)).xy();
        let mut high = low;
        for corner in [
            vec2(self.map_size().x as f32, 0.0),
            vec2(0.0, self.map_size().y as f32),
            vec2(self.map_size().x as f32, self.map_size().y as f32),
        ] {
            let pos = self.map_to_local(corner.extend(0.0)).xy();
            low = low.min(pos);
            high = high.max(pos);
        }
        self.world_size = high - low;

        // World offset
        //
        // `map.projection` keeps the map coordinate (0, 0) at the world coordinate (0, 0).
        // However after projection we may want the (0, 0) tile to map to a different position than
        // say the top left corner (eg for an iso projection it might be vertically centered).
        // We use `low` from above to figure out how to correctly translate here.
        self.world_offset = vec2(-0.5, -0.5) * self.world_size - low;
    }

    /// Return true iff this update made the uniform ready
    /// (ie. it was not ready before and is ready now).
    pub(crate) fn update_atlas_size(&mut self, atlas_size: Vec2) -> bool {
        if self.atlas_size == atlas_size {
            return false;
        }

        self.atlas_size = atlas_size;
        self.update_n_tiles();
        true
    }

    pub(crate) fn _apply_transform(&mut self, transform: GlobalTransform) {
        let affine = transform.compute_transform().compute_affine();
        self.global_transform_matrix = affine.matrix3.into();
        self.global_transform_translation = affine.translation.into();

        let inverse = affine.inverse();
        self.global_inverse_transform_matrix = inverse.matrix3.into();
        self.global_inverse_transform_translation = inverse.translation.into();
    }

    fn update_n_tiles(&mut self) {
        let inner = self.atlas_size;
        let n_tiles = inner / self.tile_size;

        let eps = 0.01;
        if (n_tiles.x - n_tiles.x.round()).abs() > eps
            || (n_tiles.y - n_tiles.y.round()).abs() > eps
        {
            panic!(
                "Expected an integral number of tiles in your atlas, but computes to be {:?}",
                n_tiles
            );
        }
        self.n_tiles = n_tiles.as_uvec2();
    }
}
