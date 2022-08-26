use bevy::{
    pbr::NotShadowReceiver,
    prelude::*,
    render::view::RenderLayers,
    scene::{Scene, SceneInstance},
};

#[derive(Component, Default)]
pub struct PleaseHologramifyThis(u8);

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
    mut to_hologramify: Query<(Entity, &mut PleaseHologramifyThis), With<SceneInstance>>,
    children_query: Query<&Children>,
) {
    for (entity, mut pht) in to_hologramify.iter_mut() {
        insert_render_layers(&mut commands, entity, &children_query);
        pht.0 += 1;
        // Why is this necessary? WHY?!
        if pht.0 == 2 {
            commands.entity(entity).remove::<PleaseHologramifyThis>();
        }
    }
}

pub struct HologramifyPlugin;

impl Plugin for HologramifyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, hologramify_system);
    }
}
