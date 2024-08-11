//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{color::palettes::css::*, prelude::*, render::primitives::Aabb};
use bevy_mod_lookat::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RotateTowardsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (mover, rotate, draw_axes, draw_forward))
        .run();
}

#[derive(Component)]
struct Move;

#[derive(Component)]
struct Rotate;

#[derive(Component)]
struct ShowAxes;

#[derive(Component)]
struct ShowForward;

fn draw_axes(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &Aabb), With<ShowAxes>>) {
    for (&transform, &aabb) in &query {
        let t = transform.compute_transform();

        let length = aabb.half_extents.length();
        gizmos.axes(t, length);
    }
}
fn draw_forward(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<ShowForward>>) {
    for &transform in &query {
        let t = transform.compute_transform();

        gizmos.line(
            t.translation,
            t.translation + t.forward() * 6.0,
            Color::BLACK,
        );
    }
}
fn rotate(mut query: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
        transform.rotate_x(time.delta_seconds());
    }
}
fn mover(time: Res<Time>, mut ents: Query<&mut Transform, With<Move>>) {
    let distance = 2.0;
    for mut ent in ents.iter_mut() {
        ent.translation.x = distance * f32::sin(time.elapsed().as_secs_f32());
        ent.translation.z = distance * f32::cos(time.elapsed().as_secs_f32());
        ent.translation.y = 1.5 + 0.5 * distance * f32::cos(3.0 * time.elapsed().as_secs_f32());
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
            Rotate,
            ShowAxes,
        ))
        .id();

    // cube
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::srgb_u8(124, 144, 255)),
                transform: Transform::from_xyz(3.5, 0.5, 0.0),
                ..default()
            },
            ShowAxes,
        ))
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
                    ShowAxes,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(0.3, 0.1, 2.0)),
                            material: materials.add(Color::from(GREEN)),
                            transform: Transform::from_xyz(0.0, 0.7, 0.0),
                            ..default()
                        },
                        RotateTo {
                            entity: target_id,
                            // this choses what the flat side should be in relation towards
                            updir: UpDirection::Parent,
                        },
                        ShowForward,
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
