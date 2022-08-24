use bevy::prelude::*;

use crate::simulation::SimulationState;

#[derive(Component)]
pub enum Animation {
    Stationary,
    Lerp(Vec3, Vec3),
}

fn animation_system(
    mut animated_objs: Query<(&mut Transform, &Animation)>,
    simulation_state: Res<SimulationState>,
) {
    for (mut transform, animation) in animated_objs.iter_mut() {
        match animation {
            Animation::Stationary => transform.translation = Vec3::ZERO,
            Animation::Lerp(a, b) => {
                transform.translation = a.lerp(*b, simulation_state.tick_progress())
            }
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animation_system);
    }
}
