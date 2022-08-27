mod base;
mod keys;
mod mouse;
pub mod setup;
mod util;

pub use base::*;
use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_mod_raycast::Intersection;

use self::{
    keys::{move_cameras, update_block_keys, update_directional_key},
    mouse::handle_mouse,
};
use crate::{
    block::{Block, BlockFacing, BlockKind, BlockRaycastSet},
    setup_menu::GlobalState,
    simulation::{self, make_input, make_output, SimulationState},
    structure::Structure,
    world::{World, WorldSnapshot},
    GameState,
};

pub const EDITING: bool = false;

pub fn interface_system(
    mut commands: Commands,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<Cursor>)>,
    mut cursor: Query<(&mut Transform, &mut Visibility), (With<Cursor>, Without<Camera3d>)>,
    block_raycast_intersection: Query<(&Intersection<BlockRaycastSet>,)>,
    mut key_events: EventReader<KeyboardInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut state: ResMut<InterfaceState>,
    simulation_state: Res<SimulationState>,
    mut world: ResMut<World>,
    assets: Res<AssetServer>,
    time: Res<Time>,
) {
    for event in key_events.iter() {
        update_directional_key(&mut commands, event, &mut *state, &*simulation_state);
        update_block_keys(event, &mut *state, &*simulation_state);
        if event.key_code == Some(KeyCode::LShift) || event.key_code == Some(KeyCode::RShift) {
            if event.state == ButtonState::Pressed {
                state.holding_shift = true;
            } else {
                state.block_to_place = None;
                state.holding_shift = false;
            }
        }
    }
    handle_mouse(
        &mut cursor,
        &mut *state,
        &*simulation_state,
        block_raycast_intersection,
        &mut commands,
        &mut mouse_button_events,
        &mut *world,
        &*assets,
    );

    move_cameras(cameras.iter_mut(), state.movement_keys, &*time);
}

pub fn simulation_interface_system(
    mut commands: Commands,
    mut key_events: EventReader<KeyboardInput>,
    mut simulation_state: ResMut<SimulationState>,
    mut world: ResMut<World>,
    mut snapshot: ResMut<WorldSnapshot>,
    assets: Res<AssetServer>,
) {
    for event in key_events.iter() {
        if event.key_code == Some(KeyCode::Space) && event.state == ButtonState::Pressed {
            if EDITING {
                // export_level(&*world);
            } else if simulation_state.is_started() {
                simulation::end_simulation(
                    &mut *world,
                    &mut *snapshot,
                    &mut *simulation_state,
                    &mut commands,
                    &*assets,
                );
            } else {
                simulation::begin_simulation(&mut *world, &mut *snapshot, &mut *simulation_state);
            }
        }
    }
}

fn export_block(block: &Block) -> String {
    let c = match block.kind {
        BlockKind::DecoStructure => '#',
        BlockKind::DecoStructure2 => 'x',
        BlockKind::DecoStructureInput => 'i',
        BlockKind::DecoStructureOutput => 'o',
        BlockKind::Structure => 's',
        BlockKind::Activator => 'a',
        BlockKind::TractorBeamSource => 't',
        BlockKind::WelderBeamSource => 'w',
        BlockKind::LaserSource => 'l',
        BlockKind::LaserSensor => 'n',
    };
    let f = match block.facing {
        BlockFacing::Px => '0',
        BlockFacing::Py => '1',
        BlockFacing::Nx => '2',
        BlockFacing::Ny => '3',
        BlockFacing::Pz => '4',
        BlockFacing::Nz => '5',
    };
    format!("{}{}", c, f)
}

pub fn import_level(
    world: &mut World,
    commands: &mut Commands,
    assets: &AssetServer,
    global_state: &GlobalState,
) {
    let input = std::fs::read_to_string(format!(
        "assets/levels/{}.level.txt",
        global_state.current_level
    ))
    .unwrap();
    let mut lines = input.lines();
    let mut start = lines.next().unwrap().split(" ");
    let min_x: i32 = start.next().unwrap().parse().unwrap();
    let min_y: i32 = start.next().unwrap().parse().unwrap();
    let min_z: i32 = start.next().unwrap().parse().unwrap();
    assert_eq!(lines.next(), Some("floor"));
    let mut mode = "floor";
    let mut current_structure = Structure { blocks: Vec::new() };
    let mut create_structure = move |mode, structure| match mode {
        "floor" => world.add_part(structure, commands, assets),
        "input" => make_input(structure, world, commands, assets),
        "output" => make_output(structure, world, commands, assets),
        _ => panic!(),
    };
    let mut position = (min_x - 1, min_y, min_z);
    for line in lines {
        if line == "input" || line == "output" {
            create_structure(mode, current_structure);
            current_structure = Structure { blocks: Vec::new() };
            mode = line;
            position = (min_x - 1, min_y, min_z);
        } else if line == "" {
            position.1 = min_y;
            position.2 += 1;
        } else {
            for pair in line.chars().collect::<Vec<_>>().chunks(2) {
                position.0 += 1;
                if let &[c, f] = pair {
                    let kind = match c {
                        '.' => continue,
                        '#' => BlockKind::DecoStructure,
                        'x' => BlockKind::DecoStructure2,
                        'i' => BlockKind::DecoStructureInput,
                        'o' => BlockKind::DecoStructureOutput,
                        's' => BlockKind::Structure,
                        'a' => BlockKind::Activator,
                        't' => BlockKind::TractorBeamSource,
                        'w' => BlockKind::WelderBeamSource,
                        'l' => BlockKind::LaserSource,
                        'n' => BlockKind::LaserSensor,
                        _ => panic!(),
                    };
                    let facing = match f {
                        '0' => BlockFacing::Px,
                        '1' => BlockFacing::Py,
                        '2' => BlockFacing::Nx,
                        '3' => BlockFacing::Ny,
                        '4' => BlockFacing::Pz,
                        '5' => BlockFacing::Nz,
                        _ => panic!(),
                    };
                    current_structure.blocks.push(Block {
                        kind,
                        facing,
                        position,
                    });
                } else {
                    panic!()
                }
            }
            position.0 = min_x - 1;
            position.1 += 1;
        }
    }
    create_structure(mode, current_structure);
}

fn export_level(world: &World, global_state: &GlobalState) {
    let mut min = (i32::MAX, i32::MAX, i32::MAX);
    let mut max = (i32::MIN, i32::MIN, i32::MIN);
    for part in world.parts() {
        for block in &part.structure.blocks {
            min.0 = min.0.min(block.position.0);
            min.1 = min.1.min(block.position.1);
            min.2 = min.2.min(block.position.2);
            max.0 = max.0.max(block.position.0);
            max.1 = max.1.max(block.position.1);
            max.2 = max.2.max(block.position.2);
        }
    }
    let mut output = String::new();
    for (index, part) in world.parts().iter().enumerate() {
        if index == 0 {
            output.push_str(&format!("{} {} {}\n", min.0, min.1, min.2));
            output.push_str("floor\n");
        } else if index == world.parts().len() - 1 {
            output.push_str("output\n");
        } else {
            output.push_str("input\n");
        }
        for z in min.2..=max.2 {
            for y in min.1..=max.1 {
                for x in min.0..=max.0 {
                    if let Some(block) = part.structure.get_block_at((x, y, z)) {
                        output.push_str(&export_block(block));
                    } else {
                        output.push_str(".0");
                    }
                }
                output.push_str("\n");
            }
            output.push_str("\n");
        }
    }
    std::fs::write(
        format!("assets/levels/{}.level.txt", global_state.current_level),
        output,
    )
    .unwrap();
    println!("Wrote level!");
}

pub fn switch_part_system(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut state: ResMut<InterfaceState>,
    mut key_events: EventReader<KeyboardInput>,
    assets: Res<AssetServer>,
) {
    let state = &mut *state;
    let cep = &mut state.currently_editing_part;
    for event in key_events.iter() {
        if event.key_code == Some(KeyCode::Equals) && event.state == ButtonState::Pressed {
            *cep += 1;
            if world.parts().len() <= *cep {
                let s = Structure { blocks: Vec::new() };
                if EDITING {
                    world.add_hologram(s, &mut commands, &*assets);
                } else {
                    world.add_part(s, &mut commands, &*assets);
                }
            }
        } else if event.key_code == Some(KeyCode::Minus) && event.state == ButtonState::Pressed {
            if EDITING {
                if *cep > 0 {
                    *cep -= 1;
                }
            } else {
                *cep -= 1;
                *cep = (*cep).max(state.first_user_part);
            }
        }
    }
}

pub struct ChangeToMenuRequest;

fn set_state(
    mut commands: Commands,
    request: Option<Res<ChangeToMenuRequest>>,
    mut game_state: ResMut<State<GameState>>,
) {
    if request.is_some() {
        game_state.set(GameState::Menu).unwrap();
        commands.remove_resource::<ChangeToMenuRequest>();
    }
}

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, set_state);
        app.add_system_set_to_stage(
            "asdf",
            SystemSet::on_update(GameState::Level)
                .with_system(interface_system)
                .with_system(simulation_interface_system)
                .with_system(switch_part_system),
        );
    }
}
