//! Main simulator crate for the 5-axis 3D printer CAM engine.
//! Provides a Bevy-based 3D visualization and UI without hardcoded machine logic.

pub mod components;
pub mod plugins;
pub mod resources;

use bevy::prelude::*;
use plugins::{kinematics_sync::KinematicsSyncPlugin, scene::ScenePlugin, ui::UiPlugin};
use resources::{MachineState, ViewportConfig};
use engine::MachineContext;
use kinematics::Kinematics;
use core_types::{ActuatorState, TargetState};
use std::sync::Mutex;

// Dummy kinematics implementation just to satisfy the initial MachineContext
struct DummyKinematics;
impl Kinematics for DummyKinematics {
    fn forward_kinematics(&self, _target: &TargetState) -> ActuatorState {
        ActuatorState::default()
    }
    fn inverse_kinematics(&self, _actuators: &ActuatorState) -> TargetState {
        TargetState::default()
    }
}

/// The main Simulator plugin that groups all simulation and UI subsystems.
pub struct SimulatorPlugin;

impl Plugin for SimulatorPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the core MachineContext inside a Mutex for thread-safe interior mutability
        let engine = MachineContext::new(Box::new(DummyKinematics));
        
        app.insert_resource(MachineState(Mutex::new(engine)))
            .insert_resource(ViewportConfig {
                show_grid: true,
                show_axis: true,
            })
            // Add sub-plugins
            .add_plugins((
                ScenePlugin,
                KinematicsSyncPlugin,
                UiPlugin,
            ));
    }
}
