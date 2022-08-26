use bevy::prelude::*;

use super::{Cursor, InterfaceMode, InterfaceState, EDITING};
use crate::{block::BlockFacing, setup::LevelEntity};

pub fn setup_interface_state(
    commands: &mut Commands,
    assets: &AssetServer,
    first_user_part: usize,
) {
    let scene = assets.load("blocks/cursor.glb#Scene0");
    let place_cursor = commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            ..Default::default()
        })
        .insert(Cursor)
        .insert(LevelEntity)
        .id();
    let scene = assets.load("blocks/remove_cursor.glb#Scene0");
    let remove_cursor = commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            ..Default::default()
        })
        .insert(Cursor)
        .insert(LevelEntity)
        .id();
    commands.insert_resource(InterfaceState {
        mode: InterfaceMode::Default,
        movement_keys: [false; 4],
        first_user_part,
        currently_editing_part: if EDITING { 0 } else { first_user_part },
        block_to_place: None,
        facing: BlockFacing::Px,
        holding_shift: false,
        place_cursor,
        remove_cursor,
    });
}
