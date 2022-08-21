use bevy::prelude::*;

use crate::world::{Part, Position, World, WorldSnapshot};

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

fn part_is_supported(parts: &[Part], part: usize) -> bool {
    for block in &parts[part].0.blocks {
        let p = block.position;
        let below = (p.0, p.1, p.2 - 1);
        if any_other_part_contains_block_at(parts, part, below) {
            return true;
        }
    }
    false
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
    let mut should_fall = vec![false; parts.len()];
    for part_index in 1..parts.len() {
        should_fall[part_index] = !part_is_supported(parts, part_index);
    }
    for part_index in 1..parts.len() {
        if should_fall[part_index] {
            world.modify_part(
                part_index,
                |part| part.translate((0, 0, -1)),
                &mut commands,
                &*assets,
            );
        }
    }
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
