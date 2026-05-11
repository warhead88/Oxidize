//! ECS Components for the hierarchical kinematics tree.

use bevy::prelude::*;

/// Abstract identifiers for physical actuator channels (stepper motors).
/// Named neutrally (`Actuator1..5`) to avoid semantic confusion between
/// Cartesian (where Actuator1=X) and CoreXY (where Actuator1=MotorA, not X).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActuatorId {
    Actuator1,
    Actuator2,
    Actuator3,
    Actuator4,
    Actuator5,
    Extruder,
}

/// Defines how the value from an actuator applies to this entity's transform.
#[derive(Debug, Clone, Copy)]
pub enum AxisMapping {
    /// Moves the entity along the specified vector (relative to its `BaseTransform`).
    /// The actuator value (in mm) is multiplied by this vector.
    Translation(Vec3),
    /// Rotates the entity around the specified axis.
    /// The actuator value represents an angle in radians.
    Rotation(Vec3),
}

/// Stores the resting (zero-position) transform of a kinematic entity.
/// The sync system adds the actuator's offset to this base value each frame,
/// preventing the initial `from_xyz` position from being overwritten.
#[derive(Component, Clone, Copy)]
pub struct BaseTransform(pub Vec3);

/// A component that marks a 3D entity as being driven by a specific physical actuator.
/// This allows us to map `engine` state to Bevy transforms dynamically.
#[derive(Component)]
pub struct KinematicLink {
    /// The motor channel that drives this link.
    pub actuator: ActuatorId,
    /// How the motor's position translates to spatial transformation.
    pub mapping: AxisMapping,
}

/// A component that marks the toolhead in a CoreXY system.
/// The CoreXY toolhead's position depends on two actuators simultaneously,
/// so it requires a dedicated synchronization system.
#[derive(Component)]
pub struct CoreXyHeadLink;

/// A component that marks the transverse gantry in a CoreXY system.
/// It moves along the Y-axis (Bevy Z) and carries the X-axis rail.
#[derive(Component)]
pub struct CoreXyGantryLink;
