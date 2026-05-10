//! Core engine module that manages the machine's state and orchestrates kinematics.

use core_types::{ActuatorState, TargetState};
use kinematics::Kinematics;

/// The core context of the machine.
/// Holds the current state of both logical targets and physical actuators,
/// and delegates the transformation to a dynamically dispatched kinematics implementation.
/// Designed to be used as a resource in an ECS system.
pub struct MachineContext {
    /// The current logical target state of the machine.
    current_target: TargetState,
    /// The current physical state of the machine's actuators.
    current_actuators: ActuatorState,
    /// The active kinematics model, allowing for different machine presets.
    kinematics: Box<dyn Kinematics>,
}

impl MachineContext {
    /// Creates a new `MachineContext` with the specified kinematics model.
    /// Initializes targets and actuators to their default states.
    pub fn new(kinematics: Box<dyn Kinematics>) -> Self {
        Self {
            current_target: TargetState::default(),
            current_actuators: ActuatorState::default(),
            kinematics,
        }
    }

    /// Updates the current logical target and automatically recalculates the
    /// required physical actuator positions using the active kinematics model.
    pub fn set_target(&mut self, new_target: TargetState) {
        self.current_actuators = self.kinematics.forward_kinematics(&new_target);
        self.current_target = new_target;
    }

    /// Gets a reference to the current logical target state.
    pub fn current_target(&self) -> &TargetState {
        &self.current_target
    }

    /// Gets a reference to the current physical actuator state.
    pub fn current_actuators(&self) -> &ActuatorState {
        &self.current_actuators
    }
}
