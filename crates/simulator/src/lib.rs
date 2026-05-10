//! Main simulator crate for the 5-axis 3D printer CAM engine.
//! Provides a Bevy-based 3D visualization and UI without hardcoded machine logic.

pub mod components;
pub mod plugins;
pub mod resources;

use bevy::prelude::*;
use engine::MachineContext;
use kinematics::config::KinematicsConfig;
use plugins::{kinematics_sync::KinematicsSyncPlugin, scene::ScenePlugin, ui::UiPlugin};
use resources::ViewportConfig;
use std::sync::Mutex;

/// The main Simulator plugin.
/// Accepts an externally-built `MachineContext` so that `main.rs` can inject
/// a real kinematics model instead of a dummy placeholder.
pub struct SimulatorPlugin {
    /// The kinematics configuration, used to build the correct scene hierarchy.
    config: KinematicsConfig,
    /// The initial machine state. Wrapped in `Option` so that it can be
    /// taken (moved) out in `build()`, which receives `&self`.
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
        // Extract the MachineContext from the Arc<Mutex<Option<...>>> and wrap it
        // in the Bevy Resource type expected by all simulator systems.
        let machine_context = self
            .machine
            .lock()
            .unwrap()
            .take()
            .expect("SimulatorPlugin::build called more than once");

        app.insert_resource(resources::MachineState(Mutex::new(machine_context)))
            .insert_resource(resources::MachineConfig(self.config.clone())) // Provide the config as a Resource for the scene setup
            .insert_resource(ViewportConfig {
                show_grid: true,
                show_axis: true,
            })
            .add_plugins((ScenePlugin, KinematicsSyncPlugin, UiPlugin));
    }
}
