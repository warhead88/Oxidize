//! Kinematics abstractions for translating logical positions to physical actuator states and vice versa.

use core_types::{ActuatorState, TargetState};

pub mod config;
pub mod factory;
pub mod models;

// Re-export common structures for use by other crates
pub use config::{AxisLimits, KinematicsConfig, KinematicsType, MachineLimits};
pub use factory::{build_kinematics, KinematicsError};

/// Interface for machine kinematics.
/// Implementations of this trait define the transformation between logical coordinates
/// (what the user/G-code wants) and physical actuator positions (what the motors must do).
/// It requires `Send + Sync` to ensure compatibility with multithreaded environments (e.g., Bevy ECS).
pub trait Kinematics: Send + Sync {
    /// Translates a logical target state into physical actuator positions.
    fn forward_kinematics(&self, target: &TargetState) -> ActuatorState;

    /// Translates physical actuator positions back into a logical state.
    fn inverse_kinematics(&self, actuators: &ActuatorState) -> TargetState;
}
