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
    world_snapshot: Res<WorldSnapshot>,
    assets: Res<AssetServer>,
    time: Res<Time>,
    mut global_state: ResMut<GlobalState>,
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
    delete_ui(&mut commands, state.ui_root);
    let new_ui_root = make_ui(&mut commands, &*assets, &*state);
    state.ui_root = new_ui_root;
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
                aspect_ratio: Some(926.0 / 108.0),
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
                        Val::Percent(69.0)
                    } else {
                        Val::Percent(72.0)
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

pub fn make_ui(commands: &mut Commands, assets: &AssetServer, state: &InterfaceState) -> Entity {
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
