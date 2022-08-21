use std::f32::consts::TAU;

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
    utils::HashSet,
};
use bevy_mod_raycast::{
    DefaultPluginState, DefaultRaycastingPlugin, Intersection, RayCastMesh, RayCastMethod,
    RayCastSource, RaycastSystem,
};
use bevy_obj::ObjPlugin;

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
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

#[derive(Component)]
pub struct Cursor;

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

fn setup_world(commands: &mut Commands, assets: &AssetServer) {
    let factory_floor = create_factory_floor();
    let mut world = World::new(factory_floor, commands, assets);
    let blank_structure = Structure { blocks: Vec::new() };
    world.add_machine_part(blank_structure, commands, assets);
    commands.insert_resource(world);
}

fn create_factory_floor() -> Structure {
    let mut factory_floor = Structure { blocks: Vec::new() };
    let size = 20;
    for x in -size..=size {
        for y in -size..=size {
            factory_floor.blocks.push(Block {
                kind: BlockKind::DecoStructure,
                facing: BlockFacing::Pz,
                position: (x, y, -1),
            });
        }
    }
    factory_floor
}

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

fn interface_sys(
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    DecoStructure,
    Structure,
    Activator,
    TractorBeamSource,
    LaserSource,
    LaserSensor,
}

impl BlockKind {
    pub fn asset_name(&self) -> &'static str {
        match self {
            Self::DecoStructure => "blocks/deco_structure.glb#Scene0",
            Self::Structure => "blocks/structure.glb#Scene0",
            Self::Activator => "blocks/activator.glb#Scene0",
            Self::TractorBeamSource => "blocks/tractor_beam_source.glb#Scene0",
            Self::LaserSource => "blocks/laser_source.glb#Scene0",
            Self::LaserSensor => "blocks/laser_sensor.glb#Scene0",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockFacing {
    Px,
    Py,
    Nx,
    Ny,
    Pz,
    Nz,
}

impl BlockFacing {
    pub fn rotation(&self) -> Quat {
        let t4 = TAU / 4.0;
        match self {
            Self::Px => Quat::IDENTITY,
            Self::Py => Quat::from_axis_angle(Vec3::Z, t4),
            Self::Nx => Quat::from_axis_angle(Vec3::Z, 2.0 * t4),
            Self::Ny => Quat::from_axis_angle(Vec3::Z, 3.0 * t4),
            Self::Pz => Quat::from_axis_angle(Vec3::Y, -t4),
            Self::Nz => Quat::from_axis_angle(Vec3::Y, t4),
        }
    }
}

type Position = (i32, i32, i32);

struct Block {
    pub kind: BlockKind,
    pub facing: BlockFacing,
    pub position: Position,
}

struct Structure {
    pub blocks: Vec<Block>,
}

impl Structure {
    pub fn debug_assert_invariants(&self) {
        let mut positions = HashSet::new();
        for block in &self.blocks {
            debug_assert!(
                !positions.contains(&block.position),
                "Structure contains overlapping blocks!"
            );
            positions.insert(block.position);
        }
    }
}

fn spawn_block(commands: &mut Commands, block: &Block, assets: &AssetServer) -> Entity {
    let bbox = assets.load::<Mesh, _>("blocks/bounding_box.obj");
    let scene = assets.load(block.kind.asset_name());
    commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene,
            transform: Transform::from_translation(Vec3::new(
                block.position.0 as f32,
                block.position.1 as f32,
                block.position.2 as f32,
            ))
            .with_rotation(block.facing.rotation()),
            ..Default::default()
        })
        // This will not be rendered since there is no material attached.
        .insert(bbox)
        .insert(RayCastMesh::<BlockRaycastSet>::default())
        .id()
}

fn spawn_structure(structure: &Structure, commands: &mut Commands, assets: &AssetServer) -> Entity {
    let root = commands
        .spawn()
        .insert_bundle(SpatialBundle::default())
        .id();

    for block in &structure.blocks {
        let block = spawn_block(commands, block, assets);
        commands.entity(root).add_child(block);
    }

    root
}

struct World {
    factory_floor: (Structure, Entity),
    machine_parts: Vec<(Structure, Entity)>,
    products: Vec<(Structure, Entity)>,
}

impl World {
    fn debug_assert_invariants(&self) {
        let mut positions = HashSet::new();
        self.factory_floor.0.debug_assert_invariants();
        for block in &self.factory_floor.0.blocks {
            debug_assert!(
                !positions.contains(&block.position),
                "we just checked for this earlier ._."
            );
            positions.insert(block.position);
        }
        for (index, part) in self.machine_parts.iter().enumerate() {
            part.0.debug_assert_invariants();
            for block in &part.0.blocks {
                debug_assert!(
                    !positions.contains(&block.position),
                    "Part {} overlaps with a previous part or the factory floor!",
                    index
                );
                positions.insert(block.position);
            }
        }
        for (index, product) in self.products.iter().enumerate() {
            product.0.debug_assert_invariants();
            for block in &product.0.blocks {
                debug_assert!(
                    !positions.contains(&block.position),
                    "Product {} overlaps with a previous product, part, or the factory floor!",
                    index
                );
                positions.insert(block.position);
            }
        }
    }

    fn new(factory_floor: Structure, commands: &mut Commands, assets: &AssetServer) -> Self {
        let factory_floor_ent = spawn_structure(&factory_floor, commands, assets);
        Self {
            factory_floor: (factory_floor, factory_floor_ent),
            machine_parts: Vec::new(),
            products: Vec::new(),
        }
    }

    fn update_part(
        root: Entity,
        new_structure: &Structure,
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let structure = spawn_structure(new_structure, commands, assets);
        let mut commands = commands.entity(root);
        commands.despawn_descendants();
        commands.add_child(structure);
    }

    pub fn add_machine_part(
        &mut self,
        part: Structure,
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let ent = commands
            .spawn()
            .insert_bundle(SpatialBundle::default())
            .id();
        Self::update_part(ent, &part, commands, assets);
        self.machine_parts.push((part, ent));
        self.debug_assert_invariants();
    }

    pub fn modify_machine_part(
        &mut self,
        index: usize,
        modifier: impl FnOnce(&mut Structure),
        commands: &mut Commands,
        assets: &AssetServer,
    ) {
        let part = &mut self.machine_parts[index];
        modifier(&mut part.0);
        Self::update_part(part.1, &part.0, commands, assets);
        self.debug_assert_invariants();
    }
}

#[derive(Component)]
struct BlockRaycastSet;

fn update_raycast_position_from_cursor(
    mut events: EventReader<CursorMoved>,
    mut source: Query<&mut RayCastSource<BlockRaycastSet>>,
) {
    if let Some(event) = events.iter().last() {
        if let Ok(mut source) = source.get_single_mut() {
            source.cast_method = RayCastMethod::Screenspace(event.position);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(DefaultRaycastingPlugin::<BlockRaycastSet>::default())
        .add_startup_system(startup)
        .add_system(interface_sys)
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_position_from_cursor.before(RaycastSystem::BuildRays::<BlockRaycastSet>),
        )
        .run();
}
