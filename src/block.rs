use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_mod_raycast::{RayCastMethod, RayCastSource};

use crate::world::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    DecoStructure,
    Structure,
    Activator,
    TractorBeamSource,
    LaserSource,
    LaserSensor,
}

impl BlockKind {
    pub fn asset_name(&self) -> &'static str {
        match self {
            Self::DecoStructure => "blocks/deco_structure.glb#Scene0",
            Self::Structure => "blocks/structure.glb#Scene0",
            Self::Activator => "blocks/activator.glb#Scene0",
            Self::TractorBeamSource => "blocks/tractor_beam_source.glb#Scene0",
            Self::LaserSource => "blocks/laser_source.glb#Scene0",
            Self::LaserSensor => "blocks/laser_sensor.glb#Scene0",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockFacing {
    Px,
    Py,
    Nx,
    Ny,
    Pz,
    Nz,
}

impl BlockFacing {
    pub fn all() -> [Self; 6] {
        [Self::Nz, Self::Pz, Self::Px, Self::Py, Self::Nx, Self::Ny]
    }

    pub fn offset(&self) -> Position {
        match self {
            Self::Px => (1, 0, 0),
            Self::Py => (0, 1, 0),
            Self::Nx => (-1, 0, 0),
            Self::Ny => (0, -1, 0),
            Self::Pz => (0, 0, 1),
            Self::Nz => (1, 0, -1),
        }
    }

    pub fn reverse(&self) -> Self {
        match self {
            Self::Px => Self::Nx,
            Self::Nx => Self::Px,
            Self::Py => Self::Ny,
            Self::Ny => Self::Py,
            Self::Pz => Self::Nz,
            Self::Nz => Self::Pz,
        }
    }

    pub fn rotation(&self) -> Quat {
        let t4 = TAU / 4.0;
        match self {
            Self::Px => Quat::IDENTITY,
            Self::Py => Quat::from_axis_angle(Vec3::Z, t4),
            Self::Nx => Quat::from_axis_angle(Vec3::Z, 2.0 * t4),
            Self::Ny => Quat::from_axis_angle(Vec3::Z, 3.0 * t4),
            Self::Pz => Quat::from_axis_angle(Vec3::Y, -t4),
            Self::Nz => Quat::from_axis_angle(Vec3::Y, t4),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub kind: BlockKind,
    pub facing: BlockFacing,
    pub position: Position,
}

#[derive(Component)]
pub struct BlockRaycastSet;

pub fn update_raycast_position_from_cursor(
    mut events: EventReader<CursorMoved>,
    mut source: Query<&mut RayCastSource<BlockRaycastSet>>,
) {
    if let Some(event) = events.iter().last() {
        if let Ok(mut source) = source.get_single_mut() {
            source.cast_method = RayCastMethod::Screenspace(event.position);
        }
    }
}
