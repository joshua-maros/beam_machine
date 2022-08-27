use bevy::prelude::*;

use super::{base::World, WorldSnapshot};
use crate::{
    interface::{import_level, EDITING},
    setup_menu::GlobalState,
    structure::Structure,
};

pub fn setup_world(
    commands: &mut Commands,
    assets: &AssetServer,
    global_state: &GlobalState,
) -> usize {
    let mut world = World::new();
    let first_user_part = import_level(
        &global_state.levels[global_state.current_level],
        &mut world,
        commands,
        assets,
    );

    let blank_structure = Structure { blocks: Vec::new() };
    if !EDITING {
        world.add_part(blank_structure, commands, assets);
    }

    commands.insert_resource(WorldSnapshot(world.clone()));
    commands.insert_resource(world);

    first_user_part
}
