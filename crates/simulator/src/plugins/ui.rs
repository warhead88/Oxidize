//! User interface plugin using bevy_egui.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::resources::{MachineConfig, MachineState, GCodePlayer};
use gcode_parser::{parse_line, GCodeCommand};
use std::fs;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, manual_control_ui);
    }
}

/// Renders the manual control panel and feeds target values back to the engine.
/// Slider ranges are read from `MachineConfig.limits` instead of being hardcoded.
fn manual_control_ui(
    mut contexts: EguiContexts,
    machine_state: ResMut<MachineState>,
    config: Res<MachineConfig>,
    mut player: ResMut<GCodePlayer>,
) {
    let limits = &config.0.limits;

    egui::Window::new("Manual Control").show(contexts.ctx_mut(), |ui| {
        let mut engine = machine_state.0.lock().unwrap();
        let mut target = *engine.current_target();
        let mut changed = false;

        ui.label("Linear Axes");
        changed |= ui.add(egui::Slider::new(&mut target.x, limits.x.min..=limits.x.max).text("X (mm)")).changed();
        changed |= ui.add(egui::Slider::new(&mut target.y, limits.y.min..=limits.y.max).text("Y (mm)")).changed();
        changed |= ui.add(egui::Slider::new(&mut target.z, limits.z.min..=limits.z.max).text("Z (mm)")).changed();

        ui.separator();

        ui.label("Rotary Axes");
        changed |= ui.add(egui::Slider::new(&mut target.a, limits.a.min..=limits.a.max).text("A (°)")).changed();
        changed |= ui.add(egui::Slider::new(&mut target.c, limits.c.min..=limits.c.max).text("C (°)")).changed();

        if changed {
            engine.set_target(target);
        }
    });

    egui::Window::new("G-code Player").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Load test.gcode").clicked() {
                if let Ok(contents) = fs::read_to_string("test.gcode") {
                    let commands: Vec<GCodeCommand> = contents
                        .lines()
                        .map(parse_line)
                        .filter(|c| *c != GCodeCommand::Other)
                        .collect();
                    player.load(commands);
                    info!("Loaded {} G-code commands", player.commands.len());
                } else {
                    error!("Could not read test.gcode from project root");
                }
            }

            if ui.button(if player.is_playing { "Pause" } else { "Play" }).clicked() {
                player.is_playing = !player.is_playing;
            }

            if ui.button("Reset").clicked() {
                player.reset();
            }
        });

        ui.separator();

        let total = player.commands.len();
        let current = player.current_index;
        ui.label(format!("Progress: {} / {}", current, total));
        ui.label(format!("Feedrate: {} mm/min", player.current_feedrate));
        ui.label(format!("Mode: {}", if player.is_absolute { "Absolute (G90)" } else { "Relative (G91)" }));
    });
}
