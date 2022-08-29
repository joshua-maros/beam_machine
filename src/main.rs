// Music by <a href="https://pixabay.com/users/sergepavkinmusic-6130722/?utm_source=link-attribution&amp;utm_medium=referral&amp;utm_campaign=music&amp;utm_content=116585">SergePavkinMusic</a> from <a href="https://pixabay.com//?utm_source=link-attribution&amp;utm_medium=referral&amp;utm_campaign=music&amp;utm_content=116585">Pixabay</a>

pub mod animations;
mod block;
mod hologramify;
mod interface;
mod setup;
mod setup_complete;
mod setup_menu;
mod simulation;
mod structure;
mod world;

use animations::AnimationPlugin;
use bevy::{audio::AudioSink, prelude::*};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastSystem};
use bevy_obj::ObjPlugin;
use block::{update_raycast_position_from_cursor, BlockRaycastSet};
use hologramify::HologramifyPlugin;
use interface::InterfacePlugin;
use setup::SetupPlugin;
use setup_complete::CompletePlugin;
use setup_menu::MenuPlugin;
use simulation::SimulationPlugin;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum GameState {
    Menu,
    Level,
    Complete,
}

pub struct Sfx {
    pub level_complete: Handle<AudioSource>,
    pub click: Handle<AudioSource>,
    pub ding: Handle<AudioSource>,
    pub place: [Handle<AudioSource>; 3],
}

fn setup_music(
    mut commands: Commands,
    assets: Res<AssetServer>,
    audio: Res<Audio>,
    mut windows: ResMut<Windows>,
) {
    let music = assets.load("music.ogg");
    audio.play_with_settings(music, PlaybackSettings::LOOP.with_volume(0.3));
    commands.insert_resource(Sfx {
        level_complete: assets.load("level_complete.ogg"),
        click: assets.load("click.ogg"),
        ding: assets.load("ding.ogg"),
        place: [
            assets.load("place1.ogg"),
            assets.load("place2.ogg"),
            assets.load("place3.ogg"),
        ],
    });
    windows.primary_mut().set_scale_factor_override(Some(1.0));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(DefaultRaycastingPlugin::<BlockRaycastSet>::default())
        .add_state(GameState::Menu)
        .add_plugin(SimulationPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(HologramifyPlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(InterfacePlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(CompletePlugin)
        .insert_resource(AmbientLight {
            color: Color::hex("264653").unwrap(),
            brightness: 5.0,
        })
        .insert_resource(ClearColor(Color::hex("264653").unwrap() * 0.6))
        .add_startup_system(setup_music)
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_position_from_cursor.before(RaycastSystem::BuildRays::<BlockRaycastSet>),
        )
        .run();
}
