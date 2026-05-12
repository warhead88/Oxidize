//! Synchronizes physical state from `engine` to Bevy entities via `KinematicLink`.

use bevy::prelude::*;
use crate::{
    components::{ActuatorId, AxisMapping, BaseTransform, CoreXyGantryLink, CoreXyHeadLink, KinematicLink},
    resources::{MachineConfig, MachineState, GCodePlayer},
};
use core_types::TargetState;
use gcode_parser::GCodeCommand;
use crate::plugins::scene::VIS_SCALE;
use core_types::ActuatorState;

pub struct KinematicsSyncPlugin;

impl Plugin for KinematicsSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_transforms_system, sync_corexy_head, gcode_playback_system));
    }
}

/// Reads actuator states from the Engine and applies them to every entity with
/// a `KinematicLink`, offset from its `BaseTransform` (resting position).
///
/// FIX: Previously `transform.translation = axis * value` which erased the
/// entity's initial spawn position. Now: `base + axis * value`.
fn update_transforms_system(
    machine_state: Res<MachineState>,
    mut query: Query<(&mut Transform, &KinematicLink, &BaseTransform), (Without<CoreXyHeadLink>, Without<CoreXyGantryLink>)>,
) {
    let actuators: ActuatorState = {
        let lock = machine_state.0.lock().unwrap();
        *lock.current_actuators()
    };

    for (mut transform, link, base) in query.iter_mut() {
        let value = match link.actuator {
            ActuatorId::Actuator1 => actuators.axis_1,
            ActuatorId::Actuator2 => actuators.axis_2,
            ActuatorId::Actuator3 => actuators.axis_3,
            ActuatorId::Actuator4 => actuators.axis_4,
            ActuatorId::Actuator5 => actuators.axis_5,
            ActuatorId::Extruder  => actuators.extruder,
        };

        match link.mapping {
            AxisMapping::Translation(axis) => {
                // FIX: Add offset to base, don't replace.
                transform.translation = base.0 + axis * value;
            }
            AxisMapping::Rotation(axis) => {
                // The angle from the engine is in degrees, Bevy expects radians.
                transform.rotation = Quat::from_axis_angle(axis.normalize(), value.to_radians());
            }
        }
    }
}

/// Dedicated sync system for the CoreXY toolhead.
///
/// FIX: Previously duplicated `inverse_kinematics` math by reading raw actuator
/// values (axis_1, axis_2) and recalculating X/Y. Now reads `current_target()`
/// directly from the engine, which already knows the logical X/Y position.
fn sync_corexy_head(
    machine_state: Res<MachineState>,
    config: Res<MachineConfig>,
    mut head_query: Query<&mut Transform, (With<CoreXyHeadLink>, Without<CoreXyGantryLink>)>,
    mut gantry_query: Query<&mut Transform, (With<CoreXyGantryLink>, Without<CoreXyHeadLink>)>,
) {
    let (target_x, target_y) = {
        let lock = machine_state.0.lock().unwrap();
        let t = lock.current_target();
        (t.x, t.y)
    };

    let bed_x_half = 0.0; // Corner-based: start at 0
    let bed_y_half = 0.0; // Corner-based: start at 0

    // 1. Move Gantry along Y (Bevy Z)
    for mut transform in gantry_query.iter_mut() {
        // GCode Y+ moves nozzle to back. In our setup, nozzle is at Z=0 world.
        // CoreXY Y+ moves gantry to back (Bevy Z-).
        transform.translation.z = -(target_y * VIS_SCALE);
    }

    // 2. Move Head along X
    for mut transform in head_query.iter_mut() {
        transform.translation.x = (target_x * VIS_SCALE);
    }
}

/// Plays back loaded G-code commands, interpolating `MachineState.target`
/// based on the current feedrate and frame delta time.
fn gcode_playback_system(
    time: Res<Time>,
    machine_state: Res<MachineState>,
    mut player: ResMut<GCodePlayer>,
) {
    if !player.is_playing || player.current_index >= player.commands.len() {
        if player.is_playing && player.current_index >= player.commands.len() {
            player.is_playing = false; // Stop when finished
        }
        return;
    }

    let command = player.commands[player.current_index].clone();
    match command {
        GCodeCommand::SetAbsolutePositioning => {
            player.is_absolute = true;
            player.current_index += 1;
        }
        GCodeCommand::SetRelativePositioning => {
            player.is_absolute = false;
            player.current_index += 1;
        }
        GCodeCommand::Other => {
            player.current_index += 1;
        }
        GCodeCommand::Move { x, y, z, a, c, e, f } => {
            if let Some(feedrate) = f {
                player.current_feedrate = feedrate;
            }

            let mut engine = machine_state.0.lock().unwrap();
            let current = *engine.current_target();

            let target_point = if player.is_absolute {
                TargetState {
                    x: x.unwrap_or(current.x),
                    y: y.unwrap_or(current.y),
                    z: z.unwrap_or(current.z),
                    a: a.unwrap_or(current.a),
                    c: c.unwrap_or(current.c),
                    e: e.unwrap_or(current.e),
                }
            } else {
                TargetState {
                    x: current.x + x.unwrap_or(0.0),
                    y: current.y + y.unwrap_or(0.0),
                    z: current.z + z.unwrap_or(0.0),
                    a: current.a + a.unwrap_or(0.0),
                    c: current.c + c.unwrap_or(0.0),
                    e: current.e + e.unwrap_or(0.0),
                }
            };

            player.target_point = target_point;

            let dx = target_point.x - current.x;
            let dy = target_point.y - current.y;
            let dz = target_point.z - current.z;
            let da = target_point.a - current.a;
            let dc = target_point.c - current.c;
            let de = target_point.e - current.e;

            // Simplistic distance metric combining all axes
            let dist_sq = dx * dx + dy * dy + dz * dz + da * da + dc * dc + de * de;
            let dist = dist_sq.sqrt();

            if dist < 0.001 {
                engine.set_target(target_point);
                player.current_index += 1;
            } else {
                let speed_mm_s = player.current_feedrate / 60.0;
                let step_dist = speed_mm_s * time.delta_seconds();

                if step_dist >= dist {
                    engine.set_target(target_point);
                    player.current_index += 1;
                } else {
                    let ratio = step_dist / dist;
                    let next_target = TargetState {
                        x: current.x + dx * ratio,
                        y: current.y + dy * ratio,
                        z: current.z + dz * ratio,
                        a: current.a + da * ratio,
                        c: current.c + dc * ratio,
                        e: current.e + de * ratio,
                    };
                    engine.set_target(next_target);
                }
            }
        }
    }
}
