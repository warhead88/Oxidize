//! Trunnion CoreXY kinematics model.
//! Combines CoreXY for X/Y with a tilt-rotary table (Z, A, C).

use crate::{config::{MachineLimits, TrunnionGeometry}, Kinematics};
use core_types::{ActuatorState, TargetState};

/// Implements 5-axis kinematics with a CoreXY toolhead and a Trunnion table.
pub struct TrunnionCoreXYKinematics {
    pub limits: MachineLimits,
    pub geometry: TrunnionGeometry,
}

impl TrunnionCoreXYKinematics {
    pub fn new(limits: MachineLimits, geometry: TrunnionGeometry) -> Self {
        Self { limits, geometry }
    }
}

impl Kinematics for TrunnionCoreXYKinematics {
    fn forward_kinematics(&self, target: &TargetState) -> ActuatorState {
        // TODO: Здесь будет расчёт RTCP (Rotation Tool Center Point) для компенсации оффсетов при G-code слайсинге.
        
        ActuatorState {
            // CoreXY formulas for head
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
