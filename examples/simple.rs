//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{color::palettes::css::*, prelude::*};
use bevy_mod_lookat::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RotateTowardsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_entities, rotate))
        .run();
}

fn move_entities(time: Res<Time>, mut ents: Query<&mut Transform, With<Move>>) {
    let distance = 2.0;
    for mut ent in ents.iter_mut() {
        ent.translation.x = distance * f32::sin(time.elapsed().as_secs_f32());
        ent.translation.z = distance * f32::cos(time.elapsed().as_secs_f32());
        ent.translation.y = 0.5 * distance * f32::cos(3.0 * time.elapsed().as_secs_f32());
    }
}

#[derive(Component)]
struct Move;

#[derive(Component)]
struct Rotate;

fn rotate(mut query: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
        transform.rotate_x(time.delta_seconds());
    }
}
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    let target_id = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
                material: materials.add(Color::from(DARK_RED)),
                transform: Transform::from_xyz(1.0, 0.5, 1.0),
                ..default()
            },
            Move,
        ))
        .id();

    // cube
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn((
                    Rotate,
                    PbrBundle {
                        mesh: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
                        material: materials.add(Color::srgb_u8(124, 100, 255)),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(0.1, 0.1, 2.0)),
                            material: materials.add(Color::from(GREEN)),
                            transform: Transform::from_xyz(0.0, 0.7, 0.0),
                            ..default()
                        },
                        // makes this component rotate towards the set target
                        RotateTo(target_id),
                    ));
                });
        });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
