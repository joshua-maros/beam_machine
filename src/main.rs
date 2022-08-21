mod block;
mod interface;
mod startup;
mod structure;
mod world;

use bevy::prelude::*;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastSystem};
use bevy_obj::ObjPlugin;
use block::{update_raycast_position_from_cursor, BlockRaycastSet};
use interface::interface_system;
use startup::startup_system;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(DefaultRaycastingPlugin::<BlockRaycastSet>::default())
        .add_startup_system(startup_system)
        .add_system(interface_system)
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_position_from_cursor.before(RaycastSystem::BuildRays::<BlockRaycastSet>),
        )
        .run();
}
