//! Global simulation resources that act as a bridge between Bevy and the headless engine.

use bevy::prelude::*;
use engine::MachineContext;
use kinematics::config::KinematicsConfig;
use std::sync::Mutex;

/// A thread-safe wrapper around the core `MachineContext`.
/// This is the single source of truth for the machine's state.
#[derive(Resource)]
pub struct MachineState(pub Mutex<MachineContext>);

/// A wrapper around the kinematics configuration, available as a Bevy Resource.
/// Used by scene-building and sync systems to read machine limits.
#[derive(Resource)]
pub struct MachineConfig(pub KinematicsConfig);
