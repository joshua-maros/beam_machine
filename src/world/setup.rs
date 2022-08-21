use bevy::prelude::*;

use super::base::World;
use crate::{
    block::{Block, BlockFacing, BlockKind},
    structure::Structure,
};

pub fn setup_world(commands: &mut Commands, assets: &AssetServer) {
    let factory_floor = create_factory_floor();
    let mut world = World::new(factory_floor, commands, assets);
    let blank_structure = Structure { blocks: Vec::new() };
    world.add_machine_part(blank_structure, commands, assets);
    commands.insert_resource(world);
}

fn create_factory_floor() -> Structure {
    let mut factory_floor = Structure { blocks: Vec::new() };
    let size = 20;
    for x in -size..=size {
        for y in -size..=size {
            factory_floor.blocks.push(Block {
                kind: BlockKind::DecoStructure,
                facing: BlockFacing::Pz,
                position: (x, y, -1),
            });
        }
    }
    factory_floor
}
