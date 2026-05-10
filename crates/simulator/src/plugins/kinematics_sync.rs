//! Synchronizes physical state from `engine` to Bevy entities via `KinematicLink`.

use bevy::prelude::*;
use crate::{components::{ActuatorId, AxisMapping, CoreXyHeadLink, KinematicLink}, resources::{MachineConfig, MachineState}};
use core_types::ActuatorState;

pub struct KinematicsSyncPlugin;

impl Plugin for KinematicsSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_transforms_system, sync_corexy_head));
    }
}

/// A thin system that reads actuator states from the Engine and applies 
/// them to the Transform of every entity with a `KinematicLink`.
fn update_transforms_system(
    machine_state: Res<MachineState>,
    mut query: Query<(&mut Transform, &KinematicLink), Without<CoreXyHeadLink>>,
) {
    // Lock the mutex and extract a snapshot of the actuators
    let actuator_snapshot: ActuatorState = {
        let lock = machine_state.0.lock().unwrap();
        *lock.current_actuators() // Implements Copy
    };

    for (mut transform, link) in query.iter_mut() {
        // Resolve the value from the specific actuator
        let value = match link.actuator {
            ActuatorId::AxisX => actuator_snapshot.axis_1,
            ActuatorId::AxisY => actuator_snapshot.axis_2,
            ActuatorId::AxisZ => actuator_snapshot.axis_3,
            ActuatorId::AxisA => actuator_snapshot.axis_4,
            ActuatorId::AxisC => actuator_snapshot.axis_5,
            ActuatorId::Extruder => actuator_snapshot.extruder,
        };

        // Apply mapping to transform
        match link.mapping {
            AxisMapping::Translation(axis) => {
                transform.translation = axis * value;
            }
            AxisMapping::Rotation(axis) => {
                // Assuming `value` is in radians for rotary axes
                transform.rotation = Quat::from_axis_angle(axis.normalize(), value);
            }
        }
    }
}

/// Dedicated synchronization system for the CoreXY toolhead.
/// The CoreXY head is driven by two motors combined (A and B, stored in axis_1 and axis_2).
fn sync_corexy_head(
    machine_state: Res<MachineState>,
    config: Res<MachineConfig>,
    mut query: Query<&mut Transform, With<CoreXyHeadLink>>,
) {
    // Lock the mutex and extract a snapshot of the actuators
    let actuator_snapshot: ActuatorState = {
        let lock = machine_state.0.lock().unwrap();
        *lock.current_actuators()
    };

    // CoreXY math backwards (actuators to physical position)
    // Motor A = X + Y -> axis_1
    // Motor B = X - Y -> axis_2
    // X = 0.5 * (Motor A + Motor B)
    // Y = 0.5 * (Motor A - Motor B)
    
    let a = actuator_snapshot.axis_1;
    let b = actuator_snapshot.axis_2;
    
    let physical_x = 0.5 * (a + b);
    let physical_y = 0.5 * (a - b);
    
    const VIS_SCALE: f32 = 0.01;
    let bed_x_offset = config.0.limits.x.max * VIS_SCALE / 2.0;
    let bed_y_offset = config.0.limits.y.max * VIS_SCALE / 2.0;

    for mut transform in query.iter_mut() {
        // Apply to transform.
        // Bevy X axis maps to physical X.
        // Bevy Z axis maps to physical Y.
        // We set the local translation relative to its parent (the frame).
        
        // The frame spawned it at (-bed_x/2, -0.1, -bed_y/2) so that logical (0,0) 
        // is at that corner. We add the calculated offset.
        transform.translation.x = (physical_x * VIS_SCALE) - bed_x_offset;
        transform.translation.z = (physical_y * VIS_SCALE) - bed_y_offset;
    }
}
