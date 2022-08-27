use std::f32::consts::TAU;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::{CameraRenderGraph, RenderTarget},
        render_resource::{
            AsBindGroup, Extent3d, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use bevy_mod_raycast::RayCastSource;

use crate::{
    block::BlockRaycastSet, interface::InterfaceState, setup_menu::GlobalState,
    simulation::SimulationState, GameState,
};

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut windows: ResMut<Windows>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PostProcessMaterial>>,
    global_state: Res<GlobalState>,
) {
    let (target, holo_target, size) = setup_render_targets(&mut commands, &*windows, &mut *images);
    setup_cameras(
        &mut commands,
        &mut *meshes,
        &mut *materials,
        target,
        holo_target,
        size,
    );
    setup_light(&mut commands);
    let first_user_part = crate::world::setup::setup_world(&mut commands, &*assets, &*global_state);
    crate::interface::setup::setup_interface_state(&mut commands, &*assets, first_user_part);
    commands.insert_resource(SimulationState {
        started: false,
        running: false,
        tick_timer: 0.0,
        existing_parts: 0,
        collected_outputs: 0,
        cycles: 0,
    });
}

fn setup_render_targets(
    commands: &mut Commands,
    windows: &Windows,
    images: &mut Assets<Image>,
) -> (Handle<Image>, Handle<Image>, Extent3d) {
    let window = windows.primary();
    let size = Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        ..Default::default()
    };
    let mut image1 = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..Default::default()
    };
    image1.resize(size);
    let image2 = image1.clone();
    (images.add(image1), images.add(image2), size)
}

fn setup_cameras(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<PostProcessMaterial>,
    normal_render_target: Handle<Image>,
    holo_render_target: Handle<Image>,
    size: Extent3d,
) {
    setup_main_camera(commands, normal_render_target.clone());
    setup_holographic_camera(commands, holo_render_target.clone());
    setup_post_process_camera(
        commands,
        meshes,
        materials,
        normal_render_target,
        holo_render_target,
        size,
    );
}

fn setup_main_camera(commands: &mut Commands, render_target: Handle<Image>) {
    commands
        .spawn()
        .insert_bundle(Camera3dBundle {
            transform: Transform::from_translation((10.0, 10.0, 10.0).into())
                .looking_at((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into()),
            camera: Camera {
                target: RenderTarget::Image(render_target),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(LevelEntity)
        .insert(RenderLayers::from_layers(&[0]))
        .insert(RayCastSource::<BlockRaycastSet>::default());
}

fn setup_holographic_camera(commands: &mut Commands, render_target: Handle<Image>) {
    commands
        .spawn()
        .insert_bundle(Camera3dBundle {
            transform: Transform::from_translation((10.0, 10.0, 10.0).into())
                .looking_at((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into()),
            camera: Camera {
                target: RenderTarget::Image(render_target),
                ..Default::default()
            },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(LevelEntity)
        .insert(RenderLayers::from_layers(&[1]));
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "80b90683-5b74-48b9-a121-5ab31d0c44b5"]
pub struct PostProcessMaterial {
    #[texture(0)]
    #[sampler(1)]
    render_target: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    holo_render_target: Handle<Image>,
}

impl Material2d for PostProcessMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/hologram.wgsl".into()
    }
}

fn setup_post_process_camera(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<PostProcessMaterial>,
    render_target: Handle<Image>,
    holo_render_target: Handle<Image>,
    size: Extent3d,
) {
    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        size.width as f32,
        size.height as f32,
    ))));
    let material_handle = materials.add(PostProcessMaterial {
        render_target,
        holo_render_target,
    });
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        })
        .insert(LevelEntity)
        .insert(post_processing_pass_layer);
    commands
        .spawn()
        .insert_bundle(Camera2dBundle {
            camera: Camera {
                priority: 2,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(LevelEntity)
        .insert(post_processing_pass_layer);
}

fn setup_light(commands: &mut Commands) {
    let tau8 = TAU / 8.0;
    commands
        .spawn()
        .insert_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 10_000.0,
                ..Default::default()
            },
            transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, tau8, 0.0, tau8)),
            ..Default::default()
        })
        .insert(LevelEntity);
}

#[derive(Component)]
pub struct LevelEntity;

fn cleanup(mut commands: Commands, entities: Query<Entity, With<LevelEntity>>) {
    commands.remove_resource::<InterfaceState>();
    commands.remove_resource::<SimulationState>();
    commands.remove_resource::<World>();
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<PostProcessMaterial>::default())
            .add_system_set_to_stage(
                "asdf",
                SystemSet::on_enter(GameState::Level).with_system(setup),
            )
            .add_system_set_to_stage(
                "asdf",
                SystemSet::on_exit(GameState::Level).with_system(cleanup),
            );
    }
}
