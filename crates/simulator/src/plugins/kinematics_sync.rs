//! Synchronizes physical state from `engine` to Bevy entities via `KinematicLink`.

use bevy::prelude::*;
use crate::{
    components::{ActuatorId, AxisMapping, BaseTransform, CoreXyGantryLink, CoreXyHeadLink, KinematicLink},
    resources::{MachineConfig, MachineState},
};
use crate::plugins::scene::VIS_SCALE;
use core_types::ActuatorState;

pub struct KinematicsSyncPlugin;

impl Plugin for KinematicsSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_transforms_system, sync_corexy_head));
    }
}

/// Reads actuator states from the Engine and applies them to every entity with
/// a `KinematicLink`, offset from its `BaseTransform` (resting position).
///
/// FIX: Previously `transform.translation = axis * value` which erased the
/// entity's initial spawn position. Now: `base + axis * value`.
fn update_transforms_system(
    machine_state: Res<MachineState>,
    mut query: Query<(&mut Transform, &KinematicLink, &BaseTransform), (Without<CoreXyHeadLink>, Without<CoreXyGantryLink>)>,
) {
    let actuators: ActuatorState = {
        let lock = machine_state.0.lock().unwrap();
        *lock.current_actuators()
    };

    for (mut transform, link, base) in query.iter_mut() {
        let value = match link.actuator {
            ActuatorId::Actuator1 => actuators.axis_1,
            ActuatorId::Actuator2 => actuators.axis_2,
            ActuatorId::Actuator3 => actuators.axis_3,
            ActuatorId::Actuator4 => actuators.axis_4,
            ActuatorId::Actuator5 => actuators.axis_5,
            ActuatorId::Extruder  => actuators.extruder,
        };

        match link.mapping {
            AxisMapping::Translation(axis) => {
                // FIX: Add offset to base, don't replace.
                transform.translation = base.0 + axis * value;
            }
            AxisMapping::Rotation(axis) => {
                // The angle from the engine is in degrees, Bevy expects radians.
                transform.rotation = Quat::from_axis_angle(axis.normalize(), value.to_radians());
            }
        }
    }
}

/// Dedicated sync system for the CoreXY toolhead.
///
/// FIX: Previously duplicated `inverse_kinematics` math by reading raw actuator
/// values (axis_1, axis_2) and recalculating X/Y. Now reads `current_target()`
/// directly from the engine, which already knows the logical X/Y position.
fn sync_corexy_head(
    machine_state: Res<MachineState>,
    config: Res<MachineConfig>,
    mut head_query: Query<&mut Transform, (With<CoreXyHeadLink>, Without<CoreXyGantryLink>)>,
    mut gantry_query: Query<&mut Transform, (With<CoreXyGantryLink>, Without<CoreXyHeadLink>)>,
) {
    let (target_x, target_y) = {
        let lock = machine_state.0.lock().unwrap();
        let t = lock.current_target();
        (t.x, t.y)
    };

    let bed_x_half = config.0.limits.x.max * VIS_SCALE / 2.0;
    let bed_y_half = config.0.limits.y.max * VIS_SCALE / 2.0;

    // 1. Move Gantry along Y (Bevy Z)
    for mut transform in gantry_query.iter_mut() {
        transform.translation.z = (target_y * VIS_SCALE) - bed_y_half;
    }

    // 2. Move Head along X
    // Note: If head is a child of gantry, translation.z is 0 (relative).
    // If head is independent, we might need to set Z too, but user asked for "standard rails"
    // which implies a moving transverse gantry.
    for mut transform in head_query.iter_mut() {
        transform.translation.x = (target_x * VIS_SCALE) - bed_x_half;
    }
}
