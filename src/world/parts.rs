use bevy::prelude::*;

use super::{Part, World};
use crate::structure::{spawn_structure, Structure};

impl World {
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

    pub fn add_part(&mut self, part: Structure, commands: &mut Commands, assets: &AssetServer) {
        let ent = commands
            .spawn()
            .insert_bundle(SpatialBundle::default())
            .id();
        Self::update_part(ent, &part, commands, assets);
        self.parts.push((part, ent));
        self.debug_assert_invariants();
    }

    pub fn modify_part(
        &mut self,
        index: usize,
        modifier: impl FnOnce(&mut Structure),
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let part = &mut self.parts[index];
        modifier(&mut part.0);
        Self::update_part(part.1, &part.0, commands, assets);
        self.debug_assert_invariants();
    }

    pub fn remove_part(&mut self, index: usize, commands: &mut Commands) {
        commands.entity(self.parts[index].1).despawn_recursive();
        self.parts.remove(index);
    }

    pub fn refresh_all_parts(&self, commands: &mut Commands, assets: &AssetServer) {
        for part in &self.parts {
            Self::update_part(part.1, &part.0, commands, assets);
        }
    }

    pub fn parts(&self) -> &[Part] {
        &self.parts[..]
    }
}
