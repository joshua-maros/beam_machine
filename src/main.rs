use std::f32::consts::TAU;

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*, core::Zeroable,
};

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    setup_camera(&mut commands);
    setup_light(&mut commands);
    setup_interface_state(&mut commands);

    let scene = assets.load("blocks/tractor_beam_source.glb#Scene0");
    commands.spawn().insert_bundle(SceneBundle {
        scene,
        ..Default::default()
    });
}

fn setup_camera(commands: &mut Commands) {
    commands.spawn().insert_bundle(Camera3dBundle {
        transform: Transform::from_translation((10.0, 10.0, 10.0).into())
            .looking_at((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into()),
        ..Default::default()
    });
}

fn setup_light(commands: &mut Commands) {
    let tau8 = TAU / 8.0;
    commands.spawn().insert_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, tau8, 0.0, tau8)),
        ..Default::default()
    });
}

fn setup_interface_state(commands: &mut Commands) {
    commands.insert_resource(InterfaceState {
        mode: InterfaceMode::Default,
        movement_keys: [false; 4],
    });
}

pub enum InterfaceMode {
    Default,
}

pub struct InterfaceState {
    pub mode: InterfaceMode,
    pub movement_keys: [bool; 4],
}

fn interface_sys(
    mut camera: Query<&mut Transform, With<Camera3d>>,
    mut key_events: EventReader<KeyboardInput>,
    mut state: ResMut<InterfaceState>,
    time: Res<Time>,
) {
    for event in key_events.iter() {
        let directional_key = match event.key_code {
            Some(KeyCode::W) => Some(0),
            Some(KeyCode::A) => Some(1),
            Some(KeyCode::S) => Some(2),
            Some(KeyCode::D) => Some(3),
            _ => None,
        };
        if event.state == ButtonState::Pressed {
            if let Some(key) = directional_key {
                state.movement_keys[key] = true;
            }
        } else {
            debug_assert_eq!(event.state, ButtonState::Released);
            if let Some(key) = directional_key {
                state.movement_keys[key] = false;
            }
        }
    }
    let mut camera_transform = camera.get_single_mut().unwrap();
    move_camera(&mut *camera_transform, state.movement_keys, &*time);
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(interface_sys)
        .run();
}
