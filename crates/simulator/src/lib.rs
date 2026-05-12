//! Main simulator crate for the 5-axis 3D printer CAM engine.
//! Provides a Bevy-based 3D visualization and UI without hardcoded machine logic.

pub mod components;
pub mod plugins;
pub mod resources;

use bevy::prelude::*;
use engine::MachineContext;
use kinematics::config::KinematicsConfig;
use plugins::{kinematics_sync::KinematicsSyncPlugin, scene::ScenePlugin, ui::UiPlugin};
use std::sync::Mutex;

/// The main Simulator plugin.
/// Accepts an externally-built `MachineContext` and `KinematicsConfig` so that
/// `main.rs` can inject a real kinematics model and the machine limits.
pub struct SimulatorPlugin {
    config: KinematicsConfig,
    /// Wrapped in Arc<Mutex<Option>> so we can `take()` in `build(&self, ...)`.
    machine: std::sync::Arc<Mutex<Option<MachineContext>>>,
}

impl SimulatorPlugin {
    /// Creates a new plugin, taking ownership of the initialized `MachineContext` and config.
    pub fn new(config: KinematicsConfig, machine: MachineContext) -> Self {
        Self {
            config,
            machine: std::sync::Arc::new(Mutex::new(Some(machine))),
        }
    }
}

impl Plugin for SimulatorPlugin {
    fn build(&self, app: &mut App) {
        let machine_context = self
            .machine
            .lock()
            .unwrap()
            .take()
            .expect("SimulatorPlugin::build called more than once");

        app.insert_resource(resources::MachineState(Mutex::new(machine_context)))
            .insert_resource(resources::MachineConfig(self.config.clone()))
            .init_resource::<resources::GCodePlayer>()
            .add_plugins((ScenePlugin, KinematicsSyncPlugin, UiPlugin));
    }
}
