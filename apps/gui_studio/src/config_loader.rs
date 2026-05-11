//! Configuration loading logic.
//! Reads a `printer.toml` file and deserializes it into `KinematicsConfig`.
//! If the file is absent, a default CoreXY config is generated and saved to disk as a template.

use anyhow::Context;
use kinematics::config::{AxisLimits, KinematicsConfig, KinematicsType, MachineLimits};
use std::fs;
use tracing::{info, warn};

/// Default axis limits: 0 to 300mm.
fn default_axis_limits() -> AxisLimits {
    AxisLimits { min: 0.0, max: 300.0 }
}

/// Generates a sensible default config: CoreXY, 300x300x300mm work volume.
fn default_config() -> KinematicsConfig {
    KinematicsConfig {
        kinematics_type: KinematicsType::CoreXY,
        limits: MachineLimits {
            x: default_axis_limits(),
            y: default_axis_limits(),
            z: default_axis_limits(),
            a: AxisLimits { min: -180.0, max: 180.0 },
            c: AxisLimits { min: -360.0, max: 360.0 },
        },
        // None for non-5-axis configs; set to Some(TrunnionGeometry{...}) for TrunnionCoreXY.
        trunnion_geometry: None,
    }
}

/// Attempts to load a `KinematicsConfig` from the given TOML file path.
/// If the file does not exist, generates a default config, writes it to disk as
/// a template, and returns it.
pub fn load_or_default(path: &str) -> anyhow::Result<KinematicsConfig> {
    if fs::metadata(path).is_ok() {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {path}"))?;
        let config: KinematicsConfig = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {path}"))?;
        info!("Loaded printer config from '{path}'");
        Ok(config)
    } else {
        warn!(
            "Config file '{path}' not found. Generating a default CoreXY template and saving to disk."
        );
        let config = default_config();

        // Serialize and write the default config to disk as a starter template
        let toml_string = toml::to_string_pretty(&config)
            .context("Failed to serialize default config to TOML")?;
        fs::write(path, toml_string)
            .with_context(|| format!("Failed to write default config to '{path}'"))?;

        info!("Default config written to '{path}'. Edit it to match your machine.");
        Ok(config)
    }
}
