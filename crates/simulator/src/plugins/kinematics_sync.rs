//! Synchronizes physical state from `engine` to Bevy entities via `KinematicLink`.

use bevy::prelude::*;
use crate::{components::{ActuatorId, AxisMapping, KinematicLink}, resources::MachineState};
use core_types::ActuatorState;

pub struct KinematicsSyncPlugin;

impl Plugin for KinematicsSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_transforms_system);
    }
}

/// A thin system that reads actuator states from the Engine and applies 
/// them to the Transform of every entity with a `KinematicLink`.
fn update_transforms_system(
    machine_state: Res<MachineState>,
    mut query: Query<(&mut Transform, &KinematicLink)>,
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
