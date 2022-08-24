use bevy::{
    pbr::NotShadowReceiver,
    prelude::*,
    render::view::RenderLayers,
    scene::{Scene, SceneInstance},
};

#[derive(Component)]
pub struct PleaseHologramifyThis;

fn insert_render_layers(commands: &mut Commands, onto: Entity, children_query: &Query<&Children>) {
    commands
        .entity(onto)
        .insert(RenderLayers::from_layers(&[1]))
        .insert(NotShadowReceiver);
    if let Ok(children) = children_query.get(onto) {
        for &child in children.iter() {
            insert_render_layers(commands, child, children_query);
        }
    }
}

fn hologramify_system(
    mut commands: Commands,
    to_hologramify: Query<(Entity, &PleaseHologramifyThis), With<SceneInstance>>,
    children_query: Query<&Children>,
) {
    for (entity, _) in to_hologramify.iter() {
        insert_render_layers(&mut commands, entity, &children_query);
        commands.entity(entity).remove::<PleaseHologramifyThis>();
    }
}

pub struct HologramifyPlugin;

impl Plugin for HologramifyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(hologramify_system);
    }
}
