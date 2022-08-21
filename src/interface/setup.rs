use bevy::prelude::*;

use crate::block::BlockFacing;

use super::{Cursor, InterfaceMode, InterfaceState};

pub fn setup_interface_state(commands: &mut Commands, assets: &AssetServer) {
    let scene = assets.load("blocks/cursor.glb#Scene0");
    let cursor = commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            ..Default::default()
        })
        .insert(Cursor)
        .id();
    commands.insert_resource(InterfaceState {
        mode: InterfaceMode::Default,
        movement_keys: [false; 4],
        block_to_place: None,
        facing: BlockFacing::Px,
        holding_shift: false,
        cursor,
    });
}
