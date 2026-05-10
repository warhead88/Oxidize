//! Standard Cartesian kinematics model.

use crate::{config::MachineLimits, Kinematics};
use core_types::{ActuatorState, TargetState};

/// Implements standard Cartesian kinematics where each logical axis maps 1:1 to a physical actuator.
pub struct CartesianKinematics {
    pub limits: MachineLimits,
}

impl CartesianKinematics {
    pub fn new(limits: MachineLimits) -> Self {
        Self { limits }
    }
}

impl Kinematics for CartesianKinematics {
    fn forward_kinematics(&self, target: &TargetState) -> ActuatorState {
        ActuatorState {
            axis_1: target.x,
            axis_2: target.y,
            axis_3: target.z,
            axis_4: target.a,
            axis_5: target.c,
            extruder: target.e,
        }
    }

    fn inverse_kinematics(&self, actuators: &ActuatorState) -> TargetState {
        TargetState {
            x: actuators.axis_1,
            y: actuators.axis_2,
            z: actuators.axis_3,
            a: actuators.axis_4,
            c: actuators.axis_5,
            e: actuators.extruder,
        }
    }
}
