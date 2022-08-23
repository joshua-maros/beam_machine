use bevy::prelude::*;

use super::{Cursor, InterfaceMode, InterfaceState};
use crate::block::BlockFacing;

pub fn setup_interface_state(commands: &mut Commands, assets: &AssetServer, first_user_part: usize) {
    let scene = assets.load("blocks/cursor.glb#Scene0");
    let place_cursor = commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            ..Default::default()
        })
        .insert(Cursor)
        .id();
    let scene = assets.load("blocks/remove_cursor.glb#Scene0");
    let remove_cursor = commands
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
        first_user_part,
        currently_editing_part: first_user_part,
        block_to_place: None,
        facing: BlockFacing::Px,
        holding_shift: false,
        place_cursor,
        remove_cursor,
    });
}
