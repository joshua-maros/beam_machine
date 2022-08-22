use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_mod_raycast::Intersection;

use super::{util::get_mouse_position_in_world, Cursor, InterfaceState};
use crate::{
    block::{Block, BlockFacing, BlockKind, BlockRaycastSet},
    world::{Position, World},
};

pub(super) fn handle_mouse(
    cursor: &mut Query<(&mut Transform, &mut Visibility), (With<Cursor>, Without<Camera3d>)>,
    state: &mut InterfaceState,
    block_raycast_intersection: Query<(&Intersection<BlockRaycastSet>,)>,
    commands: &mut Commands,
    mouse_button_events: &mut EventReader<MouseButtonInput>,
    world: &mut World,
    assets: &AssetServer,
) {
    let [(_, mut place_cursor_visibility), (_, mut remove_cursor_visibility)] = cursor
        .get_many_mut([state.place_cursor, state.remove_cursor])
        .unwrap();
    place_cursor_visibility.is_visible = state.block_to_place.is_some();
    remove_cursor_visibility.is_visible = !state.block_to_place.is_some();
    let mouse_position = get_mouse_position_in_world(&block_raycast_intersection);
    if let Some((above_cursor, below_cursor)) = mouse_position {
        handle_mouse_events(
            commands,
            mouse_button_events,
            above_cursor,
            below_cursor,
            world,
            state,
            assets,
        );

        for (mut cursor_transform, _) in cursor.iter_mut() {
            let pos = if state.block_to_place.is_some() {
                above_cursor
            } else {
                below_cursor
            };
            cursor_transform.translation = Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
            cursor_transform.rotation = state.facing.rotation();
        }
    } else {
        place_cursor_visibility.is_visible = false;
        remove_cursor_visibility.is_visible = false;
    }
}

fn handle_mouse_events(
    commands: &mut Commands,
    mouse_button_events: &mut EventReader<MouseButtonInput>,
    above_cursor: Position,
    below_cursor: Position,
    world: &mut World,
    state: &mut InterfaceState,
    assets: &AssetServer,
) {
    for event in mouse_button_events.iter() {
        if event.button == MouseButton::Left && event.state == ButtonState::Released {
            if let Some(block_to_place) = state.block_to_place {
                place_block(
                    block_to_place,
                    state.facing,
                    world,
                    above_cursor,
                    commands,
                    assets,
                );
                if !state.holding_shift {
                    state.block_to_place = None;
                }
            } else {
                remove_block(world, below_cursor, commands, assets);
            }
        }
    }
}

fn place_block(
    kind: BlockKind,
    facing: BlockFacing,
    world: &mut World,
    above_cursor: (i32, i32, i32),
    commands: &mut Commands,
    assets: &AssetServer,
) {
    world.modify_part(
        1,
        |part| {
            part.blocks.push(Block {
                facing,
                kind,
                position: above_cursor,
            })
        },
        commands,
        assets,
    );
}

fn remove_block(
    world: &mut World,
    below_cursor: Position,
    commands: &mut Commands,
    assets: &AssetServer,
) {
    for part in 1..world.parts().len() {
        world.modify_part(
            part,
            |part| part.remove_blocks_at(below_cursor),
            commands,
            assets,
        );
    }
}
