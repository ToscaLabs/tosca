use std::borrow::Cow;

/// All possible error kinds.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ErrorKind {
    /// Failed to find a device ID.
    NoIdFound,
    /// Not found address.
    NotFoundAddress,
    /// Mandatory routes are missing or invalid.
    MandatoryRoutes,
    /// Errors encountered while serializing or deserializing a file.
    Serialization,
    /// Errors encountered while configuring the discovery service.
    Service,
}

impl ErrorKind {
    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::NoIdFound => "No Device ID Found",
            Self::NotFoundAddress => "Not Found Address",
            Self::MandatoryRoutes => "Mandatory Routes",
            Self::Serialization => "Serialization",
            Self::Service => "Service",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A library error.
#[cfg_attr(test, derive(PartialEq))]
pub struct Error {
    kind: ErrorKind,
    description: Cow<'static, str>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, description: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind,
            description: description.into(),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.kind)?;
        write!(f, "Cause: {}", self.description)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Serialization, e.to_string())
    }
}

/// A specialized [`Result`] type for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
