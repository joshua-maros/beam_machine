use bevy::{prelude::*, utils::HashSet};

use crate::{
    block::{Block, BlockFacing, BlockKind},
    structure::{spawn_structure, Structure},
};

pub type Position = (i32, i32, i32);

pub struct World {
    factory_floor: (Structure, Entity),
    machine_parts: Vec<(Structure, Entity)>,
    products: Vec<(Structure, Entity)>,
}

impl World {
    fn debug_assert_invariants(&self) {
        let mut positions = HashSet::new();
        self.factory_floor.0.debug_assert_invariants();
        for block in &self.factory_floor.0.blocks {
            debug_assert!(
                !positions.contains(&block.position),
                "we just checked for this earlier ._."
            );
            positions.insert(block.position);
        }
        for (index, part) in self.machine_parts.iter().enumerate() {
            part.0.debug_assert_invariants();
            for block in &part.0.blocks {
                debug_assert!(
                    !positions.contains(&block.position),
                    "Part {} overlaps with a previous part or the factory floor!",
                    index
                );
                positions.insert(block.position);
            }
        }
        for (index, product) in self.products.iter().enumerate() {
            product.0.debug_assert_invariants();
            for block in &product.0.blocks {
                debug_assert!(
                    !positions.contains(&block.position),
                    "Product {} overlaps with a previous product, part, or the factory floor!",
                    index
                );
                positions.insert(block.position);
            }
        }
    }

    fn new(factory_floor: Structure, commands: &mut Commands, assets: &AssetServer) -> Self {
        let factory_floor_ent = spawn_structure(&factory_floor, commands, assets);
        Self {
            factory_floor: (factory_floor, factory_floor_ent),
            machine_parts: Vec::new(),
            products: Vec::new(),
        }
    }

    fn update_part(
        root: Entity,
        new_structure: &Structure,
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let structure = spawn_structure(new_structure, commands, assets);
        let mut commands = commands.entity(root);
        commands.despawn_descendants();
        commands.add_child(structure);
    }

    pub fn add_machine_part(
        &mut self,
        part: Structure,
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let ent = commands
            .spawn()
            .insert_bundle(SpatialBundle::default())
            .id();
        Self::update_part(ent, &part, commands, assets);
        self.machine_parts.push((part, ent));
        self.debug_assert_invariants();
    }

    pub fn modify_machine_part(
        &mut self,
        index: usize,
        modifier: impl FnOnce(&mut Structure),
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let part = &mut self.machine_parts[index];
        modifier(&mut part.0);
        Self::update_part(part.1, &part.0, commands, assets);
        self.debug_assert_invariants();
    }
}

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
