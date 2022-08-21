use bevy::{prelude::*, utils::HashSet};
pub type Position = (i32, i32, i32);

use crate::structure::{spawn_structure, Structure};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct World {
    pub(super) factory_floor: (Structure, Entity),
    pub(super) machine_parts: Vec<(Structure, Entity)>,
    pub(super) products: Vec<(Structure, Entity)>,
}

pub struct WorldSnapshot(pub World);

impl World {
    pub(super) fn debug_assert_invariants(&self) {
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

    pub(super) fn new(
        factory_floor: Structure,
        commands: &mut Commands,
        assets: &AssetServer,
    ) -> Self {
        let factory_floor_ent = spawn_structure(&factory_floor, commands, assets);
        Self {
            factory_floor: (factory_floor, factory_floor_ent),
            machine_parts: Vec::new(),
            products: Vec::new(),
        }
    }
}
