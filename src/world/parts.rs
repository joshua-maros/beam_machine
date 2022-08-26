use bevy::prelude::*;

use super::{Part, World};
use crate::{
    animations::Animation,
    structure::{spawn_structure, Structure}, setup::LevelEntity,
};

impl World {
    fn update_part(part: &Part, commands: &mut Commands, assets: &AssetServer) {
        let structure = spawn_structure(&part.structure, commands, assets, part.is_hologram);
        let mut commands = commands.entity(part.physical_instance);
        commands.despawn_descendants();
        commands.add_child(structure);
    }

    fn add_part_impl(
        &mut self,
        part: Structure,
        commands: &mut Commands,
        assets: &AssetServer,
        is_hologram: bool,
    ) {
        let ent = commands
            .spawn()
            .insert_bundle(SpatialBundle::default())
            .insert(LevelEntity)
            .id();
        let index = self.parts.len();
        self.parts.push(Part {
            structure: part,
            physical_instance: ent,
            is_hologram,
        });
        Self::update_part(&self.parts[index], commands, assets);
        self.debug_assert_invariants();
    }

    pub fn add_part(&mut self, part: Structure, commands: &mut Commands, assets: &AssetServer) {
        self.add_part_impl(part, commands, assets, false);
    }

    pub fn add_hologram(&mut self, part: Structure, commands: &mut Commands, assets: &AssetServer) {
        self.add_part_impl(part, commands, assets, true);
    }

    pub fn modify_part(
        &mut self,
        index: usize,
        modifier: impl FnOnce(&mut Structure),
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let part = &mut self.parts[index];
        modifier(&mut part.structure);
        Self::update_part(&*part, commands, assets);
        self.debug_assert_invariants();
    }

    pub fn merge_parts(
        &mut self,
        parts: impl Iterator<Item = usize>,
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let mut parts: Vec<_> = parts.into_iter().collect();
        parts.sort();
        parts.reverse();
        let parts: Vec<_> = parts
            .into_iter()
            .map(|index| self.remove_part(index, commands))
            .collect();
        let mut new_structure = Structure {
            blocks: parts
                .into_iter()
                .flat_map(|part| part.structure.blocks.into_iter())
                .collect(),
        };
        self.add_part(new_structure, commands, assets);
    }

    pub fn remove_part(&mut self, index: usize, commands: &mut Commands) -> Part {
        commands
            .entity(self.parts[index].physical_instance)
            .despawn_recursive();
        self.parts.remove(index)
    }

    pub fn animate_part(&mut self, index: usize, animation: Animation, commands: &mut Commands) {
        commands
            .entity(self.parts[index].physical_instance)
            .insert(animation);
    }

    pub fn refresh_all_parts(&self, commands: &mut Commands, assets: &AssetServer) {
        for part in &self.parts {
            Self::update_part(part, commands, assets);
        }
    }

    pub fn parts(&self) -> &[Part] {
        &self.parts[..]
    }
}
