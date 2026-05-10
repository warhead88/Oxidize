//! CoreXY kinematics model.

use crate::{config::MachineLimits, Kinematics};
use core_types::{ActuatorState, TargetState};

/// Implements CoreXY kinematics where X and Y axes are coupled.
/// axis_1 corresponds to motor A, axis_2 to motor B.
pub struct CoreXYKinematics {
    pub limits: MachineLimits,
}

impl CoreXYKinematics {
    pub fn new(limits: MachineLimits) -> Self {
        Self { limits }
    }
}

impl Kinematics for CoreXYKinematics {
    fn forward_kinematics(&self, target: &TargetState) -> ActuatorState {
        ActuatorState {
            // CoreXY formulas
            axis_1: target.x + target.y,
            axis_2: target.x - target.y,
            
            // 1:1 mapping for Z, A, C and Extruder
            axis_3: target.z,
            axis_4: target.a,
            axis_5: target.c,
            extruder: target.e,
        }
    }

    fn inverse_kinematics(&self, actuators: &ActuatorState) -> TargetState {
        TargetState {
            // Inverse CoreXY formulas
            x: 0.5 * (actuators.axis_1 + actuators.axis_2),
            y: 0.5 * (actuators.axis_1 - actuators.axis_2),
            
            // 1:1 mapping for Z, A, C and Extruder
            z: actuators.axis_3,
            a: actuators.axis_4,
            c: actuators.axis_5,
            e: actuators.extruder,
        }
    }
}
