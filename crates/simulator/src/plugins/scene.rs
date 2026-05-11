//! Scene plugin — cyberpunk/CAD visual style.
//!
//! Coordinate conventions:
//!   Printer mm  →  Bevy units via VIS_SCALE (1 mm = 0.01 m)
//!   Printer X   →  Bevy X
//!   Printer Y   →  Bevy Z  (depth)
//!   Printer Z   →  Bevy Y  (height, positive = up)
//!   Angles      →  degrees in engine; converted to radians in kinematics_sync.

use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::*,
};

use crate::{
    components::{ActuatorId, AxisMapping, BaseTransform, CoreXyGantryLink, CoreXyHeadLink, KinematicLink},
    resources::MachineConfig,
};
use kinematics::config::{KinematicsConfig, KinematicsType, PrintHeadGeometry};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.03, 0.04, 0.06)))
            .add_systems(Startup, (init_theme, setup_scene).chain())
            .add_systems(Update, draw_grid_system);
    }
}

/// Visual scale factor: 1 mm of logical space → 0.01 Bevy units.
pub const VIS_SCALE: f32 = 0.01;

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// Central material palette. Stored as a `Resource` so every spawn function
/// clones handles — no duplicate GPU materials are ever allocated.
#[derive(Resource)]
pub struct ThemeMaterials {
    /// Dark matte metal — fixed structural frame.
    pub frame: Handle<StandardMaterial>,
    /// Semi-transparent amber with slight glow — moving kinematic parts.
    pub moving_part: Handle<StandardMaterial>,
    /// Bright cyan neon — nozzle tip; emissive for bloom.
    pub nozzle: Handle<StandardMaterial>,
    /// Glossy chrome — linear guide rails.
    pub guide_rail: Handle<StandardMaterial>,
    /// Dark platform surface (bed / platter).
    pub platform: Handle<StandardMaterial>,
}

fn init_theme(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let theme = ThemeMaterials {
        frame: materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.11, 0.13),
            perceptual_roughness: 0.88,
            metallic: 0.35,
            ..default()
        }),
        moving_part: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.75, 0.0, 0.45),
            emissive: LinearRgba::rgb(0.35, 0.22, 0.0),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.4,
            metallic: 0.55,
            double_sided: true,
            ..default()
        }),
        nozzle: materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.9, 1.0),
            emissive: LinearRgba::rgb(0.0, 3.0, 4.0),
            perceptual_roughness: 0.08,
            metallic: 0.95,
            ..default()
        }),
        guide_rail: materials.add(StandardMaterial {
            base_color: Color::srgb(0.70, 0.75, 0.82),
            perceptual_roughness: 0.12,
            metallic: 0.95,
            reflectance: 0.85,
            ..default()
        }),
        platform: materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.09, 0.12),
            perceptual_roughness: 0.75,
            metallic: 0.2,
            ..default()
        }),
    };
    commands.insert_resource(theme);
}

// ---------------------------------------------------------------------------
// Environment setup (camera + lights)
// ---------------------------------------------------------------------------

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    theme: Res<ThemeMaterials>,
    config: Res<MachineConfig>,
) {
    // Camera with bloom for emissive glow
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2.5, 4.5, 6.5)
                .looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
            ..default()
        },
        BloomSettings {
            intensity: 0.35,
            ..default()
        },
    ));

    // Key light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: true,
            color: Color::srgb(0.9, 0.95, 1.0),
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
        ..default()
    });

    // Subtle fill light from below
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 40_000.0,
            color: Color::srgb(0.1, 0.4, 0.8),
            range: 20.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

    match config.0.kinematics_type {
        KinematicsType::Cartesian => {
            spawn_cartesian_printer(&mut commands, &mut meshes, &theme, &config.0)
        }
        KinematicsType::CoreXY => {
            spawn_corexy_printer(&mut commands, &mut meshes, &theme, &config.0)
        }
        KinematicsType::TrunnionCoreXY => {
            spawn_trunnion_corexy_printer(&mut commands, &mut meshes, &theme, &config.0)
        }
    }
}

// ---------------------------------------------------------------------------
// CAD grid gizmo (drawn every frame)
// ---------------------------------------------------------------------------

fn draw_grid_system(mut gizmos: Gizmos, config: Res<MachineConfig>) {
    let width = config.0.limits.x.max * VIS_SCALE;
    let depth = config.0.limits.y.max * VIS_SCALE;
    // Grid step = 50 mm
    let step = 0.50_f32;

    let nx = ((width / step).ceil() as i32) + 1;
    let nz = ((depth / step).ceil() as i32) + 1;
    let hw = nx as f32 * step / 2.0;
    let hd = nz as f32 * step / 2.0;

    let minor = Color::srgba(0.0, 0.85, 0.7, 0.10);
    let major = Color::srgba(0.0, 0.85, 0.7, 0.22);

    for i in -nx..=nx {
        let x = i as f32 * step;
        let color = if i % 5 == 0 { major } else { minor };
        gizmos.line(Vec3::new(x, 0.0, -hd), Vec3::new(x, 0.0, hd), color);
    }
    for j in -nz..=nz {
        let z = j as f32 * step;
        let color = if j % 5 == 0 { major } else { minor };
        gizmos.line(Vec3::new(-hw, 0.0, z), Vec3::new(hw, 0.0, z), color);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawn two vertical cylinder rails symmetrically around X at a given Z position.
fn spawn_z_rails(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    theme: &ThemeMaterials,
    height: f32,
    half_span: f32,
    z_pos: f32,
) {
    let rail_mesh = meshes.add(Cylinder::new(0.025, height));
    for &sx in &[-half_span, half_span] {
        commands.spawn(PbrBundle {
            mesh: rail_mesh.clone(),
            material: theme.guide_rail.clone(),
            transform: Transform::from_xyz(sx, height / 2.0, z_pos),
            ..default()
        });
    }
}

/// Spawn a horizontal gantry beam along X between x_min and x_max.
fn spawn_x_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    theme: &ThemeMaterials,
    length: f32,
    y_pos: f32,
    z_pos: f32,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cylinder::new(0.018, length)),
        material: theme.guide_rail.clone(),
        transform: Transform::from_xyz(0.0, y_pos, z_pos)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
}

/// Spawn a composite printhead (body + nozzle) as a parent SpatialBundle with children.
/// The parent gets the `CoreXyHeadLink` marker; translation is managed by `sync_corexy_head`.
fn spawn_composite_head(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    theme: &ThemeMaterials,
    head_geo: &PrintHeadGeometry,
    local_transform: Transform,
) -> Entity {
    let hw = head_geo.width * VIS_SCALE;
    let hh = head_geo.height * VIS_SCALE;
    let hd = head_geo.depth * VIS_SCALE;
    let nl = head_geo.nozzle_length * VIS_SCALE;

    commands
        .spawn((SpatialBundle { transform: local_transform, ..default() }, CoreXyHeadLink))
        .with_children(|p| {
            // Main hotend block
            p.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(hw, hh, hd)),
                material: theme.moving_part.clone(),
                transform: Transform::from_xyz(0.0, -hh / 2.0, 0.0),
                ..default()
            });
            // Nozzle cylinder — hangs below the block
            p.spawn(PbrBundle {
                mesh: meshes.add(Cylinder::new(0.012, nl)),
                material: theme.nozzle.clone(),
                transform: Transform::from_xyz(0.0, -hh - nl / 2.0, 0.0),
                ..default()
            });
        })
        .id()
}

// ---------------------------------------------------------------------------
// Cartesian printer
// ---------------------------------------------------------------------------

fn spawn_cartesian_printer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    theme: &ThemeMaterials,
    config: &KinematicsConfig,
) {
    let limits = &config.limits;
    let head_geo = config.head_geometry.as_ref().cloned().unwrap_or_default();

    let bx = limits.x.max * VIS_SCALE;
    let by = limits.y.max * VIS_SCALE;
    let zh = limits.z.max * VIS_SCALE;
    let nozzle_y = zh;

    // --- Z rails (vertical, front & back) ---
    spawn_z_rails(commands, meshes, theme, zh + 0.2, bx / 2.0, -by / 2.0 - 0.15);
    spawn_z_rails(commands, meshes, theme, zh + 0.2, bx / 2.0,  by / 2.0 + 0.15);

    // --- Z gantry (horizontal beam that moves up) ---
    let gantry_base = Vec3::new(0.0, 0.1, -by / 2.0 - 0.15);
    let z_gantry = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(bx + 0.1, 0.07, 0.07)),
                material: theme.frame.clone(),
                transform: Transform::from_translation(gantry_base),
                ..default()
            },
            KinematicLink {
                actuator: ActuatorId::Actuator3,
                mapping: AxisMapping::Translation(Vec3::Y * VIS_SCALE),
            },
            BaseTransform(gantry_base),
        ))
        .id();

    // --- X carriage (child of Z gantry) ---
    let carriage_local = Vec3::new(0.0, 0.05, 0.12);
    let carriage = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(carriage_local),
                ..default()
            },
            KinematicLink {
                actuator: ActuatorId::Actuator1,
                mapping: AxisMapping::Translation(Vec3::X * VIS_SCALE),
            },
            BaseTransform(carriage_local),
            CoreXyHeadLink, // reuse sync for X positioning
        ))
        .with_children(|p| {
            let hw = head_geo.width * VIS_SCALE;
            let hh = head_geo.height * VIS_SCALE;
            let hd = head_geo.depth * VIS_SCALE;
            let nl = head_geo.nozzle_length * VIS_SCALE;
            p.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(hw, hh, hd)),
                material: theme.moving_part.clone(),
                transform: Transform::from_xyz(0.0, -hh / 2.0, 0.0),
                ..default()
            });
            p.spawn(PbrBundle {
                mesh: meshes.add(Cylinder::new(0.012, nl)),
                material: theme.nozzle.clone(),
                transform: Transform::from_xyz(0.0, -hh - nl / 2.0, 0.0),
                ..default()
            });
        })
        .id();

    commands.entity(z_gantry).push_children(&[carriage]);

    // --- X rail on gantry ---
    spawn_x_beam(commands, meshes, theme, bx + 0.1, 0.05, -by / 2.0 - 0.15);

    // --- Bed (moves along Y) ---
    let bed_base = Vec3::new(0.0, 0.0, 0.0);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bx, 0.04, by)),
            material: theme.platform.clone(),
            transform: Transform::from_translation(bed_base),
            ..default()
        },
        KinematicLink {
            actuator: ActuatorId::Actuator2,
            mapping: AxisMapping::Translation(Vec3::Z * VIS_SCALE),
        },
        BaseTransform(bed_base),
    ));

    let _ = nozzle_y; // used implicitly via gantry Z offset
}

// ---------------------------------------------------------------------------
// CoreXY printer
// ---------------------------------------------------------------------------

fn spawn_corexy_printer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    theme: &ThemeMaterials,
    config: &KinematicsConfig,
) {
    let limits = &config.limits;
    let head_geo = config.head_geometry.as_ref().cloned().unwrap_or_default();

    let bx = limits.x.max * VIS_SCALE;
    let by = limits.y.max * VIS_SCALE;
    let zh = limits.z.max * VIS_SCALE;
    let nozzle_y = zh; // world Y of the nozzle tip at Z=0

    // --- 4 corner vertical rails for bed Z motion ---
    // --- 4 corner vertical rails for bed Z motion ---
    let half_x = bx / 2.0 + 0.1;
    let half_z = by / 2.0 + 0.1;
    for &(sx, sz) in &[(-half_x, -half_z), (half_x, -half_z), (-half_x, half_z), (half_x, half_z)] {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.022, zh + 0.3)),
            material: theme.guide_rail.clone(),
            transform: Transform::from_xyz(sx, zh / 2.0, sz),
            ..default()
        });
    }

    // --- Fixed Y-rails (along depth / Bevy Z) ---
    // These rails are stationary, and the X-gantry slides along them.
    for &sx in &[-bx / 2.0 - 0.1, bx / 2.0 + 0.1] {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.015, by + 0.3)),
            material: theme.guide_rail.clone(),
            transform: Transform::from_xyz(sx, nozzle_y + 0.08, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        });
    }

    // --- Moving X-Gantry (moves along Y / Bevy Z) ---
    // IMPORTANT: The logical gantry entity has NO rotation so that the child head
    // inherits world-aligned axes (local X = world X, local Y = world Y).
    // The visual cylinder is a child with its own rotation.
    let gantry = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, nozzle_y + 0.08, 0.0),
                ..default()
            },
            CoreXyGantryLink,
        ))
        .with_children(|p| {
            // Visual: horizontal cylinder along world X
            p.spawn(PbrBundle {
                mesh: meshes.add(Cylinder::new(0.018, bx + 0.25)),
                material: theme.guide_rail.clone(),
                transform: Transform::from_rotation(
                    Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                ),
                ..default()
            });
        })
        .id();

    // --- Composite CoreXY head (child of Gantry) ---
    // The gantry has no rotation, so local Y = world Y (downward = -Y).
    let head_local = Transform::from_xyz(0.0, -0.10, 0.0);
    let head = spawn_composite_head(commands, meshes, theme, &head_geo, head_local);
    commands.entity(gantry).push_children(&[head]);

    // --- Bed (moves down as Z increases) ---
    // At Z=0 the top surface is at nozzle_y.
    let bed_base = Vec3::new(0.0, nozzle_y, 0.0);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(bx, 0.04, by)),
            material: theme.platform.clone(),
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

// ---------------------------------------------------------------------------
// 5-axis TrunnionCoreXY printer
// ---------------------------------------------------------------------------

fn spawn_trunnion_corexy_printer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    theme: &ThemeMaterials,
    config: &KinematicsConfig,
) {
    let limits = &config.limits;
    let geometry = config
        .trunnion_geometry
        .as_ref()
        .expect("TrunnionCoreXY requires [trunnion_geometry] in printer.toml");
    let head_geo = config.head_geometry.as_ref().cloned().unwrap_or_default();

    let bx = limits.x.max * VIS_SCALE;
    let by = limits.y.max * VIS_SCALE;
    let zh = limits.z.max * VIS_SCALE;
    let nozzle_y = zh;

    // Total height of the trunnion stack above the Z-stage:
    //   platter_surface_world_Y = z_stage.y + pivot_a + platter_c = nozzle_y
    let stack_h = (geometry.pivot_a_offset_z + geometry.platter_c_offset_z) * VIS_SCALE;

    // --- 4 corner vertical Z rails ---
    let half_x = bx / 2.0 + 0.12;
    let half_z = by / 2.0 + 0.12;
    for &(sx, sz) in &[(-half_x, -half_z), (half_x, -half_z), (-half_x, half_z), (half_x, half_z)] {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.022, zh + 0.4)),
            material: theme.guide_rail.clone(),
            transform: Transform::from_xyz(sx, zh / 2.0, sz),
            ..default()
        });
    }

    // --- Fixed Y-rails (along depth / Bevy Z) ---
    for &sx in &[-bx / 2.0 - 0.12, bx / 2.0 + 0.12] {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.015, by + 0.35)),
            material: theme.guide_rail.clone(),
            transform: Transform::from_xyz(sx, nozzle_y + 0.08, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        });
    }

    // --- Moving X-Gantry (moves along Y / Bevy Z) ---
    let gantry = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, nozzle_y + 0.08, 0.0),
                ..default()
            },
            CoreXyGantryLink,
        ))
        .with_children(|p| {
            p.spawn(PbrBundle {
                mesh: meshes.add(Cylinder::new(0.018, bx + 0.28)),
                material: theme.guide_rail.clone(),
                transform: Transform::from_rotation(
                    Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                ),
                ..default()
            });
        })
        .id();

    // --- Composite CoreXY head (child of Gantry) ---
    let head_local = Transform::from_xyz(0.0, -0.10, 0.0);
    let head = spawn_composite_head(commands, meshes, theme, &head_geo, head_local);
    commands.entity(gantry).push_children(&[head]);

    // --- Z Stage (drops as Z increases) ---
    // Starts so that platter surface = nozzle_y
    let z_stage_base = Vec3::new(0.0, nozzle_y - stack_h, 0.0);
    let z_stage = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(bx + 0.1, 0.05, by + 0.1)),
                material: theme.frame.clone(),
                transform: Transform::from_translation(z_stage_base),
                ..default()
            },
            KinematicLink {
                actuator: ActuatorId::Actuator3,
                mapping: AxisMapping::Translation(Vec3::NEG_Y * VIS_SCALE),
            },
            BaseTransform(z_stage_base),
        ))
        .id();

    // --- Cradle (A-axis) — U-frame, rotates around X ---
    let cradle_pivot_y = geometry.pivot_a_offset_z * VIS_SCALE;
    let cradle_base = Vec3::new(0.0, cradle_pivot_y, 0.0);
    let cradle = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(bx + 0.08, 0.07, by + 0.08)),
                material: theme.moving_part.clone(),
                transform: Transform::from_translation(cradle_base),
                ..default()
            },
            KinematicLink {
                actuator: ActuatorId::Actuator4,
                mapping: AxisMapping::Rotation(Vec3::X),
            },
            BaseTransform(cradle_base),
        ))
        .id();

    // Cradle side arms (visual flair — two cylinders along Z)
    for &sx in &[-bx / 2.0, bx / 2.0] {
        commands.entity(cradle).with_children(|p| {
            p.spawn(PbrBundle {
                mesh: meshes.add(Cylinder::new(0.03, by + 0.04)),
                material: theme.guide_rail.clone(),
                transform: Transform::from_xyz(sx, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ..default()
            });
        });
    }

    // --- Platter (C-axis) — rotating disk, child of Cradle ---
    let platter_c_y = geometry.platter_c_offset_z * VIS_SCALE;
    let platter_base = Vec3::new(0.0, platter_c_y, 0.0);
    let platter = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cylinder::new(bx / 2.0 - 0.08, 0.04)),
                material: theme.platform.clone(),
                transform: Transform::from_translation(platter_base),
                ..default()
            },
            KinematicLink {
                actuator: ActuatorId::Actuator5,
                mapping: AxisMapping::Rotation(Vec3::Y),
            },
            BaseTransform(platter_base),
        ))
        .id();

    // Hierarchy: Z Stage → Cradle (A) → Platter (C)
    commands.entity(z_stage).push_children(&[cradle]);
    commands.entity(cradle).push_children(&[platter]);
}
