//! Plugin for initializing and managing the 3D scene (camera, lights).

use bevy::prelude::*;
use crate::{components::{ActuatorId, AxisMapping, CoreXyHeadLink, KinematicLink}, resources::MachineConfig};
use kinematics::config::{KinematicsType, MachineLimits};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
    }
}

// Factor for visual display (1 mm = 0.01 meters in Bevy)
const VIS_SCALE: f32 = 0.01;

/// Setup the scene based on the kinematics config.
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<MachineConfig>,
) {
    // Basic 3D Camera
    // Position the camera so we can see a 300x300x300 machine
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 4.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ..default()
    });

    // Simple Directional Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    });

    // Delegate to the correct builder
    match config.0.kinematics_type {
        KinematicsType::Cartesian => spawn_cartesian_printer(&mut commands, &mut meshes, &mut materials, &config.0.limits),
        KinematicsType::CoreXY => spawn_corexy_printer(&mut commands, &mut meshes, &mut materials, &config.0.limits),
    }
}

fn spawn_cartesian_printer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &MachineLimits,
) {
    let bed_x = limits.x.max * VIS_SCALE;
    let bed_y = limits.y.max * VIS_SCALE;
    // let z_height = limits.z.max * VIS_SCALE;

    // Bed (Moves in Y axis logically, which is Z in Bevy)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x, 0.05, bed_y)),
            material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
            // Starting position centered in X, offset in Z
            transform: Transform::from_xyz(0.0, 0.0, 0.0), 
            ..default()
        },
        KinematicLink {
            // Logical Y translates to Axis 2 in Cartesian
            actuator: ActuatorId::AxisY, 
            mapping: AxisMapping::Translation(Vec3::Z * VIS_SCALE),
        },
    ));

    // Z Gantry (Moves up in Z logically, which is Y in Bevy)
    let z_gantry = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x + 0.2, 0.1, 0.1)),
            material: materials.add(Color::srgb(0.5, 0.5, 0.5)),
            // Starts slightly above the bed, offset in Z so it doesn't clip the bed center
            transform: Transform::from_xyz(0.0, 0.1, -bed_y / 2.0 - 0.1), 
            ..default()
        },
        KinematicLink {
            // Logical Z translates to Axis 3
            actuator: ActuatorId::AxisZ,
            mapping: AxisMapping::Translation(Vec3::Y * VIS_SCALE),
        },
    )).id();

    // X Carriage (Moves in X logically, child of Z Gantry)
    let x_carriage = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.15, 0.15)),
            material: materials.add(Color::srgb(0.8, 0.1, 0.1)),
            transform: Transform::from_xyz(0.0, 0.0, 0.1), // Relative to gantry
            ..default()
        },
        KinematicLink {
            // Logical X translates to Axis 1
            actuator: ActuatorId::AxisX,
            mapping: AxisMapping::Translation(Vec3::X * VIS_SCALE),
        },
    )).id();

    // Setup hierarchy
    commands.entity(z_gantry).push_children(&[x_carriage]);
}

fn spawn_corexy_printer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &MachineLimits,
) {
    let bed_x = limits.x.max * VIS_SCALE;
    let bed_y = limits.y.max * VIS_SCALE;
    let z_height = limits.z.max * VIS_SCALE;

    // Bed (Moves in Z logically, which is Y in Bevy)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x, 0.05, bed_y)),
            material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
            // Start bed at top of Z volume (or bottom, depending on design, assuming top)
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        KinematicLink {
            // Logical Z translates to Axis 3
            actuator: ActuatorId::AxisZ,
            // Typical corexy beds move DOWN to increase Z distance from nozzle
            // We'll map increasing Z to negative Y in Bevy
            mapping: AxisMapping::Translation(Vec3::NEG_Y * VIS_SCALE),
        },
    ));

    // Fixed Frame (At the top of the machine volume)
    let frame = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(bed_x + 0.2, 0.1, bed_y + 0.2)),
        material: materials.add(Color::srgb(0.4, 0.4, 0.4)),
        transform: Transform::from_xyz(0.0, z_height + 0.1, 0.0),
        ..default()
    }).id();

    // CoreXY Head (Moves in X and Y logically, handled by custom component)
    let head = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.15, 0.15, 0.15)),
            material: materials.add(Color::srgb(0.1, 0.8, 0.1)),
            // Start at corner relative to frame
            transform: Transform::from_xyz(-bed_x / 2.0, -0.1, -bed_y / 2.0),
            ..default()
        },
        CoreXyHeadLink, // Special component
    )).id();

    // Setup hierarchy
    commands.entity(frame).push_children(&[head]);
}
