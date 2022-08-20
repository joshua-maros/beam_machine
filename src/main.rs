use std::f32::consts::TAU;

use bevy::prelude::*;

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn().insert_bundle(Camera3dBundle {
        transform: Transform::from_translation((10.0, 10.0, 10.0).into())
            .looking_at((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into()),
        ..Default::default()
    });

    let tau8 = TAU / 8.0;
    commands.spawn().insert_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, tau8, 0.0, tau8)),
        ..Default::default()
    });

    let scene = assets.load("blocks/tractor_beam_source.glb#Scene0");
    commands.spawn().insert_bundle(SceneBundle {
        scene,
        ..Default::default()
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .run();
}
