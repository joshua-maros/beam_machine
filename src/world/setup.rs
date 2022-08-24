use bevy::prelude::*;

use super::{base::World, WorldSnapshot};
use crate::{
    block::{Block, BlockFacing, BlockKind},
    interface::{import_level, EDITING},
    structure::Structure,
};

pub fn setup_world(commands: &mut Commands, assets: &AssetServer) -> usize {
    let mut world = World::new();
    import_level(&mut world, commands, assets);

    let first_user_part = world.parts().len();

    let blank_structure = Structure { blocks: Vec::new() };
    if !EDITING {
        world.add_part(blank_structure, commands, assets);
    }

    commands.insert_resource(WorldSnapshot(world.clone()));
    commands.insert_resource(world);

    first_user_part
}
