//! Synchronizes physical state from `engine` to Bevy entities via `KinematicLink`.

use bevy::prelude::*;
use crate::{
    components::{ActuatorId, AxisMapping, BaseTransform, CoreXyHeadLink, KinematicLink},
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
    mut query: Query<(&mut Transform, &KinematicLink, &BaseTransform), Without<CoreXyHeadLink>>,
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
                transform.rotation = Quat::from_axis_angle(axis.normalize(), value);
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
    mut query: Query<&mut Transform, With<CoreXyHeadLink>>,
) {
    // Read logical X/Y from the target — no manual inverse kinematics needed.
    let (target_x, target_y) = {
        let lock = machine_state.0.lock().unwrap();
        let t = lock.current_target();
        (t.x, t.y)
    };

    // Center offset: the head starts at corner (-bed_x/2, _, -bed_y/2) of the frame.
    // When target is 0mm, translation should be -half_size; at max, it's +half_size.
    let bed_x_half = config.0.limits.x.max * VIS_SCALE / 2.0;
    let bed_y_half = config.0.limits.y.max * VIS_SCALE / 2.0;

    for mut transform in query.iter_mut() {
        // Bevy X  ← logical X, Bevy Z ← logical Y (depth in Bevy = Y in print space)
        transform.translation.x = (target_x * VIS_SCALE) - bed_x_half;
        transform.translation.z = (target_y * VIS_SCALE) - bed_y_half;
    }
}
