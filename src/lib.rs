use bevy::prelude::*;

#[derive(Clone, Component, Debug, Reflect)]
#[reflect(Component, Debug)]
/// When this component is added on an entity, [`Transform::forward()`] direction points towards the selected
/// entity always
#[relationship(relationship_target = RotatedToBy)]
pub struct RotateTo {
    /// entity to target, the Targeted entity must have a [`GlobalTransform`]
    #[relationship]
    pub entity: Entity,
    /// The rotated entity will match its [`Transform::up()`] according to this
    pub updir: UpDirection,
}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = RotateTo)]
pub struct RotatedToBy(Vec<Entity>);

#[derive(Clone, Copy, Debug, PartialEq, Reflect, Default)]
#[reflect(Debug, PartialEq)]
/// The rotated entity will try to have its [`Transform::up()`] direction matching this selection
pub enum UpDirection {
    /// Will synchronize the direction of UP towards the UP direction of the target
    /// Useful when rotating towards the camera and wanting the direction to be up for example
    #[default]
    Target,
    /// Keeps the up-direction the same as for the parent of this entity
    /// useful when you want it rotated in relation to what this entity is attached to
    /// Note: if there is no parent, the up direction will fallback to be Vec3::Y
    Parent,
    /// Keeps a static direction of UP set to this value
    /// useful when you want to decide what is up for the entity under rotation
    Dir(Dir3),
}

/// Plugin that constantly rotates entities towards a selected target when they have the [`RotateTo`]
/// component on them.
///
/// If you only want the math for calculating the local rotation needed to look at a target,
/// see the function [`calculate_local_rotation_to_target`]
pub struct RotateTowardsPlugin {
    /// determines if the plugins shall
    /// calculate new global transforms before trying to change rotation to match the target
    /// This can have a negative effect on performance, but helps combat the rotation lagging behind
    calculate_new_globals: bool,
}

impl Default for RotateTowardsPlugin {
    fn default() -> Self {
        Self::new(true)
    }
}

impl RotateTowardsPlugin {
    pub fn new(calculate_new_globals: bool) -> Self {
        Self {
            calculate_new_globals,
        }
    }
}

impl Plugin for RotateTowardsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RotateTo>();
        app.register_type::<RotatedToBy>();
        if self.calculate_new_globals {
            app.add_systems(
                PostUpdate,
                rotate_towards_with_updated_global_transforms.before(TransformSystems::Propagate),
            );
        } else {
            app.add_systems(
                PostUpdate,
                rotate_towards_without_updating_global_transforms
                    .before(TransformSystems::Propagate),
            );
        }
    }
}

fn rotate_towards_without_updating_global_transforms(
    global_transforms: Query<&GlobalTransform>, // potential_targets
    mut rotators: Query<(
        &mut Transform,
        &GlobalTransform,
        Option<&ChildOf>,
        &RotateTo,
    )>, // the ones to rotate
) {
    for (mut rotator_t, rotator_gt, child_of, target) in rotators.iter_mut() {
        let Ok(target_gt) = global_transforms.get(target.entity) else {
            continue;
        };

        let parent_gt = if let Some(child_of) = child_of {
            global_transforms.get(child_of.parent()).ok()
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

        let rotation = calculate_local_rotation_to_target(rotator_gt, target_gt, parent_gt, updir);

        rotator_t.rotation = rotation;
    }
}

fn rotate_towards_with_updated_global_transforms(
    mut commands: Commands,
    mut rotators: Query<(
        Entity,
        &Transform, // cant have mut access here, will conflict with TransformHelper
        Option<&ChildOf>,
        &RotateTo,
    )>, // the ones to rotate
    trans_helper: TransformHelper,
) {
    for (rotator_e, rotator_t, child_of, target) in rotators.iter_mut() {
        let Ok(target_gt) = trans_helper.compute_global_transform(target.entity) else {
            continue;
        };

        let parent_gt = if let Some(child_of) = child_of {
            trans_helper
                .compute_global_transform(child_of.parent())
                .ok()
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

        let Ok(rotator_gt) = trans_helper.compute_global_transform(rotator_e) else {
            continue;
        };

        let rotation =
            calculate_local_rotation_to_target(&rotator_gt, &target_gt, parent_gt.as_ref(), updir);

        // workaround since if we have a mutable access to Transforms in the rotators query,
        // we will create a Query Conflict panic
        let mut new_rotator_t = *rotator_t;
        new_rotator_t.rotation = rotation;

        let Ok(mut ec) = commands.get_entity(rotator_e) else {
            continue;
        };

        ec.try_insert(new_rotator_t);
    }
}

/// Calculates the local rotation on a rotator towards a target,
/// adjusting for rotations of eventual parents,
/// with the selected rotator up direction.
pub fn calculate_local_rotation_to_target(
    rotator_gt: &GlobalTransform,
    target_gt: &GlobalTransform,
    parent_gt: Option<&GlobalTransform>,
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
