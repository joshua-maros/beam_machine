use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_mod_raycast::Intersection;

use super::{util::get_mouse_position_in_world, Cursor, InterfaceState, EDITING};
use crate::{
    block::{Block, BlockFacing, BlockKind, BlockRaycastSet},
    simulation::SimulationState,
    structure::Structure,
    world::{Position, World},
    Sfx,
};

pub(super) fn handle_mouse(
    cursor: &mut Query<(&mut Transform, &mut Visibility), (With<Cursor>, Without<Camera3d>)>,
    state: &mut InterfaceState,
    simulation_state: &SimulationState,
    block_raycast_intersection: Query<(&Intersection<BlockRaycastSet>,)>,
    commands: &mut Commands,
    clicked: bool,
    world: &mut World,
    assets: &AssetServer,
    sfx: &Sfx,
    audio: &Audio,
) {
    let [(_, mut place_cursor_visibility), (_, mut remove_cursor_visibility)] = cursor
        .get_many_mut([state.place_cursor, state.remove_cursor])
        .unwrap();
    place_cursor_visibility.is_visible = state.block_to_place.is_some();
    remove_cursor_visibility.is_visible = !state.block_to_place.is_some();
    if simulation_state.is_started() {
        place_cursor_visibility.is_visible = false;
        remove_cursor_visibility.is_visible = false;
        return;
    }
    let mouse_position = get_mouse_position_in_world(&block_raycast_intersection);
    if let Some((above_cursor, below_cursor)) = mouse_position {
        handle_mouse_events(
            commands,
            clicked,
            above_cursor,
            below_cursor,
            world,
            state,
            assets,
            sfx,
            audio,
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
    clicked: bool,
    above_cursor: Position,
    below_cursor: Position,
    world: &mut World,
    state: &mut InterfaceState,
    assets: &AssetServer,
    sfx: &Sfx,
    audio: &Audio,
) {
    if clicked {
        if let Some(block_to_place) = state.block_to_place {
            place_block(
                block_to_place,
                state.facing,
                &mut state.currently_editing_part,
                world,
                above_cursor,
                commands,
                assets,
                sfx,
                audio,
            );
            if !state.holding_shift {
                state.block_to_place = None;
            }
        } else {
            remove_block(world, below_cursor, commands, assets, &*state, sfx, audio);
        }
    }
}

fn place_block(
    kind: BlockKind,
    facing: BlockFacing,
    part_index: &mut usize,
    world: &mut World,
    above_cursor: (i32, i32, i32),
    commands: &mut Commands,
    assets: &AssetServer,
    sfx: &Sfx,
    audio: &Audio,
) {
    let index = match kind {
        BlockKind::Structure => 0,
        BlockKind::TractorBeamSource => 1,
        BlockKind::WelderBeamSource => 2,
        _ => 0,
    };
    audio.play_with_settings(
        sfx.place[index].clone(),
        PlaybackSettings::ONCE.with_volume(0.3),
    );
    world.modify_part(
        *part_index,
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
    state: &InterfaceState,
    sfx: &Sfx,
    audio: &Audio,
) {
    let start = if EDITING { 0 } else { state.first_user_part };
    for part in start..world.parts().len() {
        world.modify_part(
            part,
            |part| part.remove_blocks_at(below_cursor),
            commands,
            assets,
        );
    }
    audio.play_with_settings(sfx.click.clone(), PlaybackSettings::ONCE.with_volume(0.3));
}
