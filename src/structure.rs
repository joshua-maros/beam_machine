use bevy::{pbr::NotShadowCaster, prelude::*, utils::HashSet};
use bevy_mod_raycast::RayCastMesh;

use crate::{
    block::{Block, BlockFacing, BlockKind, BlockRaycastSet},
    world::Position,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Structure {
    pub blocks: Vec<Block>,
}

impl Structure {
    pub fn debug_assert_invariants(&self) {
        let mut positions = HashSet::new();
        for block in &self.blocks {
            debug_assert!(
                !positions.contains(&block.position),
                "Structure contains overlapping blocks!"
            );
            positions.insert(block.position);
        }
    }

    pub fn translate(&mut self, offset: Position) {
        for block in &mut self.blocks {
            block.position.0 += offset.0;
            block.position.1 += offset.1;
            block.position.2 += offset.2;
        }
    }

    pub fn remove_blocks_at(&mut self, position: Position) {
        self.debug_assert_invariants();
        for i in 0..self.blocks.len() {
            if self.blocks[i].position == position {
                self.blocks.remove(i);
                return;
            }
        }
    }

    pub fn set_block(&mut self, block: Block) {
        for existing_block in &mut self.blocks {
            if existing_block.position == block.position {
                *existing_block = block;
                return;
            }
        }
        self.blocks.push(block);
    }

    pub fn contains_block(&self, block: &Block) -> bool {
        self.blocks.contains(block)
    }

    pub fn matches(&self, other: &Structure) -> bool {
        if self.blocks.len() != other.blocks.len() {
            false
        } else {
            for block in &self.blocks {
                if !other.contains_block(block) {
                    return false;
                }
            }
            true
        }
    }
}

fn spawn_block(commands: &mut Commands, block: &Block, assets: &AssetServer) -> Entity {
    let bbox = assets.load::<Mesh, _>("blocks/bounding_box.obj");
    let scene = assets.load(block.kind.asset_name());
    commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            transform: Transform::from_translation(Vec3::new(
                block.position.0 as f32,
                block.position.1 as f32,
                block.position.2 as f32,
            ))
            .with_rotation(block.facing.rotation()),
            ..Default::default()
        })
        // This will not be rendered since there is no material attached.
        .insert(bbox)
        .insert(RayCastMesh::<BlockRaycastSet>::default())
        .insert(NotShadowCaster)
        .id()
}

#[derive(Component)]
pub struct Beam {
    pub for_block: Block,
}

pub fn spawn_structure(
    structure: &Structure,
    commands: &mut Commands,
    assets: &AssetServer,
) -> Entity {
    let root = commands
        .spawn()
        .insert_bundle(SpatialBundle::default())
        .id();

    for block in &structure.blocks {
        if block.kind == BlockKind::TractorBeamSource {
            let scene = assets.load("tractor_beam.glb#Scene0");
            let beam = commands
                .spawn()
                .insert(Beam { for_block: *block })
                .insert_bundle(SceneBundle {
                    scene,
                    transform: Transform::from_translation(Vec3::new(
                        block.position.0 as f32,
                        block.position.1 as f32,
                        block.position.2 as f32,
                    ))
                    .with_rotation(block.facing.rotation()),
                    ..Default::default()
                })
                .id();
            commands.entity(root).add_child(beam);
        }
        let block = spawn_block(commands, block, assets);
        commands.entity(root).add_child(block);
    }

    root
}
