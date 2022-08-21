mod base;
mod keys;
mod mouse;
pub mod setup;
mod util;

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_mod_raycast::Intersection;

use crate::{block::BlockRaycastSet, world::World};

pub use base::*;

use self::{
    keys::{move_camera, update_block_keys, update_directional_key},
    mouse::handle_mouse,
};

pub fn interface_system(
    mut commands: Commands,
    mut camera: Query<(&mut Transform,), (With<Camera3d>, Without<Cursor>)>,
    mut cursor: Query<(&mut Transform, &mut Visibility), (With<Cursor>, Without<Camera3d>)>,
    block_raycast_intersection: Query<(&Intersection<BlockRaycastSet>,)>,
    mut key_events: EventReader<KeyboardInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut state: ResMut<InterfaceState>,
    mut world: ResMut<World>,
    assets: Res<AssetServer>,
    time: Res<Time>,
) {
    for event in key_events.iter() {
        update_directional_key(event, &mut *state);
        update_block_keys(event, &mut *state);
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
        block_raycast_intersection,
        &mut commands,
        &mut mouse_button_events,
        &mut *world,
        &*assets,
    );

    let (mut camera_transform,) = camera.get_single_mut().unwrap();
    move_camera(&mut *camera_transform, state.movement_keys, &*time);
}
