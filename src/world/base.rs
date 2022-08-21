use bevy::{prelude::*, utils::HashSet};
pub type Position = (i32, i32, i32);

use crate::structure::{spawn_structure, Structure};

pub type Part = (Structure, Entity);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct World {
    pub(super) parts: Vec<Part>,
}

pub struct WorldSnapshot(pub World);

impl World {
    pub(super) fn debug_assert_invariants(&self) {
        let mut positions = HashSet::new();
        for (index, part) in self.parts.iter().enumerate() {
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
    }

    pub(super) fn new(
        factory_floor: Structure,
        commands: &mut Commands,
        assets: &AssetServer,
    ) -> Self {
        let factory_floor_ent = spawn_structure(&factory_floor, commands, assets);
        Self {
            parts: vec![(factory_floor, factory_floor_ent)],
        }
    }
}
