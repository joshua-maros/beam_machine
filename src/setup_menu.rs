use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
struct MenuEntity;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn()
        .insert_bundle(Camera2dBundle::default())
        .insert(MenuEntity);
    let menu_bg = assets.load("menu.png");
    commands
        .spawn()
        .insert_bundle(ImageBundle {
            image: UiImage(menu_bg),
            style: Style {
                aspect_ratio: Some(16.0 / 9.0),
                margin: UiRect {
                    bottom: Val::Auto,
                    ..Default::default()
                },
                align_self: AlignSelf::Auto,
                flex_basis: Val::Px(100.0),
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MenuEntity);
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set_to_stage(
            "asdf",
            SystemSet::on_enter(GameState::Menu).with_system(setup),
        );
    }
}
