use bevy::{prelude::*, utils::HashSet};
pub type Position = (i32, i32, i32);

use crate::structure::{spawn_structure, Structure};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Part {
    pub structure: Structure,
    pub physical_instance: Entity,
    pub is_hologram: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct World {
    pub(super) parts: Vec<Part>,
}

pub struct WorldSnapshot(pub World);

impl World {
    pub(super) fn debug_assert_invariants(&self) {
        let mut positions = HashSet::new();
        for (index, part) in self.parts.iter().enumerate() {
            if part.is_hologram {
                continue;
            }
            part.structure.debug_assert_invariants();
            for block in &part.structure.blocks {
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
        let factory_floor_ent = spawn_structure(&factory_floor, commands, assets, false);
        Self {
            parts: vec![Part {
                structure: factory_floor,
                physical_instance: factory_floor_ent,
                is_hologram: false,
            }],
        }
    }

    pub fn set(&mut self, to: Self, commands: &mut Commands, assets: &AssetServer) {
        for part in &self.parts {
            commands.entity(part.physical_instance).despawn_recursive();
        }
        self.parts.clear();
        for part in to.parts {
            if part.is_hologram {
                self.add_hologram(part.structure, commands, assets);
            } else {
                self.add_part(part.structure, commands, assets);
            }
        }
    }
}
