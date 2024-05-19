#import bevy_sprite::mesh2d_bindings::mesh
#import bevy_sprite::mesh2d_functions::{get_model_matrix, mesh2d_position_local_to_clip, mesh2d_position_local_to_world}

struct ExtractIn {
    tile_index: u32,
    tile_position: vec2<i32>,
    tile_offset: vec2<f32>,
    animation_state: f32,
};

#[user_code]

struct Map {
    /// Size of the map, in tiles.
    /// Will be derived from underlying map texture.
    map_size: vec2<u32>,

    /// Size of the tile atlas, in pixels.
    /// Will be derived from the tile atlas texture.
    atlas_size: vec2<f32>,

    /// Size of each tile, in pixels.
    tile_size: vec2<f32>,

    /// Padding between tiles in atlas.
    inner_padding: vec2<f32>,

    /// Padding at atlas top/left and bottom/right
    outer_padding_topleft: vec2<f32>,
    outer_padding_bottomright: vec2<f32>,

    /// Relative anchor point position in a tile (in [0..1]^2)
    tile_anchor_point: vec2<f32>,

    /// fractional 2d map index -> relative local world pos
    projection: mat3x3<f32>,

    /// Global transform of the entity holding the map as transformation matrix & offset.
    /// This is currently redundant with mesh.model,
    /// which should represent the same info as a 4x4 affine matrix, but we consider it a bit
    /// more consistent in conjunction with the inverse below. May be removed in the future.
    global_transform_matrix: mat3x3<f32>,
    global_transform_translation: vec3<f32>,

    // -----
    /// [derived] Size of the map in world units necessary to display
    /// all tiles according to projection.
    world_size: vec2<f32>,

    /// [derived] Offset of the projected map in world coordinates
    world_offset: vec2<f32>,

    /// [derived] Number of tiles in atlas
    n_tiles: vec2<u32>,

    /// [derived] Inverse of global transform of the entity holding the map as transformation matrix & offset.
    global_inverse_transform_matrix: mat3x3<f32>,
    global_inverse_transform_translation: vec3<f32>,
};

@group(2) @binding(0)
var<uniform> map: Map;

@group(2) @binding(1)
var<uniform> user_data: UserData;

@group(2) @binding(100)
var<storage> map_texture: array<u32>;

@group(2) @binding(101)
var atlas_texture: texture_2d<f32>;

@group(2) @binding(102)
var atlas_sampler: sampler;


struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) map_position: vec2<f32>,
    @location(2) mix_color: vec4<f32>,
    @location(3) animation_state: f32,
};

struct VertexOutput {
    // this is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) map_position: vec2<f32>,
    @location(2) mix_color: vec4<f32>,
    @location(3) animation_state: f32,
}

/// Custom vertex shader for passing along the UV coordinate
@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var model: mat4x4<f32> = get_model_matrix(v.instance_index);

    out.position = mesh2d_position_local_to_clip(model, vec4<f32>(v.position, 1.0));
    out.world_position = mesh2d_position_local_to_world(model, vec4<f32>(v.position, 1.0));
    out.mix_color = v.mix_color;
    out.map_position = v.map_position;
    out.animation_state = v.animation_state;
    return out;
}

/// Position (world/pixel units) in tilemap atlas of the top left corner
/// of the tile with the given index
fn atlas_index_to_position(index: u32) -> vec2<f32> {
    var index_f = f32(index);
    var index_y = floor(index_f / f32(map.n_tiles.x));
    var index_x = index_f - index_y * f32(map.n_tiles.x);
    var index2d = vec2<f32>(index_x, index_y);

    var pos = index2d * (map.tile_size + map.inner_padding) + map.outer_padding_topleft;
    return pos;
}

/// Sample tile from the tile atlas
/// tile_index: Index of the tile in the atlas
/// tile_offset: Offset from tile anchor point in pixel/world coordinates
fn _sample_tile(
    tile_index: u32,
    pos: MapPosition,
    animation_state: f32,
) -> vec4<f32> {

    var e: ExtractIn;
    e.tile_index = tile_index;
    e.tile_position = pos.tile;
    e.tile_offset = pos.offset;
    e.animation_state = animation_state;

    return sample_tile(e);
}

fn sample_tile_at(
    tile_index: u32,
    tile_position: vec2<i32>,
    tile_offset: vec2<f32>,
) -> vec4<f32> {
    // Tile start position in the atlas
    var tile_start = atlas_index_to_position(tile_index);

    // Offset in pixels from tile_start to sample from
    var rect_offset = tile_offset + map.tile_anchor_point * map.tile_size;
    var total_offset = tile_start + rect_offset;

    return textureSample(
        atlas_texture, atlas_sampler, total_offset / map.atlas_size
    );
}

/// 2d map tile position and offset in map tile coordinates
struct MapPosition {
    /// The 2d tile position on the map
    tile: vec2<i32>,
    /// Offset in pixels/world coordinates from the reference position of that tile
    offset: vec2<f32>
};


///
fn get_tile_index(map_position: vec2<i32>) -> u32 {
    return map_texture[map_position.y * i32(map.map_size.x) + map_position.x];
}

fn get_tile_index_checked(map_position: vec2<i32>) -> u32 {
    if !is_valid_tile(map_position) {
        return 0u;
    }
    return get_tile_index(map_position);
}

fn blend(c0: vec4<f32>, c1: vec4<f32>) -> vec4<f32> {
    return mix(c0, c1, c1.a);
}

fn is_valid_tile(tile: vec2<i32>) -> bool {
    if tile.x < 0 || tile.y < 0 {
        return false;
    }
    let map_size = vec2<i32>(map.map_size);
    if tile.x >= map_size.x || tile.y >= map_size.y {
        return false;
    }
    return true;
}

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    var world_position = in.world_position.xy;
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    var tile = floor(in.map_position);
    var map_space_offset = in.map_position - tile;

    var world_space_offset = map.global_transform_matrix * (
        map.projection * vec3<f32>(map_space_offset, 0.0)
    ) * vec3<f32>(map.tile_size, 1.0);

    var pos: MapPosition;
    pos.tile = vec2<i32>(tile);
    pos.offset = vec2<f32>(1.0, -1.0) * world_space_offset.xy;

    var index = get_tile_index(pos.tile);
    var is_valid = is_valid_tile(pos.tile);
    var sample_color = color;

    if is_valid {
        sample_color = _sample_tile(index, pos, in.animation_state);
    }
    else {
        // for invalid tile, assume low index so (almost) everything overlaps in dominance rendering
        index = 0u;
    }

    if is_valid {
        color = blend(color, sample_color);
    }

    color = color * in.mix_color;

    return color;
}
