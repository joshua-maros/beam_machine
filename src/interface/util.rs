use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_mod_raycast::Intersection;

use crate::{block::BlockRaycastSet, world::Position};

fn world_to_block_pos(world: Vec3) -> Position {
    (
        world.x.round() as i32,
        world.y.round() as i32,
        world.z.round() as i32,
    )
}

pub(super) fn get_mouse_position_in_world(
    block_raycast_intersection: &Query<(&Intersection<BlockRaycastSet>,)>,
) -> Option<(Position, Position)> {
    if let Ok((intersection,)) = block_raycast_intersection.get_single() {
        if let (Some(&pos), Some(norm)) = (intersection.position(), intersection.normal()) {
            let above_cursor = world_to_block_pos(pos + 0.5 * norm);
            let below_cursor = world_to_block_pos(pos - 0.5 * norm);
            Some((above_cursor, below_cursor))
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn directional_key_index(event: &KeyboardInput) -> Option<usize> {
    let directional_key = match event.key_code {
        Some(KeyCode::W) => Some(0),
        Some(KeyCode::A) => Some(1),
        Some(KeyCode::S) => Some(2),
        Some(KeyCode::D) => Some(3),
        _ => None,
    };
    directional_key
}
