use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

use super::{util::directional_key_index, InterfaceState, EDITING};
use crate::{
    block::{BlockFacing, BlockKind},
    simulation::SimulationState,
};

pub(super) fn update_directional_key(
    event: &KeyboardInput,
    state: &mut InterfaceState,
    simulation_state: &SimulationState,
) {
    if state.block_to_place.is_some() && !simulation_state.is_started() {
        state.movement_keys.fill(false);
        return;
    }
    let directional_key = directional_key_index(event);
    if let Some(key) = directional_key {
        state.movement_keys[key] = event.state == ButtonState::Pressed;
    }
}

pub(super) fn update_block_keys(
    event: &KeyboardInput,
    state: &mut InterfaceState,
    simulation_state: &SimulationState,
) {
    if event.state != ButtonState::Pressed || simulation_state.is_started() {
        return;
    }
    if EDITING {
        match event.key_code {
            Some(KeyCode::Escape) => state.block_to_place = None,
            Some(KeyCode::Key1) => state.block_to_place = Some(BlockKind::DecoStructure),
            Some(KeyCode::Key2) => state.block_to_place = Some(BlockKind::DecoStructure2),
            Some(KeyCode::Key3) => state.block_to_place = Some(BlockKind::DecoStructureInput),
            Some(KeyCode::Key4) => state.block_to_place = Some(BlockKind::DecoStructureOutput),
            _ => (),
        }
    } else {
        match event.key_code {
            Some(KeyCode::Escape) => state.block_to_place = None,
            Some(KeyCode::Key1) => state.block_to_place = Some(BlockKind::Structure),
            Some(KeyCode::Key2) => state.block_to_place = Some(BlockKind::TractorBeamSource),
            _ => (),
        }
    }
    if state.block_to_place.is_some() {
        match event.key_code {
            Some(KeyCode::Q) => state.facing = BlockFacing::Ny,
            Some(KeyCode::W) => state.facing = BlockFacing::Pz,
            Some(KeyCode::E) => state.facing = BlockFacing::Nx,
            Some(KeyCode::A) => state.facing = BlockFacing::Px,
            Some(KeyCode::S) => state.facing = BlockFacing::Nz,
            Some(KeyCode::D) => state.facing = BlockFacing::Py,
            _ => (),
        }
    }
}

pub(super) fn move_cameras<'a>(
    cameras: impl Iterator<Item = Mut<'a, Transform>>,
    movement_keys: [bool; 4],
    time: &Time,
) {
    let mut total_offset = Vec2::ZERO;
    let offsets = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    for (&active, key_offset) in movement_keys.iter().zip(offsets.into_iter()) {
        if active {
            total_offset.x += key_offset.0;
            total_offset.y += key_offset.1;
        }
    }
    let speed = 10.0;
    for mut transform in cameras {
        transform.translation.x += total_offset.x * speed * time.delta_seconds();
        transform.translation.y += total_offset.y * speed * time.delta_seconds();
    }
}
