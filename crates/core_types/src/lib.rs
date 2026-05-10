//! Basic primitive types for the 5-axis 3D printer engine.
//! Contains logical and physical state representations.

/// Represents the logical state (coordinates) of the machine, as defined by G-code or a slicer.
/// This defines the target position in 3D space along with tilt, rotation, and extrusion.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TargetState {
    /// X-axis coordinate in logical space
    pub x: f32,
    /// Y-axis coordinate in logical space
    pub y: f32,
    /// Z-axis coordinate in logical space
    pub z: f32,
    /// Extruder position or flow amount
    pub e: f32,
    /// A-axis (tilt/rotation) logical coordinate
    pub a: f32,
    /// C-axis (rotation) logical coordinate
    pub c: f32,
}

/// Represents the physical state of the machine's actuators (stepper motors).
/// This is the abstract physical representation of the individual axes' positions.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ActuatorState {
    /// Position of the first physical axis motor
    pub axis_1: f32,
    /// Position of the second physical axis motor
    pub axis_2: f32,
    /// Position of the third physical axis motor
    pub axis_3: f32,
    /// Position of the fourth physical axis motor
    pub axis_4: f32,
    /// Position of the fifth physical axis motor
    pub axis_5: f32,
    /// Position of the extruder motor
    pub extruder: f32,
}
