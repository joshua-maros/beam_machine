use std::collections::VecDeque;

use bevy::{ecs::schedule::ParallelExecutor, prelude::*, utils::HashSet};

use crate::{
    animations::Animation,
    block::{Block, BlockFacing, BlockKind},
    interface::{exit_level, ChangeToCompleteRequest, InterfaceState},
    setup::LevelEntity,
    setup_menu::GlobalState,
    structure::{Beam, Structure},
    world::{Part, Position, World, WorldSnapshot},
    GameState, Sfx,
};

pub struct SimulationState {
    pub started: bool,
    pub running: bool,
    pub tick_timer: f32,
    pub existing_parts: usize,
    pub cycles: usize,
    pub collected_outputs: usize,
    pub speed: f32,
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

    pub(crate) fn tick_progress(&self) -> f32 {
        self.tick_timer
    }
}

pub fn begin_simulation(
    world: &mut World,
    snapshot: &mut WorldSnapshot,
    simulation_state: &mut SimulationState,
    speed: f32,
    commands: &mut Commands,
    assets: &AssetServer,
) {
    simulation_state.resume();
    simulation_state.speed = speed;
    if simulation_state.started {
        return;
    }
    snapshot.0 = world.clone();
    world.split_loose_parts(commands, assets);
    simulation_state.started = true;
    simulation_state.existing_parts = world.parts().len();
    simulation_state.collected_outputs = 0;
    simulation_state.cycles = 0;
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
    world.set(snapshot.0.clone(), commands, assets);
    simulation_state.started = false;
}

fn any_other_part_contains_block_at(parts: &[Part], exclude: usize, at_position: Position) -> bool {
    let parts: Box<dyn Iterator<Item = &Part>> = if exclude < parts.len() {
        Box::new(parts[..exclude].iter().chain(parts[exclude + 1..].iter()))
    } else {
        Box::new(parts.iter())
    };
    for part in parts {
        if part.is_hologram {
            continue;
        }
        if part
            .structure
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
    for block in &parts[part].structure.blocks {
        let p = block.position;
        let o = in_direction.offset();
        let below = (p.0 + o.0, p.1 + o.1, p.2 + o.2);
        if any_other_part_contains_block_at(parts, part, below) {
            return true;
        }
    }
    false
}

fn part_touches(parts: &[Part], part: usize, in_direction: BlockFacing) -> HashSet<usize> {
    let mut included_parts = HashSet::from_iter(std::iter::once(part));
    let mut included_parts_queue = VecDeque::from_iter(std::iter::once(part));
    while let Some(part) = included_parts_queue.pop_front() {
        for block in &parts[part].structure.blocks {
            let p = block.position;
            let o = in_direction.offset();
            let touching = (p.0 + o.0, p.1 + o.1, p.2 + o.2);
            for part in 0..parts.len() {
                if included_parts.contains(&part) {
                    continue;
                }
                if !parts[part].is_hologram
                    && parts[part]
                        .structure
                        .blocks
                        .iter()
                        .any(|b| b.position == touching)
                {
                    included_parts.insert(part);
                    included_parts_queue.push_back(part);
                }
            }
        }
    }
    included_parts
}

#[derive(Clone, Copy, Debug)]
struct PhysicsState {
    can_move: [bool; 6],
    farthest_tractor_beam: [(i32, usize); 6],
}

#[derive(Clone, Debug, Component)]
pub struct Input {
    spawns: Structure,
}

pub fn make_input(
    spawns: Structure,
    world: &mut World,
    commands: &mut Commands,
    assets: &AssetServer,
) {
    world.add_hologram(spawns.clone(), commands, assets);
    commands
        .spawn()
        .insert(Input { spawns })
        .insert(LevelEntity);
}

#[derive(Clone, Debug, Component)]
pub struct Output {
    accepts: Structure,
}

pub fn make_output(
    accepts: Structure,
    world: &mut World,
    commands: &mut Commands,
    assets: &AssetServer,
) {
    world.add_hologram(accepts.clone(), commands, assets);
    commands
        .spawn()
        .insert(Output { accepts })
        .insert(LevelEntity);
}

fn run_simulation(
    mut commands: Commands,
    inputs: Query<&Input>,
    outputs: Query<&Output>,
    mut beams: Query<(&mut Transform, &Beam)>,
    mut world: ResMut<World>,
    world_snapshot: Res<WorldSnapshot>,
    mut state: ResMut<SimulationState>,
    time: Res<Time>,
    assets: Res<AssetServer>,
    mut global_state: ResMut<GlobalState>,
    mut interface_state: ResMut<InterfaceState>,
    sfx: Res<Sfx>,
    audio: Res<Audio>,
) {
    if !state.running {
        return;
    }
    state.tick_timer += 4.0 * state.speed * time.delta_seconds();
    if state.tick_timer >= 1.0 {
        // Skip excess ticks if the number is far greater than one.
        state.tick_timer = state.tick_timer % 1.0;
        state.cycles += 1;
    } else {
        return;
    }

    for input in inputs.iter() {
        let mut blocks = input.spawns.blocks.iter();
        let parts = world.parts();
        let should_spawn = !blocks
            .any(|block| any_other_part_contains_block_at(parts, usize::MAX, block.position));
        if should_spawn {
            world.add_part(input.spawns.clone(), &mut commands, &*assets);
        }
    }

    if state.collected_outputs == 10 {
        let level = global_state.current_level;
        let mut num_blocks = 0;
        for part in &world.parts()[interface_state.first_user_part..state.existing_parts] {
            num_blocks += part.structure.blocks.len();
        }
        global_state.last[level] = Some((
            state.cycles as u32 - 1,
            num_blocks as u32,
            (state.existing_parts - interface_state.first_user_part) as u32,
        ));
        let (gs_cycles, gs_num_blocks, gs_parts) =
            global_state.completed[level].get_or_insert((u32::MAX, u32::MAX, u32::MAX));
        *gs_cycles = (*gs_cycles).min(state.cycles as u32 - 1);
        *gs_num_blocks = (*gs_num_blocks).min(num_blocks as u32);
        *gs_parts =
            (*gs_parts).min((state.existing_parts - interface_state.first_user_part) as u32);
        exit_level(
            &mut commands,
            &world_snapshot.0,
            &mut *interface_state,
            &mut *global_state,
            true,
        );
        commands.insert_resource(ChangeToCompleteRequest);
        audio.play_with_settings(
            sfx.level_complete.clone(),
            PlaybackSettings::ONCE.with_volume(0.3),
        );
    }

    let parts: Vec<_> = world.parts().iter().cloned().collect();
    for (_, block) in all_blocks(&parts).filter(|(_, x)| x.kind == BlockKind::WelderBeamSource) {
        let bp = block.position;
        let o = block.facing.offset();
        let (mut transform, _) = beams.iter_mut().find(|x| &x.1.for_block == block).unwrap();
        transform.scale = Vec3::ZERO;
        let mut intersects = HashSet::new();
        for distance in 1..100 {
            let parts = world.parts();
            let position = (
                bp.0 + distance * o.0,
                bp.1 + distance * o.1,
                bp.2 + distance * o.2,
            );
            if let Some(part_index) = find_part_containing_block_at(parts, position) {
                transform.scale = Vec3::new(distance as f32 - 0.5, 1.0, 1.0);
                if part_index < state.existing_parts {
                    if intersects.len() > 1 {
                        world.merge_parts(intersects.iter().copied(), &mut commands, &*assets);
                    }
                    break;
                } else {
                    intersects.insert(part_index);
                }
            } else {
                if intersects.len() > 1 {
                    world.merge_parts(intersects.iter().copied(), &mut commands, &*assets);
                }
                intersects.clear();
            }
        }
    }

    for output in outputs.iter() {
        let parts = world.parts();
        let matching_part_index = parts
            .iter()
            .position(|part| part.structure.matches(&output.accepts) && !part.is_hologram);
        if let Some(matching_part_index) = matching_part_index {
            world.remove_part(matching_part_index, &mut commands);
            state.collected_outputs += 1;
            if state.collected_outputs < 10 {
                audio.play_with_settings(sfx.ding.clone(), PlaybackSettings::ONCE.with_volume(0.5));
            }
        }
    }

    let parts = world.parts();
    let mut states = vec![
        PhysicsState {
            can_move: [false; 6],
            farthest_tractor_beam: [(0, 0); 6]
        };
        parts.len()
    ];
    let directions = BlockFacing::all();

    for (part_containing_tractor_beam, block) in
        all_blocks(parts).filter(|(_, x)| x.kind == BlockKind::TractorBeamSource)
    {
        let pull_direction = block.facing.reverse();
        let pull_direction_index = directions
            .iter()
            .position(|x| *x == pull_direction)
            .unwrap();
        let bp = block.position;
        let o = block.facing.offset();
        let (mut transform, _) = beams.iter_mut().find(|x| &x.1.for_block == block).unwrap();
        transform.scale = Vec3::ZERO;
        for distance in 1..100 {
            let position = (
                bp.0 + distance * o.0,
                bp.1 + distance * o.1,
                bp.2 + distance * o.2,
            );
            if let Some(part_index) = find_part_containing_block_at(parts, position) {
                transform.scale = Vec3::new(distance as f32 - 0.5, 1.0, 1.0);
                if part_index == part_containing_tractor_beam {
                    break;
                }
                let ftb = &mut states[part_index].farthest_tractor_beam[pull_direction_index];
                ftb.0 = ftb.0.max(distance);
                if ftb.0 == distance {
                    ftb.1 = part_containing_tractor_beam;
                }
                break;
            }
        }
    }

    for part_index in 1..parts.len() {
        world.animate_part(part_index, Animation::Stationary, &mut commands);
    }

    let parts = world.parts();
    for part_index in 1..parts.len() {
        if world.parts()[part_index].is_hologram {
            continue;
        }
        let state = &mut states[part_index];
        // Gravity.
        let upwards_pull = state.farthest_tractor_beam[0].0;
        state.farthest_tractor_beam[1].0 = if upwards_pull < 1 {
            i32::MAX
        } else {
            upwards_pull
        };
        let mut directions: Vec<_> = directions.iter().copied().enumerate().collect();
        directions.sort_by_key(|&(idx, _)| -state.farthest_tractor_beam[idx].0);
        for (direction_index, direction) in directions {
            let parts = world.parts();
            let touches = part_touches(parts, part_index, direction);
            let can_move = !touches.contains(&state.farthest_tractor_beam[direction_index].1)
                && !touches.contains(&0);
            if can_move && state.farthest_tractor_beam[direction_index].0 > 1 {
                for part_index in touches.into_iter() {
                    let o = direction.offset();
                    let start = Vec3::new(-o.0 as _, -o.1 as _, -o.2 as _);
                    world.animate_part(
                        part_index,
                        Animation::Lerp(start, Vec3::ZERO),
                        &mut commands,
                    );
                    world.modify_part(
                        part_index,
                        |part| part.translate(o),
                        &mut commands,
                        &*assets,
                    );
                }
                break;
            }
        }
    }
}

fn find_part_containing_block_at(parts: &[Part], position: Position) -> Option<usize> {
    parts.iter().position(|part| {
        !part.is_hologram
            && part
                .structure
                .blocks
                .iter()
                .any(|block| block.position == position)
    })
}

fn all_blocks(parts: &[Part]) -> impl Iterator<Item = (usize, &Block)> {
    parts
        .iter()
        .enumerate()
        .flat_map(|(index, part)| part.structure.blocks.iter().map(move |x| (index, x)))
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            CoreStage::PreUpdate,
            "asdf",
            SystemStage::new(Box::new(ParallelExecutor::default())),
        );
        app.add_system_set_to_stage("asdf", State::<GameState>::get_driver());
        app.add_system_set_to_stage(
            "asdf",
            SystemSet::on_update(GameState::Level).with_system(run_simulation),
        );
    }
}
