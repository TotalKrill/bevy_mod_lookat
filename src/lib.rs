use bevy::prelude::*;

#[derive(Component)]
/// When this component is added on an entity, [`Transform::forward()`] direction points towards the selected
/// entity
pub struct RotateTo {
    /// entity to target, the Targeted entity must have a [`GlobalTransform`]
    pub entity: Entity,
    /// The rotated entity will match its [`Transform::up()`] according to this
    pub updir: UpDirection,
}

/// The rotated entity will try to have its [`Transform::up()`] direction matching this selection
pub enum UpDirection {
    /// Will synchronize the direction of UP towards the UP direction of the target
    /// Useful when rotating towards the camera and wanting the direction to be up for example
    Target,
    /// Keeps the up-direction the same as for the parent of this entity
    /// useful when you want it rotated in relation to what this entity is attached to
    /// Note: if there is no parent, the up direction will fallback to be Vec3::Y
    Parent,
    /// Keeps a static direction of UP set to this value
    /// useful when you want to decide what is up for the entity under rotation
    Dir(Dir3),
}

/// Plugin that constantly rotates entities towards a selected target when they have the [`RotateTo`] component on them
/// if you only want the math for calculating the local rotation needed to look at a target, see [`calculate_local_rotation_to_target`]
pub struct RotateTowardsPlugin;

impl Plugin for RotateTowardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rotations>();
        app.add_systems(
            PostUpdate,
            (rotate_towards, apply_rotations)
                .before(TransformSystem::TransformPropagate)
                .chain(),
        );
    }
}
#[derive(Resource, Default)]
struct Rotations(pub Vec<(Entity, Quat)>);

fn rotate_towards(
    mut rotations: ResMut<Rotations>,
    transhelp: TransformHelper,
    calculation_query: Query<(Entity, Option<&Parent>, &RotateTo)>, // the ones to rotate
) {
    // store the rotations
    rotations.0.clear();
    for (rotator_entity, parent, target) in calculation_query.iter() {
        // let transhelp = transform_params.p0();
        let Ok(target_gt) = transhelp.compute_global_transform(target.entity) else {
            bevy::log::error!("Entity used as target was not found: {}", target.entity);
            continue;
        };
        let Ok(rotator_gt) = transhelp.compute_global_transform(rotator_entity) else {
            bevy::log::error!(
                "Failed to calculate global transform for {}",
                rotator_entity
            );
            continue;
        };

        let parent_gt = if let Some(parent_e) = parent {
            transhelp.compute_global_transform(parent_e.get()).ok()
        } else {
            None
        };

        let updir = match target.updir {
            UpDirection::Target => target_gt.up(),
            UpDirection::Dir(dir) => dir,
            UpDirection::Parent => {
                if let Some(parent_gt) = parent_gt {
                    parent_gt.up()
                } else {
                    // if there is no parent, fallback to bevy up direction
                    Dir3::Y
                }
            }
        };

        let rotation =
            calculate_local_rotation_to_target(&rotator_gt, &target_gt, parent_gt, updir);

        rotations.0.push((rotator_entity, rotation));
    }
}

fn apply_rotations(
    rotations: Res<Rotations>,
    // put another modification query, and use this to apply the transformations calculated
    mut modification: Query<&mut Transform, With<RotateTo>>,
) {
    for (entity, newrot) in rotations.0.iter() {
        if let Ok(mut rotator_t) = modification.get_mut(*entity) {
            rotator_t.rotation = *newrot;
        }
    }
}

/// Calculates the local rotation on a rotator towards a target, adjusting for rotations of eventual parents, with the selected rotator up direction.
pub fn calculate_local_rotation_to_target(
    rotator_gt: &GlobalTransform,
    target_gt: &GlobalTransform,
    parent_gt: Option<GlobalTransform>,
    updir: Dir3,
) -> Quat {
    let target_gt_computed = target_gt.compute_transform();
    let parent_gt_computed: Option<Transform> = parent_gt.map(|p| p.compute_transform());

    let mut rotation = rotator_gt
        .compute_transform()
        .looking_at(target_gt_computed.translation, updir)
        .rotation;

    if let Some(parent_gt_computed) = parent_gt_computed {
        rotation = parent_gt_computed.rotation.inverse() * rotation;
    }
    rotation
}
