//! User interface plugin using bevy_egui.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::resources::{MachineConfig, MachineState};

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
}
