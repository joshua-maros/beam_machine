use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_mod_raycast::RayCastSource;

use crate::{
    block::{BlockFacing, BlockRaycastSet},
    interface::{Cursor, InterfaceMode, InterfaceState},
    world::setup_world,
};

pub fn startup_system(mut commands: Commands, assets: Res<AssetServer>) {
    setup_camera(&mut commands);
    setup_light(&mut commands);
    setup_interface_state(&mut commands, &*assets);
    setup_world(&mut commands, &*assets);
}

fn setup_camera(commands: &mut Commands) {
    commands
        .spawn()
        .insert_bundle(Camera3dBundle {
            transform: Transform::from_translation((10.0, 10.0, 10.0).into())
                .looking_at((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into()),
            ..Default::default()
        })
        .insert(RayCastSource::<BlockRaycastSet>::default());
}

fn setup_light(commands: &mut Commands) {
    let tau8 = TAU / 8.0;
    commands.spawn().insert_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, tau8, 0.0, tau8)),
        ..Default::default()
    });
}

fn setup_interface_state(commands: &mut Commands, assets: &AssetServer) {
    let scene = assets.load("blocks/cursor.glb#Scene0");
    let cursor = commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            ..Default::default()
        })
        .insert(Cursor)
        .id();
    commands.insert_resource(InterfaceState {
        mode: InterfaceMode::Default,
        movement_keys: [false; 4],
        block_to_place: None,
        facing: BlockFacing::Px,
        holding_shift: false,
        cursor,
    });
}
