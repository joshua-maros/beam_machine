use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_mod_raycast::Intersection;

use crate::{
    block::{Block, BlockFacing, BlockKind, BlockRaycastSet},
    world::{Position, World},
};

#[derive(Component)]
pub struct Cursor;

pub enum InterfaceMode {
    Default,
}

pub struct InterfaceState {
    pub mode: InterfaceMode,
    pub movement_keys: [bool; 4],
    pub block_to_place: Option<BlockKind>,
    pub facing: BlockFacing,
    pub holding_shift: bool,
    pub cursor: Entity,
}
