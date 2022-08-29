use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    setup_menu::GlobalState,
    world::{Position, World},
    GameState, Sfx,
};

#[derive(Component)]
struct CompleteEntity;

fn setup(mut commands: Commands, assets: Res<AssetServer>, global_state: Res<GlobalState>) {
    let (this_cycles, this_blocks, this_parts) =
        global_state.last[global_state.current_level].unwrap();
    let (high_cycles, high_blocks, high_parts) =
        global_state.completed[global_state.current_level].unwrap();
    commands
        .spawn()
        .insert_bundle(Camera2dBundle::default())
        .insert(CompleteEntity);
    let root = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                position: UiRect {
                    left: Val::Percent(0.0),
                    bottom: Val::Percent(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: UiColor(Color::BLACK),
            ..Default::default()
        })
        .insert(CompleteEntity)
        .id();
    let bg = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                position: UiRect {
                    left: Val::Percent(0.0),
                    bottom: Val::Percent(0.0),
                    ..Default::default()
                },
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            color: UiColor(Color::hex("264653").unwrap()),

            ..Default::default()
        })
        .insert(CompleteEntity)
        .id();
    commands.entity(root).add_child(bg);
    let t1_box = commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                position: UiRect {
                    left: Val::Percent(0.0),
                    bottom: Val::Percent(20.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                },
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .id();
    commands.entity(bg).add_child(t1_box);
    let t1 = commands
        .spawn()
        .insert_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: format!("Level Complete!\n"),
                        style: TextStyle {
                            font: assets.load("RobotoSlab-Regular.ttf"),
                            font_size: 150.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!("Scores:\n"),
                        style: TextStyle {
                            font: assets.load("RobotoSlab-Regular.ttf"),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!(
                            "{} Cycles, {} Blocks, {} Parts.\n",
                            this_cycles, this_blocks, this_parts
                        ),
                        style: TextStyle {
                            font: assets.load("RobotoSlab-Regular.ttf"),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!("Your High Scores:\n"),
                        style: TextStyle {
                            font: assets.load("RobotoSlab-Regular.ttf"),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!(
                            "{} Cycles, {} Blocks, {} Parts.\n",
                            high_cycles, high_blocks, high_parts
                        ),
                        style: TextStyle {
                            font: assets.load("RobotoSlab-Regular.ttf"),
                            font_size: 90.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!("Press any key to continue."),
                        style: TextStyle {
                            font: assets.load("RobotoSlab-Regular.ttf"),
                            font_size: 60.0,
                            color: Color::rgba(1.0, 1.0, 1.0, 0.6),
                        },
                    },
                ],
                alignment: TextAlignment::CENTER,
            },
            ..Default::default()
        })
        .insert(CompleteEntity)
        .id();
    commands.entity(t1_box).add_child(t1);
    for i in 0..1000 {
        let mut colors = [
            Color::hex("2A9D8F").unwrap(),
            Color::hex("E9C46A").unwrap(),
            Color::hex("F4A261").unwrap(),
            Color::hex("E76F51").unwrap(),
        ];
        colors.shuffle(&mut thread_rng());
        let color = colors[0];
        let e = commands
            .spawn()
            .insert(Confetti {
                pos: Vec2::new(if i % 2 == 0 { 10.0 } else { 90.0 }, -80.0),
                vel: Vec2::new(
                    thread_rng().gen_range(-20.0..20.0),
                    thread_rng().gen_range(120.0..200.0),
                ),
                mass: thread_rng().gen_range(1.5..3.0),
            })
            .insert(CompleteEntity)
            .insert_bundle(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Px(10.0),
                        height: Val::Px(10.0),
                    },
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                color: UiColor(color),
                ..Default::default()
            })
            .id();
        commands.entity(bg).add_child(e);
    }
}

#[derive(Component)]
struct Confetti {
    pub pos: Vec2,
    pub vel: Vec2,
    pub mass: f32,
}

fn simulate_confetti(
    mut commands: Commands,
    mut confetti: Query<(&mut Style, &mut Confetti)>,
    time: Res<Time>,
    mut key_events: EventReader<KeyboardInput>,
    mut mouse_events: EventReader<MouseButtonInput>,
    sfx: Res<Sfx>,
    audio: Res<Audio>,
) {
    for (mut style, mut confetti) in confetti.iter_mut() {
        let dx = confetti.vel * time.delta_seconds() * 2.0;
        confetti.pos += dx;
        let dv = 1.0 - time.delta_seconds().clamp(0.0, 0.2) * confetti.mass;
        confetti.vel.y -= 10.0 * time.delta_seconds();
        confetti.vel *= dv;
        style.position.left = Val::Percent(confetti.pos.x);
        style.position.bottom = Val::Percent(confetti.pos.y);
    }
    if key_events.iter().any(|e| e.state == ButtonState::Pressed)
        || mouse_events.iter().any(|e| e.state == ButtonState::Pressed)
    {
        audio.play_with_settings(sfx.click.clone(), PlaybackSettings::ONCE.with_volume(0.3));
        commands.insert_resource(ChangeToMenuRequest);
    }
}

fn cleanup(mut commands: Commands, entities: Query<Entity, With<CompleteEntity>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct ChangeToMenuRequest;

fn set_state(
    mut commands: Commands,
    request: Option<Res<ChangeToMenuRequest>>,
    mut game_state: ResMut<State<GameState>>,
) {
    if request.is_some() {
        game_state.set(GameState::Menu).unwrap();
        commands.remove_resource::<ChangeToMenuRequest>();
    }
}

pub struct CompletePlugin;

impl Plugin for CompletePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(CoreStage::First, set_state);
        app.add_system_set_to_stage(
            "asdf",
            SystemSet::on_enter(GameState::Complete).with_system(setup),
        )
        .add_system_set_to_stage(
            "asdf",
            SystemSet::on_update(GameState::Complete).with_system(simulate_confetti),
        )
        .add_system_set_to_stage(
            "asdf",
            SystemSet::on_exit(GameState::Complete).with_system(cleanup),
        );
    }
}
