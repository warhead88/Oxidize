//! Global simulation resources that act as a bridge between Bevy and the headless engine.

use bevy::prelude::*;
use engine::MachineContext;
use std::sync::Mutex;

/// A thread-safe wrapper around the core `MachineContext`.
/// This is the single source of truth for the machine's state.
#[derive(Resource)]
pub struct MachineState(pub Mutex<MachineContext>);

/// Configuration for the 3D viewport representation.
#[derive(Resource)]
pub struct ViewportConfig {
    pub show_grid: bool,
    pub show_axis: bool,
}
