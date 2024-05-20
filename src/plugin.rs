use crate::map::{log_map_events, update_loading_maps, update_map_vertex_attributes};
use bevy::{
    prelude::*,
    render::render_resource::{encase::internal::WriteInto, AsBindGroup, ShaderSize, ShaderType},
    sprite::Material2dPlugin,
};

use crate::{
    map::{DefaultUserData, Map},
    SHADER_CODE, SHADER_HANDLE,
};

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use [`crate::map_builder::MapBuilder`].
pub type FastTileMapPlugin = CustomFastTileMapPlugin<DefaultUserData>;

/// Plugin for fast tilemap.
/// Add this to you app and then spawn one or multiple maps use [`crate::map_builder::MapBuilder`].
#[derive(Default)]
pub struct CustomFastTileMapPlugin<UserData = DefaultUserData> {
    pub user_code: Option<String>,
    pub _user_data: std::marker::PhantomData<UserData>,
}

impl<UserData> Plugin for CustomFastTileMapPlugin<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<Map<UserData>>::default());
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.insert(SHADER_HANDLE, Shader::from_wgsl(SHADER_CODE, file!()));
        app.add_systems(
            Update,
            (
                (update_loading_maps::<UserData>, log_map_events::<UserData>).chain(),
                update_map_vertex_attributes::<UserData>,
            ),
        );
    }
}
