use crate::{
    map::{DefaultUserData, Map, MapIndexer},
    map_uniform::MapUniform,
};
use bevy::{
    math::uvec2,
    prelude::*,
    render::render_resource::{encase::internal::WriteInto, AsBindGroup, ShaderSize, ShaderType},
};

/// Builder for constructing a map component. This is usually the preferred way of constructing.
pub struct MapBuilder<UserData = DefaultUserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    map: Map<UserData>,
}

impl<UserData> MapBuilder<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    /// Create a builder for the given map size (number of tiles in each dimension),
    /// the given atlas texture and the tile size (in the atlas).
    pub fn new(map_size: UVec2, atlas_texture: Handle<Image>, tile_size: Vec2) -> Self {
        Self {
            map: Map::<UserData> {
                atlas_texture,
                map_uniform: MapUniform {
                    map_size,
                    tile_size,
                    ..default()
                },
                ..default()
            },
        }
    } // fn new

    /// Create a builder for the given map size (number of tiles in each dimension),
    /// the given atlas texture and the tile size (in the atlas).
    pub fn custom(
        map_size: UVec2,
        atlas_texture: Handle<Image>,
        tile_size: Vec2,
        user_data: UserData,
    ) -> Self {
        Self {
            map: Map::<UserData> {
                atlas_texture,
                map_uniform: MapUniform {
                    map_size,
                    tile_size,
                    ..default()
                },
                user_data,
                ..default()
            },
        }
    } // fn new

    pub fn with_user_data(mut self, new_user_data: UserData) -> Self {
        self.map.user_data = new_user_data;
        self
    }

    /// Specify the padding in the `atlas_texture`.
    /// `inner`: Padding between the tiles,
    /// `topleft`: Padding to top and left of the tile atlas,
    /// `bottomright`: Padding to bottom and right of the atlas.
    ///
    /// Note that it is crucial that these values are precisely correct,
    /// we use them internally to determine how many tiles there are in the atlas in each
    /// direction, if that does not produce a number close to an integer,
    /// you will get a `panic` when the tile atlas is loaded.
    pub fn with_padding(mut self, inner: Vec2, topleft: Vec2, bottomright: Vec2) -> Self {
        self.map.map_uniform.inner_padding = inner;
        self.map.map_uniform.outer_padding_topleft = topleft;
        self.map.map_uniform.outer_padding_bottomright = bottomright;
        self
    }

    /// Build the map component.
    pub fn build(self) -> Map<UserData> {
        self.build_and_initialize(|_| {})
    }

    /// Build the map component and immediately initialize the map
    /// data with the given initializer callback.
    /// The callback will receive a mutable reference to a `MapIndexer`.
    pub fn build_and_initialize<F>(mut self, initializer: F) -> Map<UserData>
    where
        F: FnOnce(&mut MapIndexer<UserData>),
    {
        self.map.map_texture.resize(
            (self.map.map_size().x * self.map.map_size().y) as usize,
            0u32,
        );

        initializer(&mut MapIndexer::<UserData> { map: &mut self.map });

        self.map.map_uniform.update_world_size();

        self.map
    } // fn build_and_initialize

    /// Build the map component and immediately initialize the map
    /// data with the given initializer callback.
    /// The callback will receive a `UVec2` and return a `u32`.
    pub fn build_and_set<F>(self, mut initializer: F) -> Map<UserData>
    where
        F: FnMut(UVec2) -> u32,
    {
        let sx = self.map.map_size().x;
        let sy = self.map.map_size().y;

        self.build_and_initialize(|m: &mut MapIndexer<UserData>| {
            for y in 0..sy {
                for x in 0..sx {
                    m.set(x, y, initializer(uvec2(x, y)));
                }
            }
        })
    } // build_and_set()
}
