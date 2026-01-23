#[cfg(feature = "wasm-opt")]
use wasm_opt::OptimizationError;
use crate::{LegionError, errors::LegionErrorKind};

impl From<std::io::Error> for LegionError {
    fn from(error: std::io::Error) -> Self {
        let kind = LegionErrorKind::Custom { message: error.to_string() };
        Self { kind: Box::new(kind) }
    }
}

impl From<anyhow::Error> for LegionError {
    fn from(error: anyhow::Error) -> Self {
        let kind = LegionErrorKind::Custom { message: error.to_string() };
        Self { kind: Box::new(kind) }
    }
}

impl From<wat::Error> for LegionError {
    fn from(error: wat::Error) -> Self {
        let kind = LegionErrorKind::Custom { message: error.to_string() };
        Self { kind: Box::new(kind) }
    }
}

impl From<dialoguer::Error> for LegionError {
    fn from(error: dialoguer::Error) -> Self {
        let kind = LegionErrorKind::Custom { message: error.to_string() };
        Self { kind: Box::new(kind) }
    }
}



#[cfg(feature = "wasm-opt")]
impl From<OptimizationError> for LegionError {
    fn from(error: OptimizationError) -> Self {
        let kind = LegionErrorKind::Custom { message: error.to_string() };
        Self { kind: Box::new(kind) }
    }
}
