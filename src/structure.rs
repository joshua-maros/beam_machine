use bevy::{prelude::*, utils::HashSet};
use bevy_mod_raycast::RayCastMesh;

use crate::block::{Block, BlockRaycastSet};

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
        .id()
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
        let block = spawn_block(commands, block, assets);
        commands.entity(root).add_child(block);
    }

    root
}
