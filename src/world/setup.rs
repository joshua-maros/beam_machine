use bevy::prelude::*;

use super::{base::World, WorldSnapshot};
use crate::{
    block::{Block, BlockFacing, BlockKind},
    simulation::make_spawner,
    structure::Structure,
};

pub fn setup_world(commands: &mut Commands, assets: &AssetServer) -> usize {
    let factory_floor = create_factory_floor();
    let mut world = World::new(factory_floor, commands, assets);

    let spawns = Structure {
        blocks: vec![Block {
            kind: BlockKind::DecoStructure2,
            facing: BlockFacing::Pz,
            position: (0, 0, 0),
        }],
    };
    make_spawner(spawns, &mut world, commands, assets);

    let first_user_part = world.parts().len();

    let blank_structure = Structure { blocks: Vec::new() };
    world.add_part(blank_structure, commands, assets);

    commands.insert_resource(WorldSnapshot(world.clone()));
    commands.insert_resource(world);

    first_user_part
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
