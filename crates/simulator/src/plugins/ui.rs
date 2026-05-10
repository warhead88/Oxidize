//! User interface plugin using bevy_egui.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::resources::MachineState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, manual_control_ui);
    }
}

/// Renders the manual control panel and feeds target values back to the engine.
fn manual_control_ui(
    mut contexts: EguiContexts,
    machine_state: ResMut<MachineState>,
) {
    egui::Window::new("Manual Control").show(contexts.ctx_mut(), |ui| {
        // Lock the engine state to read/modify targets
        let mut engine = machine_state.0.lock().unwrap();
        let mut target = *engine.current_target(); // Implements Copy

        let mut changed = false;

        ui.label("Linear Axes");
        changed |= ui.add(egui::Slider::new(&mut target.x, -200.0..=200.0).text("X")).changed();
        changed |= ui.add(egui::Slider::new(&mut target.y, -200.0..=200.0).text("Y")).changed();
        changed |= ui.add(egui::Slider::new(&mut target.z, 0.0..=400.0).text("Z")).changed();

        ui.separator();

        ui.label("Rotary Axes");
        // For visual consistency, you might display degrees in UI but keep target as radians,
        // but here we just manipulate the target fields directly assuming they match UI units.
        changed |= ui.add(egui::Slider::new(&mut target.a, -std::f32::consts::PI..=std::f32::consts::PI).text("A")).changed();
        changed |= ui.add(egui::Slider::new(&mut target.c, -std::f32::consts::PI..=std::f32::consts::PI).text("C")).changed();

        // If any slider was dragged, update the engine target
        if changed {
            engine.set_target(target);
        }
    });
}
