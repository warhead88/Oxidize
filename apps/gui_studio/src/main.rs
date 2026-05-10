//! Entry point for the 5-Axis CAM Studio application.
//! Responsible only for bootstrapping: parsing CLI, loading config,
//! building the Bevy App. No business logic lives here.

mod cli;
mod config_loader;

use anyhow::Context;
use bevy::prelude::*;
use clap::Parser;
use cli::Cli;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    // --- 1. Parse CLI arguments ---
    let cli = Cli::parse();

    // --- 2. Initialize the tracing logger ---
    // If --debug was passed, use DEBUG level; otherwise INFO.
    let log_level = if cli.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .init();

    // --- 3. Load machine configuration ---
    let config_path = cli.config.as_deref().unwrap_or("printer.toml");
    let config = config_loader::load_or_default(config_path)?;

    info!(
        kinematics_type = ?config.kinematics_type,
        "Machine configuration loaded successfully."
    );

    // --- 4. Build kinematics from config (factory pattern) ---
    let active_kinematics = kinematics::factory::build_kinematics(config.clone())
        .context("Failed to build kinematics from the loaded configuration")?;

    // --- 5. Initialize the headless engine core ---
    let machine = engine::MachineContext::new(active_kinematics);
    info!("MachineContext initialized.");

    // --- 6. Assemble and run the Bevy App ---
    if cli.headless {
        // Headless mode: run without a window (future use for background slicing).
        tracing::warn!("Headless mode is not fully implemented yet. Running in dry-run mode.");
        // In the future, this would add a headless Bevy schedule and run slicing tasks.
    } else {
        info!("Starting GUI Studio in windowed mode.");
        App::new()
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "5A Slicer — CAM Studio".into(),
                    ..default()
                }),
                ..default()
            }))
            // Inject the fully initialized machine context into the simulator plugin
            .add_plugins(simulator::SimulatorPlugin::new(config.clone(), machine))
            .run();
    }

    Ok(())
}
