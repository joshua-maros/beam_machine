use std::f32::consts::TAU;

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
    utils::HashSet,
};
use bevy_mod_raycast::{
    DefaultRaycastingPlugin, Intersection, RayCastMesh, RayCastMethod, RayCastSource,
    RaycastSystem,
};
use bevy_obj::ObjPlugin;

use crate::{block::{BlockKind, BlockFacing, BlockRaycastSet, Block}, world::{Position, World}};


#[derive(Component)]
pub struct Cursor;

pub enum InterfaceMode {
    Default,
}

pub struct InterfaceState {
    pub mode: InterfaceMode,
    pub movement_keys: [bool; 4],
    pub block_to_place: Option<BlockKind>,
    pub facing: BlockFacing,
    pub holding_shift: bool,
    pub cursor: Entity,
}

fn world_to_block_pos(world: Vec3) -> Position {
    (
        world.x.round() as i32,
        world.y.round() as i32,
        world.z.round() as i32,
    )
}

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
    let (mut cursor_transform, mut cursor_visibility) = cursor.get_single_mut().unwrap();
    cursor_visibility.is_visible = state.block_to_place.is_some();
    let mouse_position = get_mouse_position_in_world(&block_raycast_intersection);
    if let Some((above_cursor, _)) = mouse_position {
        handle_mouse_events(
            &mut commands,
            &mut mouse_button_events,
            above_cursor,
            &mut *world,
            &mut *state,
            &*assets,
        );
        cursor_transform.translation = Vec3::new(
            above_cursor.0 as f32,
            above_cursor.1 as f32,
            above_cursor.2 as f32,
        );
        cursor_transform.rotation = state.facing.rotation();
    } else {
        cursor_visibility.is_visible = false;
    }

    let (mut camera_transform,) = camera.get_single_mut().unwrap();
    move_camera(&mut *camera_transform, state.movement_keys, &*time);
}

fn directional_key_index(event: &KeyboardInput) -> Option<usize> {
    let directional_key = match event.key_code {
        Some(KeyCode::W) => Some(0),
        Some(KeyCode::A) => Some(1),
        Some(KeyCode::S) => Some(2),
        Some(KeyCode::D) => Some(3),
        _ => None,
    };
    directional_key
}

fn update_directional_key(event: &KeyboardInput, state: &mut InterfaceState) {
    if state.block_to_place.is_some() {
        state.movement_keys.fill(false);
        return;
    }
    let directional_key = directional_key_index(event);
    if let Some(key) = directional_key {
        state.movement_keys[key] = event.state == ButtonState::Pressed;
    }
}

fn update_block_keys(event: &KeyboardInput, state: &mut InterfaceState) {
    if event.state != ButtonState::Pressed {
        return;
    }
    match event.key_code {
        Some(KeyCode::Escape) => state.block_to_place = None,
        Some(KeyCode::Key1) => state.block_to_place = Some(BlockKind::Structure),
        Some(KeyCode::Key2) => state.block_to_place = Some(BlockKind::TractorBeamSource),
        _ => (),
    }
    if state.block_to_place.is_some() {
        match event.key_code {
            Some(KeyCode::Q) => state.facing = BlockFacing::Ny,
            Some(KeyCode::W) => state.facing = BlockFacing::Pz,
            Some(KeyCode::E) => state.facing = BlockFacing::Nx,
            Some(KeyCode::A) => state.facing = BlockFacing::Px,
            Some(KeyCode::S) => state.facing = BlockFacing::Nz,
            Some(KeyCode::D) => state.facing = BlockFacing::Py,
            _ => (),
        }
    }
}

fn handle_mouse_events(
    commands: &mut Commands,
    mouse_button_events: &mut EventReader<MouseButtonInput>,
    above_cursor: Position,
    world: &mut World,
    state: &mut InterfaceState,
    assets: &AssetServer,
) {
    for event in mouse_button_events.iter() {
        if event.button == MouseButton::Left && event.state == ButtonState::Released {
            if let (Some(block_to_place)) = (state.block_to_place) {
                // let (above_cursor, _) = get_mouse_position_in_world(intersection);
                place_block(
                    block_to_place,
                    state.facing,
                    world,
                    above_cursor,
                    commands,
                    assets,
                );
                if !state.holding_shift {
                    state.block_to_place = None;
                }
            }
        }
    }
}

fn get_mouse_position_in_world(
    block_raycast_intersection: &Query<(&Intersection<BlockRaycastSet>,)>,
) -> Option<(Position, Position)> {
    if let Ok((intersection,)) = block_raycast_intersection.get_single() {
        if let (Some(&pos), Some(norm)) = (intersection.position(), intersection.normal()) {
            let above_cursor = world_to_block_pos(pos + 0.5 * norm);
            let below_cursor = world_to_block_pos(pos - 0.5 * norm);
            Some((above_cursor, below_cursor))
        } else {
            None
        }
    } else {
        None
    }
}

fn place_block(
    kind: BlockKind,
    facing: BlockFacing,
    world: &mut World,
    above_cursor: (i32, i32, i32),
    commands: &mut Commands,
    assets: &AssetServer,
) {
    world.modify_machine_part(
        0,
        |part| {
            part.blocks.push(Block {
                facing,
                kind,
                position: above_cursor,
            })
        },
        commands,
        assets,
    );
}

fn move_camera(camera_transform: &mut Transform, movement_keys: [bool; 4], time: &Time) {
    let mut total_offset = Vec2::ZERO;
    let offsets = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    for (&active, key_offset) in movement_keys.iter().zip(offsets.into_iter()) {
        if active {
            total_offset.x += key_offset.0;
            total_offset.y += key_offset.1;
        }
    }
    let speed = 10.0;
    camera_transform.translation.x += total_offset.x * speed * time.delta_seconds();
    camera_transform.translation.y += total_offset.y * speed * time.delta_seconds();
}
