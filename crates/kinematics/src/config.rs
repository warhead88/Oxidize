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

/// Physical dimensions and mass of the printhead assembly.
/// Used by the slicer for toolpath collision avoidance and by the
/// visualizer to render an accurate composite head mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintHeadGeometry {
    /// Width of the main hotend block in mm (printer X).
    pub width: f32,
    /// Height of the main hotend block in mm.
    pub height: f32,
    /// Depth of the main hotend block in mm (printer Y).
    pub depth: f32,
    /// Length of the nozzle tip protruding downward in mm.
    pub nozzle_length: f32,
    /// Approximate mass in grams (reserved for acceleration calculations).
    pub mass: f32,
}

impl Default for PrintHeadGeometry {
    fn default() -> Self {
        Self { width: 40.0, height: 50.0, depth: 40.0, nozzle_length: 20.0, mass: 300.0 }
    }
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
    /// Optional printhead geometry. Falls back to `PrintHeadGeometry::default()` if absent.
    pub head_geometry: Option<PrintHeadGeometry>,
}
