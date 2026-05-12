//! Global simulation resources that act as a bridge between Bevy and the headless engine.

use bevy::prelude::*;
use core_types::TargetState;
use engine::MachineContext;
use gcode_parser::GCodeCommand;
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

/// A stub for a future advanced acceleration and motion planner.
/// Currently it only acts as an architectural placeholder.
#[derive(Debug, Default)]
pub struct MotionPlanner {
    // TODO: Add fields for trapezoidal/S-curve acceleration profiles.
}

/// Stores the state of the G-code playback engine.
#[derive(Resource, Default)]
pub struct GCodePlayer {
    pub commands: Vec<GCodeCommand>,
    pub current_index: usize,
    pub is_playing: bool,
    pub is_absolute: bool,
    pub current_feedrate: f32, // mm/min
    pub target_point: TargetState,
    pub motion_planner: MotionPlanner,
}

impl GCodePlayer {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
            is_playing: false,
            is_absolute: true,
            current_feedrate: 3000.0, // Default 50mm/s
            target_point: TargetState::default(),
            motion_planner: MotionPlanner::default(),
        }
    }

    pub fn load(&mut self, commands: Vec<GCodeCommand>) {
        self.commands = commands;
        self.current_index = 0;
        self.is_playing = false;
        // Keep current settings (feedrate, absolute) or reset them?
        // Usually safe to reset them on new file load, except target_point which is real position.
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.is_playing = false;
    }
}
