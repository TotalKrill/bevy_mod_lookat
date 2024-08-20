# bevy_mod_lookat

A microplugin and library for bevy to rotate an entity towards a target through a hierarchy

[![Crates.io](https://img.shields.io/crates/v/bevy_ui_anchor)](https://crates.io/crates/bevy_ui_anchor)
[![Documentation](https://docs.rs/bevy_ui_anchor/badge.svg)](https://docs.rs/bevy_ui_anchor)
[![License](https://img.shields.io/crates/l/bevy_ui_anchor)](https://opensource.org/licenses/MIT)

```rust
use bevy::prelude::*;
use bevy_ui_anchor::{RotateTowardsPlugin, RotateTo, UpDirection};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RotateTowardsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let target = commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).id();

    commands.spawn((
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        GlobalTransform::default(),
        RotateTo {
            entity: target,
            updir: UpDirection::Target,
        },
    ));

    commands.spawn(Camera3dBundle::default());
}

```
