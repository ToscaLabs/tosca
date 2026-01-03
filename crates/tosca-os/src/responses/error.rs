use tosca::response::{ErrorKind, ErrorResponse as ToscaErrorResponse};

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// A response providing details about an error encountered during a
/// device operation.
///
/// Contains an [`ErrorKind`], a general error description,
/// and optional information about the encountered error.
pub struct ErrorResponse(Response);

impl ErrorResponse {
    /// Generates an [`ErrorResponse`].
    ///
    /// Requires specifying an [`ErrorKind`] and a general description.
    #[must_use]
    #[inline]
    pub fn with_description(error: ErrorKind, description: &str) -> Self {
        let value = ToscaErrorResponse::with_description(error, description);
        Self((StatusCode::INTERNAL_SERVER_ERROR, Json(value)).into_response())
    }

    /// Generates an [`ErrorResponse`].
    ///
    /// Requires specifying an [`ErrorKind`], a general error
    /// description, and optional information about the encountered error.
    #[must_use]
    #[inline]
    pub fn with_description_error(error: ErrorKind, description: &str, info: &str) -> Self {
        let value = ToscaErrorResponse::with_description_error(error, description, info);
        Self((StatusCode::INTERNAL_SERVER_ERROR, Json(value)).into_response())
    }

    /// Generates an [`ErrorResponse`] for invalid data.
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn invalid_data(description: &str) -> Self {
        Self::with_description(ErrorKind::InvalidData, description)
    }

    /// Generates an [`ErrorResponse`] for invalid data.
    ///
    /// Requires specifying a general error description and optional
    /// information about the encountered error.
    #[must_use]
    #[inline]
    pub fn invalid_data_with_error(description: &str, error: &str) -> Self {
        Self::with_description_error(ErrorKind::InvalidData, description, error)
    }

    /// Generates an [`ErrorResponse`] for an internal error.
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn internal(description: &str) -> Self {
        Self::with_description(ErrorKind::Internal, description)
    }

    /// Generates an [`ErrorResponse`] for an internal error.
    ///
    ///
    /// Requires specifying a general error description and optional
    /// information about the encountered error.
    #[must_use]
    #[inline]
    pub fn internal_with_error(description: &str, error: &str) -> Self {
        Self::with_description_error(ErrorKind::Internal, description, error)
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        self.0
    }
}
