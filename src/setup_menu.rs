use bevy::prelude::*;

use crate::GameState;

struct MenuState {
    hovers: Vec<(Entity, f32)>,
}

#[derive(Component)]
struct MenuEntity;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
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
        state.hovers.push((ent, 0.0));
        commands.entity(root).add_child(ent);
    }
    commands.insert_resource(state);
}

fn update_menu(
    mut hovers: Query<&mut UiColor>,
    mut state: ResMut<MenuState>,
    time: Res<Time>,
    windows: Res<Windows>,
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
    for (index, (entity, opacity)) in state.hovers.iter_mut().enumerate() {
        let size = 0.17 * width;
        let start = positions[index] - Vec2::new(0.0, size);
        let end = positions[index] + Vec2::new(size, 0.0);
        if mouse_pos.cmpge(start).all() && mouse_pos.cmple(end).all() {
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

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
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
