use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

mod convert;

pub struct LegionError {
    kind: Box<LegionErrorKind>,
}

#[derive(Debug)]
pub enum LegionErrorKind {
    Custom { message: String },
}

impl Error for LegionError {}

impl Debug for LegionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}
impl Display for LegionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.kind, f)
    }
}
impl Display for LegionErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LegionErrorKind::Custom { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl AsRef<LegionErrorKind> for LegionError {
    fn as_ref(&self) -> &LegionErrorKind {
        &self.kind
    }
}

impl AsMut<LegionErrorKind> for LegionError {
    fn as_mut(&mut self) -> &mut LegionErrorKind {
        &mut self.kind
    }
}
