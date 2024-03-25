#![allow(clippy::all, unused_imports, deprecated, dead_code)]
// @generated by cargo bevy-api-gen generate, modify the templates not this file

use super::bevy_ecs::*;

use super::bevy_reflect::*;

use super::bevy_asset::*;

use super::bevy_core::*;

use super::bevy_hierarchy::*;

use super::bevy_input::*;

use super::bevy_window::*;

use super::bevy_render::*;

use super::bevy_time::*;

use super::bevy_transform::*;

use super::bevy_core_pipeline::*;

use super::bevy_pbr::*;

use super::bevy_sprite::*;

extern crate self as bevy_script_api;

/// The [`GizmoConfigGroup`] used for debug visualizations of [`Aabb`] components on entities

#[derive(bevy_mod_scripting_lua_derive::LuaProxy)]
#[proxy(
derive(clone,debug),
remote="bevy_gizmos::aabb::AabbGizmoConfigGroup",
functions[r#"

    #[lua(as_trait = "std::clone::Clone", kind = "Method", output(proxy))]
    fn clone(&self) -> bevy_gizmos::aabb::AabbGizmoConfigGroup;

"#]
)]

pub struct LuaAabbGizmoConfigGroup {}

/// Add this [`Component`] to an entity to draw its [`Aabb`] component.

#[derive(bevy_mod_scripting_lua_derive::LuaProxy)]
#[proxy(
derive(clone,debug),
remote="bevy_gizmos::aabb::ShowAabbGizmo",
functions[]
)]

pub struct LuaShowAabbGizmo {}

/// The default gizmo config group.

#[derive(bevy_mod_scripting_lua_derive::LuaProxy)]
#[proxy(
derive(clone,debug),
remote="bevy_gizmos::config::DefaultGizmoConfigGroup",
functions[]
)]

pub struct LuaDefaultGizmoConfigGroup {}

/// A struct that stores configuration for gizmos.

#[derive(bevy_mod_scripting_lua_derive::LuaProxy)]
#[proxy(
derive(clone,debug),
remote="bevy_gizmos::config::GizmoConfig",
functions[r#"

    #[lua(as_trait = "std::clone::Clone", kind = "Method", output(proxy))]
    fn clone(&self) -> bevy_gizmos::config::GizmoConfig;

"#]
)]

pub struct LuaGizmoConfig {}

bevy_script_api::util::impl_tealr_generic!(pub(crate) struct T);

#[derive(Default)]
pub(crate) struct Globals;

impl bevy_mod_scripting_lua::tealr::mlu::ExportInstances for Globals {
    fn add_instances<'lua, T: bevy_mod_scripting_lua::tealr::mlu::InstanceCollector<'lua>>(
        self,
        instances: &mut T,
    ) -> bevy_mod_scripting_lua::tealr::mlu::mlua::Result<()> {
        Ok(())
    }
}

pub struct BevyGizmosAPIProvider;

impl bevy_mod_scripting_core::hosts::APIProvider for BevyGizmosAPIProvider {
    type APITarget = std::sync::Mutex<bevy_mod_scripting_lua::tealr::mlu::mlua::Lua>;
    type ScriptContext = std::sync::Mutex<bevy_mod_scripting_lua::tealr::mlu::mlua::Lua>;
    type DocTarget = bevy_mod_scripting_lua::docs::LuaDocFragment;

    fn attach_api(
        &mut self,
        ctx: &mut Self::APITarget,
    ) -> Result<(), bevy_mod_scripting_core::error::ScriptError> {
        let ctx = ctx
            .get_mut()
            .expect("Unable to acquire lock on Lua context");
        bevy_mod_scripting_lua::tealr::mlu::set_global_env(Globals, ctx)
            .map_err(|e| bevy_mod_scripting_core::error::ScriptError::Other(e.to_string()))
    }

    fn get_doc_fragment(&self) -> Option<Self::DocTarget> {
        Some(bevy_mod_scripting_lua::docs::LuaDocFragment::new(
            "BevyGizmosAPI",
            |tw| {
                tw.document_global_instance::<Globals>()
                    .expect("Something went wrong documenting globals")
                    .process_type::<LuaAabbGizmoConfigGroup>()
                    .process_type::<LuaShowAabbGizmo>()
                    .process_type::<LuaDefaultGizmoConfigGroup>()
                    .process_type::<LuaGizmoConfig>()
            },
        ))
    }

    fn setup_script(
        &mut self,
        script_data: &bevy_mod_scripting_core::hosts::ScriptData,
        ctx: &mut Self::ScriptContext,
    ) -> Result<(), bevy_mod_scripting_core::error::ScriptError> {
        Ok(())
    }

    fn setup_script_runtime(
        &mut self,
        world_ptr: bevy_mod_scripting_core::world::WorldPointer,
        _script_data: &bevy_mod_scripting_core::hosts::ScriptData,
        ctx: &mut Self::ScriptContext,
    ) -> Result<(), bevy_mod_scripting_core::error::ScriptError> {
        Ok(())
    }

    fn register_with_app(&self, app: &mut bevy::app::App) {
        app.register_foreign_lua_type::<bevy_gizmos::aabb::AabbGizmoConfigGroup>();

        app.register_foreign_lua_type::<bevy_gizmos::aabb::ShowAabbGizmo>();

        app.register_foreign_lua_type::<bevy_gizmos::config::DefaultGizmoConfigGroup>();

        app.register_foreign_lua_type::<bevy_gizmos::config::GizmoConfig>();
    }
}
