use bevy::prelude::*;

use crate::block::{BlockFacing, BlockKind};

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
