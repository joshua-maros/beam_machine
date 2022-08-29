mod base;
mod keys;
mod mouse;
pub mod setup;
mod util;

pub use base::*;
use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
    ui::widget::ImageMode,
};
use bevy_mod_raycast::Intersection;

pub use self::keys::exit_level;
use self::{
    keys::{move_cameras, update_block_keys, update_directional_key},
    mouse::handle_mouse,
};
use crate::{
    block::{Block, BlockFacing, BlockKind, BlockRaycastSet},
    setup::LevelEntity,
    setup_menu::GlobalState,
    simulation::{self, make_input, make_output, SimulationState},
    structure::Structure,
    world::{World, WorldSnapshot},
    GameState, Sfx,
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
    mut simulation_state: ResMut<SimulationState>,
    mut world: ResMut<World>,
    mut world_snapshot: ResMut<WorldSnapshot>,
    assets: Res<AssetServer>,
    time: Res<Time>,
    mut global_state: ResMut<GlobalState>,
    windows: Res<Windows>,
    sfx: Res<Sfx>,
    audio: Res<Audio>,
) {
    for event in key_events.iter() {
        update_directional_key(
            &mut commands,
            event,
            &mut *state,
            &*simulation_state,
            &mut *global_state,
            if simulation_state.is_started() {
                &world_snapshot.0
            } else {
                &*world
            },
        );
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
    let mut clicked = false;
    for event in mouse_button_events.iter() {
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            clicked = true;
        }
    }
    let ui_captured_click = handle_ui(
        &mut commands,
        &mut *world,
        &mut *world_snapshot,
        simulation_state.is_started(),
        &mut *state,
        &mut *simulation_state,
        &mut *global_state,
        &*windows,
        clicked,
        &*assets,
    );
    handle_mouse(
        &mut cursor,
        &mut *state,
        &*simulation_state,
        block_raycast_intersection,
        &mut commands,
        clicked && !ui_captured_click,
        &mut *world,
        &*assets,
        &*sfx,
        &*audio,
    );

    move_cameras(cameras.iter_mut(), state.movement_keys, &*time);
    delete_ui(&mut commands, state.ui_root);
    let new_ui_root = make_ui(
        &mut commands,
        &*assets,
        &*state,
        &*simulation_state,
        &*global_state,
    );
    state.ui_root = new_ui_root;
}

fn handle_ui(
    commands: &mut Commands,
    world: &mut World,
    world_snapshot: &mut WorldSnapshot,
    is_started: bool,
    state: &mut InterfaceState,
    simulation_state: &mut SimulationState,
    global_state: &mut GlobalState,
    windows: &Windows,
    clicked: bool,
    assets: &AssetServer,
) -> bool {
    let window = windows.primary();
    let height = window.height();
    let cursor_pos = window
        .cursor_position()
        .map(|x| x * 720.0 / height)
        .unwrap_or(Vec2::new(-1000.0, -1000.0));
    let cep = &mut state.currently_editing_part;
    if !clicked {
        false
    } else if cursor_pos.clamp((2.0, 671.0).into(), (76.0, 718.0).into()) == cursor_pos {
        exit_level(
            commands,
            if is_started {
                &world_snapshot.0
            } else {
                &*world
            },
            state,
            global_state,
            false,
        );
        true
    } else if cursor_pos.clamp((286.0, 671.0).into(), (361.0, 718.0).into()) == cursor_pos {
        if EDITING {
            if *cep > 0 {
                *cep -= 1;
            }
        } else {
            *cep -= 1;
            *cep = (*cep).max(state.first_user_part);
        }
        true
    } else if cursor_pos.clamp((426.0, 671.0).into(), (505.0, 718.0).into()) == cursor_pos {
        *cep += 1;
        if world.parts().len() <= *cep {
            let s = Structure { blocks: Vec::new() };
            if EDITING {
                world.add_hologram(s, commands, assets);
            } else {
                world.add_part(s, commands, assets);
            }
        }
        true
    } else if cursor_pos.clamp((6.0, 0.0).into(), (148.0, 94.0).into()) == cursor_pos {
        state.block_to_place = Some(BlockKind::Structure);
        true
    } else if cursor_pos.clamp((148.0, 0.0).into(), (261.0, 94.0).into()) == cursor_pos {
        state.block_to_place = Some(BlockKind::TractorBeamSource);
        true
    } else if cursor_pos.clamp((261.0, 0.0).into(), (398.0, 94.0).into()) == cursor_pos {
        state.block_to_place = Some(BlockKind::WelderBeamSource);
        true
    } else if cursor_pos.clamp((33.0, 100.0).into(), (98.0, 148.0).into()) == cursor_pos {
        simulation::end_simulation(world, world_snapshot, simulation_state, commands, assets);
        true
    } else if cursor_pos.clamp((98.0, 100.0).into(), (157.0, 148.0).into()) == cursor_pos {
        simulation::begin_simulation(
            world,
            world_snapshot,
            simulation_state,
            0.3,
            commands,
            assets,
        );
        true
    } else if cursor_pos.clamp((157.0, 100.0).into(), (215.0, 148.0).into()) == cursor_pos {
        simulation::begin_simulation(
            world,
            world_snapshot,
            simulation_state,
            1.0,
            commands,
            assets,
        );
        true
    } else if cursor_pos.clamp((215.0, 100.0).into(), (275.0, 148.0).into()) == cursor_pos {
        simulation::begin_simulation(
            world,
            world_snapshot,
            simulation_state,
            3.0,
            commands,
            assets,
        );
        true
    } else {
        false
    }
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
                simulation::begin_simulation(
                    &mut *world,
                    &mut *snapshot,
                    &mut *simulation_state,
                    1.0,
                    &mut commands,
                    &*assets,
                );
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

#[must_use]
pub fn import_level(
    input: &str,
    world: &mut World,
    commands: &mut Commands,
    assets: &AssetServer,
) -> usize {
    let mut lines = input.lines();
    let mut start = lines.next().unwrap().split(" ");
    let min_x: i32 = start.next().unwrap().parse().unwrap();
    let min_y: i32 = start.next().unwrap().parse().unwrap();
    let min_z: i32 = start.next().unwrap().parse().unwrap();
    assert_eq!(lines.next(), Some("floor"));
    let mut mode = "floor";
    let mut current_structure = Structure { blocks: Vec::new() };
    let mut first_user_part = 1;
    let mut create_structure = move |mode, structure| match mode {
        "floor" => world.add_part(structure, commands, assets),
        "input" => make_input(structure, world, commands, assets),
        "output" => make_output(structure, world, commands, assets),
        "part" => world.add_part(structure, commands, assets),
        _ => panic!(),
    };
    let mut position = (min_x - 1, min_y, min_z);
    for line in lines {
        if line == "input" || line == "output" || line == "part" {
            if line != "part" {
                first_user_part += 1;
            }
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
                        other => panic!("{}", other),
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
    first_user_part
}

fn export_level(world: &World, first_user_part: usize) -> String {
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
        } else if index == first_user_part - 1 {
            output.push_str("output\n");
        } else if index < first_user_part {
            output.push_str("input\n");
        } else {
            output.push_str("part\n");
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
    output
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

pub struct ChangeToCompleteRequest;

fn set_state(
    mut commands: Commands,
    request: Option<Res<ChangeToCompleteRequest>>,
    mut game_state: ResMut<State<GameState>>,
) {
    if request.is_some() {
        game_state.set(GameState::Complete).unwrap();
        commands.remove_resource::<ChangeToCompleteRequest>();
    }
}

fn make_parts_bar(commands: &mut Commands, assets: &AssetServer, state: &InterfaceState) -> Entity {
    let root = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                position: UiRect {
                    bottom: Val::Percent(93.0),
                    left: Val::Percent(0.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(7.0),
                },
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .id();
    let parts_label_container = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                aspect_ratio: Some(1080.0 / 108.0),
                size: Size {
                    width: Val::Auto,
                    height: Val::Percent(100.0),
                },
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .id();
    commands.entity(root).add_child(parts_label_container);
    let parts_label = commands
        .spawn()
        .insert_bundle(ImageBundle {
            image: UiImage(assets.load("icons/parts.png")),
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                min_size: Size {
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands
        .entity(parts_label_container)
        .add_child(parts_label);
    let part_index = state.currently_editing_part + 1 - state.first_user_part;
    let parts_number = commands
        .spawn()
        .insert_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: format!("{}", part_index),
                    style: TextStyle {
                        font: assets.load("RobotoSlab-Regular.ttf"),
                        font_size: 55.0,
                        ..Default::default()
                    },
                }],
                alignment: TextAlignment::BOTTOM_CENTER,
            },
            style: Style {
                position: UiRect {
                    left: if part_index >= 10 {
                        Val::Percent(74.0)
                    } else {
                        Val::Percent(76.0)
                    },
                    bottom: Val::Percent(-4.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands.entity(parts_label).add_child(parts_number);
    root
}

fn make_hotbar(
    commands: &mut Commands,
    assets: &AssetServer,
    state: &InterfaceState,
    simulation_state: &SimulationState,
) -> Entity {
    let root = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                position: UiRect {
                    bottom: Val::Percent(0.0),
                    left: Val::Percent(0.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(21.0),
                },
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .id();
    let hotbar_container = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                aspect_ratio: Some(1080.0 / 324.0),
                size: Size {
                    width: Val::Auto,
                    height: Val::Percent(100.0),
                },
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .id();
    commands.entity(root).add_child(hotbar_container);
    let parts_label = commands
        .spawn()
        .insert_bundle(ImageBundle {
            image: UiImage(assets.load("icons/hotbar.png")),
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                min_size: Size {
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands.entity(hotbar_container).add_child(parts_label);
    let parts_number = commands
        .spawn()
        .insert_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: format!("{}/10", simulation_state.collected_outputs),
                    style: TextStyle {
                        font: assets.load("RobotoSlab-Regular.ttf"),
                        font_size: 55.0,
                        ..Default::default()
                    },
                }],
                alignment: TextAlignment::BOTTOM_CENTER,
            },
            style: Style {
                position: UiRect {
                    left: Val::Percent(58.0),
                    bottom: Val::Percent(65.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands.entity(parts_label).add_child(parts_number);
    root
}

fn make_hint_box(
    commands: &mut Commands,
    assets: &AssetServer,
    state: &InterfaceState,
    global_state: &GlobalState,
    simulation_state: &SimulationState,
) -> Entity {
    let root = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                position: UiRect {
                    bottom: Val::Percent(25.0),
                    left: Val::Percent(70.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Percent(30.0),
                    height: Val::Percent(50.0),
                },
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .id();
    let hint_text = match global_state.current_level {
        0 => "The goal of this game is to take\nthe inputs (on the blue squares)\nand turn them into the outputs \n(on the red squares).\n\nPlace a tractor beam to do so.\nPress play once you are ready to \nsimulate the machine you built.",
        1 => "Use WASD to look around.\n\nGravity is stronger than tractor \nbeams. Use structure blocks to \nbridge the gap. Make sure all \nthe blocks are in the same part, \nor they will separate!\n\nHold shift to place multiple \nblocks at a time.",
        2 => "Welder beams attach adjacent \nblocks together.\n\nUse QWEASD to change the \norientation of what you're \nplacing.",
        3 => "If multiple tractor beams are \npulling an object, the beam that \nis the longest will win out.",
        5 => "Tractor beams can only pull a \nsingle part at a time, but moving \nparts can push multiple parts\nat a time.",
        _ => "",
    };
    if hint_text.is_empty() {
        return root;
    }
    let hint_container = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                ..Default::default()
            },
            color: UiColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
            ..Default::default()
        })
        .id();
    commands.entity(root).add_child(hint_container);
    let hint = commands
        .spawn()
        .insert_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: hint_text.to_owned(),
                    style: TextStyle {
                        font: assets.load("RobotoSlab-Regular.ttf"),
                        font_size: 30.0,
                        ..Default::default()
                    },
                }],
                alignment: TextAlignment::BOTTOM_LEFT,
            },
            style: Style {
                position: UiRect {
                    left: Val::Percent(5.0),
                    bottom: Val::Percent(5.0),
                    // right: Val::Percent(100.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands.entity(hint_container).add_child(hint);
    root
}

pub fn make_ui(
    commands: &mut Commands,
    assets: &AssetServer,
    state: &InterfaceState,
    simulation_state: &SimulationState,
    global_state: &GlobalState,
) -> Entity {
    let root = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                position: UiRect {
                    bottom: Val::Percent(0.0),
                    left: Val::Percent(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .id();
    let parts_bar = make_parts_bar(commands, assets, state);
    commands.entity(root).add_child(parts_bar);
    let hotbar = make_hotbar(commands, assets, state, simulation_state);
    commands.entity(root).add_child(hotbar);
    let hint_box = make_hint_box(commands, assets, state, global_state, simulation_state);
    commands.entity(root).add_child(hint_box);
    root
}

pub fn delete_ui(commands: &mut Commands, root: Entity) {
    commands.entity(root).despawn_recursive();
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
