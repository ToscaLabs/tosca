use std::borrow::Cow;

use tracing::error;

/// All possible error kinds.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorKind {
    /// Errors encountered during the discovery service.
    Discovery,
    /// Errors encountered when sending requests to a device.
    Request,
    /// Errors caused by a wrong parameter.
    WrongParameter,
    /// Errors encountered while parsing a `json` response.
    JsonResponse,
    /// Errors encountered while parsing a byte stream response.
    StreamResponse,
    /// Errors encountered while constructing the request sender.
    Sender,
    /// Errors related to event management.
    Events,
}

impl ErrorKind {
    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::Discovery => "Discovery",
            Self::Request => "Request",
            Self::WrongParameter => "Wrong Parameter",
            Self::JsonResponse => "Json Response",
            Self::StreamResponse => "Stream Response",
            Self::Sender => "Response Sender",
            Self::Events => "Events",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.description().fmt(f)
    }
}

/// Controller error.
#[derive(PartialEq)]
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
    /// Creates an [`Error`] from an [`ErrorKind`] and a description.
    #[inline]
    pub fn new(kind: ErrorKind, description: impl Into<Cow<'static, str>>) -> Self {
        let description = description.into();
        error!("{}", description.as_ref());
        Self { kind, description }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.description)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::new(ErrorKind::Request, e.to_string())
    }
}

impl From<mdns_sd::Error> for Error {
    fn from(e: mdns_sd::Error) -> Self {
        Self::new(ErrorKind::Discovery, e.to_string())
    }
}

impl From<rumqttc::v5::ClientError> for Error {
    fn from(e: rumqttc::v5::ClientError) -> Self {
        Self::new(ErrorKind::Events, e.to_string())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}

/// A specialized [`Result`] type for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::{Error, ErrorKind};

    #[test]
    fn controller_error() {
        let error = Error::new(ErrorKind::Discovery, "Process failed.");

        assert_eq!(error.to_string(), r"Discovery: Process failed.");
    }
}
