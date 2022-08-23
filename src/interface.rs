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
    keys::{move_camera, update_block_keys, update_directional_key},
    mouse::handle_mouse,
};
use crate::{
    block::BlockRaycastSet,
    simulation::{self, SimulationState},
    structure::Structure,
    world::{World, WorldSnapshot},
};

pub fn interface_system(
    mut commands: Commands,
    mut camera: Query<(&mut Transform,), (With<Camera3d>, Without<Cursor>)>,
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
        update_directional_key(event, &mut *state, &*simulation_state);
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

    let (mut camera_transform,) = camera.get_single_mut().unwrap();
    move_camera(&mut *camera_transform, state.movement_keys, &*time);
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
            if simulation_state.is_started() {
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
                world.add_part(Structure { blocks: Vec::new() }, &mut commands, &*assets);
            }
            println!("{:#?}", *cep);
        } else if event.key_code == Some(KeyCode::Minus) && event.state == ButtonState::Pressed {
            *cep -= 1;
            *cep = (*cep).max(state.first_user_part);
            println!("{:#?}", *cep);
        }
    }
}

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(interface_system)
            .add_system(simulation_interface_system)
            .add_system(switch_part_system);
    }
}
