use bevy::{prelude::*, transform::systems::propagate_transforms};

#[derive(Component)]
pub struct RotateTo(pub Entity);

pub struct RotateTowardsPlugin;

impl Plugin for RotateTowardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, rotate_towards.after(propagate_transforms));
    }
}

fn rotate_towards(
    global_transforms: Query<&GlobalTransform>, // potential_targets
    mut query: Query<(&mut Transform, &GlobalTransform, Option<&Parent>, &RotateTo)>, // the ones to rotate
) {
    for (mut rotated_t, rotated_gt, parent, target) in query.iter_mut() {
        let Ok(target_gt) = global_transforms.get(target.0) else {
            bevy::log::error!("Entity used as target was not found: {}", target.0);
            continue;
        };
        let target_gt = target_gt.compute_transform();
        let mut rotation = rotated_gt
            .compute_transform()
            .looking_at(target_gt.translation, target_gt.up())
            .rotation;

        if let Some(parent_e) = parent {
            if let Ok(parent_gt) = global_transforms.get(parent_e.get()) {
                rotation = parent_gt.compute_transform().rotation.inverse() * rotation;
            }
        };
        rotated_t.rotation = rotation;
    }
}
