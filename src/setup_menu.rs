use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};

use crate::{
    world::{Position, World},
    GameState, Sfx,
};

pub struct GlobalState {
    pub current_level: usize,
    pub last: [Option<(u32, u32, u32)>; 10],
    pub completed: [Option<(u32, u32, u32)>; 10],
    pub levels: Vec<String>,
}

impl GlobalState {
    pub fn unlocked(&self, index: usize) -> bool {
        let requirements = match index {
            0 => vec![],
            1 => vec![0],
            2 => vec![1],
            3 => vec![2],
            4 => vec![2, 3],
            5 => vec![2, 3],
            6 => vec![4, 5],
            7 => vec![5],
            8 => vec![5],
            9 => vec![7],
            _ => panic!(),
        };
        requirements
            .iter()
            .all(|&req| self.completed[req].is_some())
    }
}

struct MenuState {
    hovers: Vec<(Entity, f32)>,
}

#[derive(Component)]
struct MenuEntity;

fn setup(mut commands: Commands, assets: Res<AssetServer>, global_state: Res<GlobalState>) {
    commands
        .spawn()
        .insert_bundle(Camera2dBundle::default())
        .insert(MenuEntity);
    let menu_bg = assets.load("menu/base.png");
    let root = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                },
                position: UiRect {
                    left: Val::Percent(0.0),
                    bottom: Val::Percent(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MenuEntity)
        .id();
    let bg = commands
        .spawn()
        .insert_bundle(ImageBundle {
            image: UiImage(menu_bg),
            style: Style {
                aspect_ratio: Some(16.0 / 9.0),
                margin: UiRect {
                    bottom: Val::Auto,
                    ..Default::default()
                },
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                },
                position: UiRect {
                    left: Val::Percent(0.0),
                    bottom: Val::Percent(0.0),
                    ..Default::default()
                },
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MenuEntity)
        .id();
    commands.entity(root).add_child(bg);
    let mut state = MenuState { hovers: Vec::new() };
    for index in 0..10 {
        let hover = assets.load(&format!("menu/l{}.png", index));
        let ent = commands
            .spawn()
            .insert_bundle(ImageBundle {
                image: UiImage(hover),
                style: Style {
                    aspect_ratio: Some(16.0 / 9.0),
                    margin: UiRect {
                        bottom: Val::Auto,
                        ..Default::default()
                    },
                    size: Size {
                        width: Val::Percent(100.0),
                        height: Val::Auto,
                    },
                    position: UiRect {
                        left: Val::Percent(0.0),
                        bottom: Val::Percent(0.0),
                        ..Default::default()
                    },
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                color: UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
                ..Default::default()
            })
            .insert(MenuEntity)
            .id();
        commands.entity(root).add_child(ent);
        state.hovers.push((ent, 0.0));
        if let Some((cycles, blocks, parts)) = global_state.completed[index] {
            let pos = [
                (19.0, 77.0),
                (15.5, 51.5),
                (39.5, 75.0),
                (36.0, 49.5),
                (60.5, 73.0),
                (57.0, 47.5),
                (81.0, 71.0),
                (78.0, 45.5),
                (74.5, 20.0),
                (92.0, 25.0),
            ][index];
            let ent = commands
                .spawn()
                .insert_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("{}c\n{}b\n{}p", cycles, blocks, parts),
                            style: TextStyle {
                                font: assets.load("RobotoSlab-Regular.ttf"),
                                font_size: 30.0,
                                color: Color::BLACK,
                            },
                        }],
                        ..Default::default()
                    },
                    style: Style {
                        position: UiRect {
                            left: Val::Percent(pos.0),
                            bottom: Val::Percent(pos.1),
                            ..Default::default()
                        },
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .id();
            commands.entity(root).add_child(ent);
            let ent = commands
                .spawn()
                .insert_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("{}c\n{}b\n{}p", cycles, blocks, parts),
                            style: TextStyle {
                                font: assets.load("RobotoSlab-Regular.ttf"),
                                font_size: 30.0,
                                color: Color::hex("E9C46A").unwrap(),
                            },
                        }],
                        ..Default::default()
                    },
                    style: Style {
                        position: UiRect {
                            left: Val::Percent(pos.0),
                            bottom: Val::Percent(pos.1),
                            ..Default::default()
                        },
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .id();
            commands.entity(root).add_child(ent);
        }
    }
    for index in 1..10 {
        if global_state.unlocked(index) {
            continue;
        }
        let lock = assets.load(&format!("menu/locked{}.png", index));
        let ent = commands
            .spawn()
            .insert_bundle(ImageBundle {
                image: UiImage(lock),
                style: Style {
                    aspect_ratio: Some(16.0 / 9.0),
                    margin: UiRect {
                        bottom: Val::Auto,
                        ..Default::default()
                    },
                    size: Size {
                        width: Val::Percent(100.0),
                        height: Val::Auto,
                    },
                    position: UiRect {
                        left: Val::Percent(0.0),
                        bottom: Val::Percent(0.0),
                        ..Default::default()
                    },
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                color: UiColor(Color::rgba(1.0, 1.0, 1.0, 1.0)),
                ..Default::default()
            })
            .insert(MenuEntity)
            .id();
        commands.entity(root).add_child(ent);
    }
    commands.insert_resource(state);
}

fn update_menu(
    mut commands: Commands,
    mut hovers: Query<&mut UiColor>,
    mut menu_state: ResMut<MenuState>,
    mut global_state: ResMut<GlobalState>,
    time: Res<Time>,
    windows: Res<Windows>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    sfx: Res<Sfx>,
    audio: Res<Audio>,
) {
    let d = time.delta_seconds() * 4.0;
    let win = windows.get_primary().unwrap();
    let width = win.width();
    let mouse_pos = win
        .cursor_position()
        .map(|x| x * 1920.0 / width)
        .unwrap_or(Vec2::new(-1000.0, -1000.0));
    let positions = [
        Vec2::new(125.0, 971.0),
        Vec2::new(60.0, 701.0),
        Vec2::new(525.0, 948.0),
        Vec2::new(455.0, 679.0),
        Vec2::new(921.0, 929.0),
        Vec2::new(852.0, 663.0),
        Vec2::new(1321.0, 910.0),
        Vec2::new(1252.0, 637.0),
        Vec2::new(1185.0, 375.0),
        Vec2::new(1650.0, 618.0),
    ];
    let mouse_pressed = mouse_button_events
        .iter()
        .any(|e| e.button == MouseButton::Left && e.state == ButtonState::Pressed);
    for (index, (entity, opacity)) in menu_state.hovers.iter_mut().enumerate() {
        let size = 0.17 * width;
        let start = positions[index] - Vec2::new(0.0, size);
        let end = positions[index] + Vec2::new(size, 0.0);
        if mouse_pos.cmpge(start).all()
            && mouse_pos.cmple(end).all()
            && global_state.unlocked(index)
        {
            if mouse_pressed {
                commands.insert_resource(ChangeToLevelRequest);
                global_state.current_level = index;
                audio
                    .play_with_settings(sfx.click.clone(), PlaybackSettings::ONCE.with_volume(0.3));
            }
            *opacity += d;
        } else {
            *opacity -= d / 3.0;
        }
        *opacity = (*opacity).clamp(0.0, 1.0);
        hovers.get_mut(*entity).unwrap().0 = Color::rgba(1.0, 1.0, 1.0, *opacity);
    }
}

fn cleanup(mut commands: Commands, entities: Query<Entity, With<MenuEntity>>) {
    commands.remove_resource::<MenuState>();
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct ChangeToLevelRequest;

fn set_state(
    mut commands: Commands,
    request: Option<Res<ChangeToLevelRequest>>,
    mut game_state: ResMut<State<GameState>>,
) {
    if request.is_some() {
        game_state.set(GameState::Level).unwrap();
        commands.remove_resource::<ChangeToLevelRequest>();
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let levels = (0..10)
            .into_iter()
            .map(|index| {
                std::fs::read_to_string(format!("assets/levels/{}.level.txt", index)).unwrap()
            })
            .collect();
        app.insert_resource(GlobalState {
            current_level: 0,
            last: [None; 10],
            completed: [None; 10],
            levels,
        });
        app.add_system_to_stage(CoreStage::First, set_state);
        app.add_system_set_to_stage(
            "asdf",
            SystemSet::on_enter(GameState::Menu).with_system(setup),
        )
        .add_system_set(SystemSet::on_update(GameState::Menu).with_system(update_menu))
        .add_system_set_to_stage(
            "asdf",
            SystemSet::on_exit(GameState::Menu).with_system(cleanup),
        );
    }
}
