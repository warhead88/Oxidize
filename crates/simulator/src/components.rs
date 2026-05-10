//! ECS Components for the hierarchical kinematics tree.

use bevy::prelude::*;

/// Identifiers for abstract physical actuators (motors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActuatorId {
    AxisX,
    AxisY,
    AxisZ,
    AxisA,
    AxisC,
    Extruder,
}

/// Defines how the value from an actuator applies to this entity's transform.
#[derive(Debug, Clone, Copy)]
pub enum AxisMapping {
    /// Moves the entity along the specified vector. 
    /// The actuator value is multiplied by this vector.
    Translation(Vec3),
    /// Rotates the entity around the specified axis. 
    /// The actuator value represents an angle, multiplying the axis.
    Rotation(Vec3),
}

/// A component that marks a 3D entity as being driven by a specific physical actuator.
/// This allows us to map `engine` state to Bevy transforms dynamically.
#[derive(Component)]
pub struct KinematicLink {
    /// The motor that drives this link.
    pub actuator: ActuatorId,
    /// How the motor's position translates to spatial transformation.
    pub mapping: AxisMapping,
}
