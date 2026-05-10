//! Factory for building kinematics profiles.

use crate::{
    config::{KinematicsConfig, KinematicsType},
    models::{cartesian::CartesianKinematics, corexy::CoreXYKinematics},
    Kinematics,
};
use thiserror::Error;

/// Errors that can occur during kinematics initialization.
#[derive(Debug, Error)]
pub enum KinematicsError {
    #[error("Unknown or unsupported kinematics type: {0}")]
    UnsupportedType(String),
}

/// Builds a boxed kinematics instance based on the provided configuration.
pub fn build_kinematics(config: KinematicsConfig) -> Result<Box<dyn Kinematics>, KinematicsError> {
    match config.kinematics_type {
        KinematicsType::Cartesian => {
            let model = CartesianKinematics::new(config.limits);
            Ok(Box::new(model))
        }
        KinematicsType::CoreXY => {
            let model = CoreXYKinematics::new(config.limits);
            Ok(Box::new(model))
        }
        // If there were other unsupported types parsed, we would use KinematicsError::UnsupportedType
    }
}
