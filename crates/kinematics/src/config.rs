//! Configuration layer for machine kinematics.
//! Ready for serde deserialization from TOML.

use serde::{Deserialize, Serialize};

/// Represents the minimum and maximum physical limits of a single axis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxisLimits {
    pub min: f32,
    pub max: f32,
}

/// Contains the physical limits for all axes in the machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineLimits {
    pub x: AxisLimits,
    pub y: AxisLimits,
    pub z: AxisLimits,
    pub a: AxisLimits,
    pub c: AxisLimits,
}

/// Defines the physical geometry offsets for a 5-axis trunnion table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrunnionGeometry {
    /// Height of the cradle (A-axis) pivot relative to the Z-carriage.
    pub pivot_a_offset_z: f32,
    /// Height of the rotary platter (C-axis) surface relative to the cradle pivot.
    pub platter_c_offset_z: f32,
}

/// Defines the underlying physical kinematics layout of the machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KinematicsType {
    /// Standard Cartesian printer (e.g. bed moves Y, head moves X/Z).
    Cartesian,
    /// CoreXY mechanism where X and Y are coupled via belts.
    CoreXY,
    /// 5-axis mechanism: CoreXY for X/Y head, Trunnion table for Z/A/C.
    TrunnionCoreXY,
}

/// The main kinematics configuration loaded from a file (e.g. `printer.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinematicsConfig {
    pub kinematics_type: KinematicsType,
    pub limits: MachineLimits,
    pub trunnion_geometry: Option<TrunnionGeometry>,
}
