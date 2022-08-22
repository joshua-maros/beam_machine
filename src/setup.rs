use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_mod_raycast::RayCastSource;

use crate::block::BlockRaycastSet;

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    setup_camera(&mut commands);
    setup_light(&mut commands);
    crate::interface::setup::setup_interface_state(&mut commands, &*assets);
    crate::world::setup::setup_world(&mut commands, &*assets);
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
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, tau8, 0.0, tau8)),
        ..Default::default()
    });
}
