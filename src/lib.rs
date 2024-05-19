//! GPU-accelerated tilemap functionality for bevy.
//! Aims at rendering tilemaps with lightning speed by using just a single quad per map (layer)
//! and offloading the actual rendering to GPU.
//! This should be faster than most other bevy tilemap implementations as of this writing.
//!
//! ## Features
//!
//! - Very high rendering performance (hundreds of fps, largely independent of map size)
//! - Tilemaps can be very large or have many "layers"
//! - Rectangular and isometric (axonometric) tile maps.
//! - Tile overlaps either by "dominance" rule or by perspective
//! - Optional custom mesh for which the map serves as a texture
//!
//! ## How it works
//!
//! The principle is probably not new but nonetheless quite helpful: The whole tilemap (-layer) is
//! rendered as a single quad and a shader cares for rendering the correct tiles at the correct
//! position.

pub mod bundle;
pub mod map;
pub mod map_builder;
pub mod map_uniform;
pub mod plugin;

pub use crate::{
    bundle::{MapBundleManaged, MapBundleUnmanaged},
    map::{Map, MapAttributes, MapIndexer, MeshManagedByMap},
    plugin::{CustomFastTileMapPlugin, FastTileMapPlugin},
};

use bevy::math::{mat2, mat3, vec2, vec3};
use bevy::prelude::*;

pub const SHADER_CODE: &str = include_str!("../assets/tilemap_shader.wgsl");
pub const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(15375856360518374895);

pub const PROJECTION: Mat3 = mat3(
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, 0.0, 1.0),
);
pub const INVERSE_PROJECTION: Mat2 = mat2(vec2(1.0, 0.0), vec2(0.0, -1.0));
