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

/// Defines the underlying physical kinematics layout of the machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KinematicsType {
    /// Standard Cartesian printer (e.g. bed moves Y, head moves X/Z).
    Cartesian,
    /// CoreXY mechanism where X and Y are coupled via belts.
    CoreXY,
}

/// The main kinematics configuration loaded from a file (e.g. `printer.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinematicsConfig {
    pub kinematics_type: KinematicsType,
    pub limits: MachineLimits,
}
