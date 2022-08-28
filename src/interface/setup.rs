use bevy::{input::keyboard::KeyboardInput, prelude::*};

use super::{make_ui, Cursor, InterfaceMode, InterfaceState, EDITING};
use crate::{block::BlockFacing, setup::LevelEntity, simulation::SimulationState, setup_menu::GlobalState};

pub fn setup_interface_state(
    commands: &mut Commands,
    assets: &AssetServer,
    simulation_state: &SimulationState,
    first_user_part: usize,
    global_state: &GlobalState,
) {
    let scene = assets.load("blocks/cursor.glb#Scene0");
    let place_cursor = commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            ..Default::default()
        })
        .insert(Cursor)
        .insert(LevelEntity)
        .id();
    let scene = assets.load("blocks/remove_cursor.glb#Scene0");
    let remove_cursor = commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            ..Default::default()
        })
        .insert(Cursor)
        .insert(LevelEntity)
        .id();
    let ui_root = commands.spawn().insert(LevelEntity).id();
    let mut state = InterfaceState {
        mode: InterfaceMode::Default,
        movement_keys: [false; 4],
        first_user_part,
        currently_editing_part: if EDITING { 0 } else { first_user_part },
        block_to_place: None,
        facing: BlockFacing::Nx,
        holding_shift: false,
        place_cursor,
        remove_cursor,
        ui_root,
    };
    let ui_root = make_ui(commands, assets, &state, simulation_state, global_state);
    state.ui_root = ui_root;
    commands.insert_resource(state);
}
