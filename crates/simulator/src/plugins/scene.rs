//! Plugin for initializing and managing the 3D scene (camera, lights).

use bevy::prelude::*;
use crate::{
    components::{ActuatorId, AxisMapping, BaseTransform, CoreXyHeadLink, KinematicLink},
    resources::MachineConfig,
};
use kinematics::config::{KinematicsConfig, KinematicsType, MachineLimits};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
    }
}

/// Visual scale factor: 1 mm of logical space → 0.01 Bevy units (meters).
pub const VIS_SCALE: f32 = 0.01;

/// Setup the scene based on the kinematics config.
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<MachineConfig>,
) {
    // Position the camera to see a ~300x300x300mm machine (3x3x3 Bevy units)
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 4.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    });

    match config.0.kinematics_type {
        KinematicsType::Cartesian => spawn_cartesian_printer(&mut commands, &mut meshes, &mut materials, &config.0.limits),
        KinematicsType::CoreXY   => spawn_corexy_printer(&mut commands, &mut meshes, &mut materials, &config.0.limits),
        KinematicsType::TrunnionCoreXY => spawn_trunnion_corexy_printer(&mut commands, &mut meshes, &mut materials, &config.0),
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

    // --- Bed: moves along logical Y (Bevy Z) ---
    let bed_base = Vec3::new(0.0, 0.0, 0.0);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x, 0.05, bed_y)),
            material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
            transform: Transform::from_translation(bed_base),
            ..default()
        },
        KinematicLink {
            // Cartesian: Actuator2 = Y motor
            actuator: ActuatorId::Actuator2,
            mapping: AxisMapping::Translation(Vec3::Z * VIS_SCALE),
        },
        BaseTransform(bed_base),
    ));

    // --- Z Gantry: moves up along logical Z (Bevy Y) ---
    let gantry_base = Vec3::new(0.0, 0.1, -bed_y / 2.0 - 0.1);
    let z_gantry = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x + 0.2, 0.1, 0.1)),
            material: materials.add(Color::srgb(0.5, 0.5, 0.5)),
            transform: Transform::from_translation(gantry_base),
            ..default()
        },
        KinematicLink {
            // Cartesian: Actuator3 = Z motor
            actuator: ActuatorId::Actuator3,
            mapping: AxisMapping::Translation(Vec3::Y * VIS_SCALE),
        },
        BaseTransform(gantry_base),
    )).id();

    // --- X Carriage: moves along logical X (Bevy X), child of gantry ---
    let carriage_base = Vec3::new(0.0, 0.0, 0.1); // relative to gantry
    let x_carriage = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.15, 0.15)),
            material: materials.add(Color::srgb(0.8, 0.1, 0.1)),
            transform: Transform::from_translation(carriage_base),
            ..default()
        },
        KinematicLink {
            // Cartesian: Actuator1 = X motor
            actuator: ActuatorId::Actuator1,
            mapping: AxisMapping::Translation(Vec3::X * VIS_SCALE),
        },
        BaseTransform(carriage_base),
    )).id();

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

    // Nozzle world-Y position: frame sits at (z_height + 0.1), head hangs at -0.1 → nozzle_y = z_height
    let nozzle_y = z_height;

    // --- Fixed overhead frame with CoreXY head ---
    let frame = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(bed_x + 0.2, 0.1, bed_y + 0.2)),
        material: materials.add(Color::srgb(0.4, 0.4, 0.4)),
        transform: Transform::from_xyz(0.0, nozzle_y + 0.1, 0.0),
        ..default()
    }).id();

    let head = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.15, 0.15, 0.15)),
            material: materials.add(Color::srgb(0.1, 0.8, 0.1)),
            // Local to frame: hangs 0.1 below, starts at X/Z corner
            transform: Transform::from_xyz(-bed_x / 2.0, -0.1, -bed_y / 2.0),
            ..default()
        },
        CoreXyHeadLink,
    )).id();
    commands.entity(frame).push_children(&[head]);

    // --- Bed ---
    // At Z=0: bed surface at nozzle_y (touching the nozzle).
    // As Z increases → bed moves NEG_Y, increasing the gap.
    let bed_base = Vec3::new(0.0, nozzle_y, 0.0);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x, 0.05, bed_y)),
            material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
            transform: Transform::from_translation(bed_base),
            ..default()
        },
        KinematicLink {
            actuator: ActuatorId::Actuator3,
            mapping: AxisMapping::Translation(Vec3::NEG_Y * VIS_SCALE),
        },
        BaseTransform(bed_base),
    ));
}

fn spawn_trunnion_corexy_printer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &KinematicsConfig,
) {
    let limits = &config.limits;
    let geometry = config.trunnion_geometry.as_ref()
        .expect("TrunnionCoreXY requires [trunnion_geometry] in printer.toml");

    let bed_x     = limits.x.max * VIS_SCALE;
    let bed_y     = limits.y.max * VIS_SCALE;
    let z_height  = limits.z.max * VIS_SCALE;

    // Nozzle world-Y: frame at (z_height + 0.1), head hangs at -0.1 from frame
    let nozzle_y  = z_height;

    // Total height of the trunnion stack above the Z-stage:
    //   Cradle pivot is pivot_a_offset_z above the Z-stage.
    //   Platter surface is platter_c_offset_z above the cradle pivot.
    // At Z=0, platter surface must be at nozzle_y:
    //   z_stage.y + pivot_a + platter_c = nozzle_y
    //   → z_stage.y = nozzle_y - (pivot_a + platter_c)
    let stack_height = (geometry.pivot_a_offset_z + geometry.platter_c_offset_z) * VIS_SCALE;
    let z_stage_base = Vec3::new(0.0, nozzle_y - stack_height, 0.0);

    // --- Fixed overhead frame with CoreXY head ---
    let frame = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(bed_x + 0.2, 0.1, bed_y + 0.2)),
        material: materials.add(Color::srgb(0.4, 0.4, 0.4)),
        transform: Transform::from_xyz(0.0, nozzle_y + 0.1, 0.0),
        ..default()
    }).id();

    let head = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.15, 0.15, 0.15)),
            material: materials.add(Color::srgb(0.1, 0.8, 0.1)),
            transform: Transform::from_xyz(-bed_x / 2.0, -0.1, -bed_y / 2.0),
            ..default()
        },
        CoreXyHeadLink,
    )).id();
    commands.entity(frame).push_children(&[head]);

    // --- Z Stage ---
    // As Z increases → stage moves NEG_Y, the whole trunnion stack drops.
    let z_stage = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x + 0.1, 0.05, bed_y + 0.1)),
            material: materials.add(Color::srgb(0.3, 0.3, 0.3)),
            transform: Transform::from_translation(z_stage_base),
            ..default()
        },
        KinematicLink {
            actuator: ActuatorId::Actuator3,
            mapping: AxisMapping::Translation(Vec3::NEG_Y * VIS_SCALE),
        },
        BaseTransform(z_stage_base),
    )).id();

    // --- Cradle (A-axis) — child of Z Stage ---
    // Local offset: pivot_a_offset_z above the Z-stage plate.
    let cradle_base = Vec3::new(0.0, geometry.pivot_a_offset_z * VIS_SCALE, 0.0);
    let cradle = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bed_x + 0.05, 0.08, bed_y + 0.05)),
            material: materials.add(Color::srgb(0.8, 0.5, 0.2)),
            transform: Transform::from_translation(cradle_base),
            ..default()
        },
        KinematicLink {
            actuator: ActuatorId::Actuator4,
            mapping: AxisMapping::Rotation(Vec3::X), // A-axis tilts around Bevy X
        },
        BaseTransform(cradle_base),
    )).id();

    // --- Platter (C-axis) — child of Cradle ---
    // Local offset: platter_c_offset_z above the cradle pivot.
    let platter_base = Vec3::new(0.0, geometry.platter_c_offset_z * VIS_SCALE, 0.0);
    let platter = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(bed_x / 2.0 - 0.05, 0.04)),
            material: materials.add(Color::srgb(0.2, 0.2, 0.8)),
            transform: Transform::from_translation(platter_base),
            ..default()
        },
        KinematicLink {
            actuator: ActuatorId::Actuator5,
            mapping: AxisMapping::Rotation(Vec3::Y), // C-axis spins around Bevy Y
        },
        BaseTransform(platter_base),
    )).id();

    // Hierarchy: Z Stage → Cradle (A) → Platter (C)
    commands.entity(z_stage).push_children(&[cradle]);
    commands.entity(cradle).push_children(&[platter]);
}
