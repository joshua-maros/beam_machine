pub mod animations;
mod block;
mod interface;
mod setup;
mod simulation;
mod structure;
mod world;

use animations::AnimationPlugin;
use bevy::prelude::*;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastSystem};
use bevy_obj::ObjPlugin;
use block::{update_raycast_position_from_cursor, BlockRaycastSet};
use interface::InterfacePlugin;
use setup::SetupPlugin;
use simulation::SimulationPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(DefaultRaycastingPlugin::<BlockRaycastSet>::default())
        .add_plugin(AnimationPlugin)
        .add_plugin(InterfacePlugin)
        .add_plugin(SimulationPlugin)
        .add_plugin(SetupPlugin)
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_position_from_cursor.before(RaycastSystem::BuildRays::<BlockRaycastSet>),
        )
        .run();
}
