use bevy::prelude::*;

use crate::{
    block::{Block, BlockFacing, BlockKind},
    world::{Part, Position, World, WorldSnapshot},
};

pub struct SimulationState {
    started: bool,
    running: bool,
    tick_timer: f32,
}

impl SimulationState {
    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn resume(&mut self) {
        self.running = true;
        self.tick_timer = 0.0;
    }
}

pub fn begin_simulation(
    world: &mut World,
    snapshot: &mut WorldSnapshot,
    simulation_state: &mut SimulationState,
) {
    simulation_state.resume();
    if simulation_state.started {
        return;
    }
    snapshot.0 = world.clone();
    simulation_state.started = true;
}

pub fn end_simulation(
    world: &mut World,
    snapshot: &WorldSnapshot,
    simulation_state: &mut SimulationState,
    commands: &mut Commands,
    assets: &AssetServer,
) {
    simulation_state.pause();
    if !simulation_state.started {
        return;
    }
    *world = snapshot.0.clone();
    world.refresh_all_parts(commands, assets);
    simulation_state.started = false;
}

fn any_other_part_contains_block_at(parts: &[Part], exclude: usize, at_position: Position) -> bool {
    for part in parts[..exclude].iter().chain(parts[exclude + 1..].iter()) {
        if part
            .0
            .blocks
            .iter()
            .any(|block| block.position == at_position)
        {
            return true;
        }
    }
    false
}

fn part_is_supported(parts: &[Part], part: usize, in_direction: BlockFacing) -> bool {
    for block in &parts[part].0.blocks {
        let p = block.position;
        let o = in_direction.offset();
        let below = (p.0 + o.0, p.1 + o.1, p.2 + o.2);
        if any_other_part_contains_block_at(parts, part, below) {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy)]
struct PhysicsState {
    can_move: [bool; 6],
    farthest_tractor_beam: [i32; 6],
}

fn run_simulation(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut state: ResMut<SimulationState>,
    time: Res<Time>,
    assets: Res<AssetServer>,
) {
    if !state.running {
        return;
    }
    state.tick_timer = state.tick_timer + 2.0 * time.delta_seconds();
    if state.tick_timer >= 1.0 {
        // Skip excess ticks if the number is far greater than one.
        state.tick_timer = state.tick_timer % 1.0;
    } else {
        return;
    }
    let parts = world.parts();
    let mut states = vec![
        PhysicsState {
            can_move: [false; 6],
            farthest_tractor_beam: [0; 6]
        };
        parts.len()
    ];
    let directions = BlockFacing::all();
    for block in all_blocks(parts).filter(|x| x.kind == BlockKind::TractorBeamSource) {
        let pull_direction = block.facing.reverse();
        let pull_direction_index = directions
            .iter()
            .position(|x| *x == pull_direction)
            .unwrap();
        let bp = block.position;
        let o = block.facing.offset();
        for distance in 1..100 {
            let position = (
                bp.0 + distance * o.0,
                bp.1 + distance * o.1,
                bp.2 + distance * o.2,
            );
            if let Some(part_index) = find_part_containing_block_at(parts, position) {
                let ftb = &mut states[part_index].farthest_tractor_beam[pull_direction_index];
                *ftb = (*ftb).max(distance);
            }
        }
    }
    for part_index in 1..parts.len() {
        // Gravity.
        states[part_index].farthest_tractor_beam[0] = i32::MAX;
        for (direction_index, &direction) in directions.iter().enumerate() {
            let parts = world.parts();
            let can_move = !part_is_supported(parts, part_index, direction);
            if can_move && states[part_index].farthest_tractor_beam[direction_index] > 1 {
                world.modify_part(
                    part_index,
                    |part| part.translate(direction.offset()),
                    &mut commands,
                    &*assets,
                );
                break;
            }
        }
    }
}

fn find_part_containing_block_at(parts: &[Part], position: Position) -> Option<usize> {
    parts
        .iter()
        .position(|part| part.0.blocks.iter().any(|block| block.position == position))
}

fn all_blocks(parts: &[Part]) -> impl Iterator<Item = &Block> {
    parts.iter().flat_map(|part| part.0.blocks.iter())
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationState {
            started: false,
            running: false,
            tick_timer: 0.0,
        });
        app.add_system(run_simulation);
    }
}
