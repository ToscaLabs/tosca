use alloc::borrow::Cow;
use alloc::string::ToString;
use alloc::vec::Vec;

use tosca::device::DeviceInfo;
use tosca::response::{
    ErrorKind, ErrorResponse as ToscaErrorResponse, InfoResponse as ToscaInfoResponse,
    OkResponse as ToscaOkResponse, SERIALIZATION_ERROR, SerialResponse as ToscaSerialResponse,
};

use edge_http::io::Error;
use edge_http::io::server::Connection;

use embedded_io_async::{Read, Write};

use serde::Serialize;

/// A response which transmits a concise JSON message over the network to notify
/// a controller that an operation completed successfully.
pub struct OkResponse(Response);

impl Default for OkResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl OkResponse {
    /// Creates an [`OkResponse`].
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(json_to_response(Headers::json(), ToscaOkResponse::ok()))
    }
}

/// A response which transmits a JSON message over the network containing
/// the data produced during a device operation.
pub struct SerialResponse(Response);

impl SerialResponse {
    /// Creates a [`SerialResponse`] from a serializable value.
    #[must_use]
    #[inline]
    pub fn new<T: Serialize>(value: T) -> Self {
        Self(json_to_response(
            Headers::json(),
            ToscaSerialResponse::new(value),
        ))
    }

    /// Creates a [`SerialResponse`] from a given text.
    #[must_use]
    #[inline]
    pub fn text(value: &str) -> Self {
        let value = Cow::Borrowed(value);
        Self(json_to_response(
            Headers::json(),
            ToscaSerialResponse::new(value),
        ))
    }
}

/// A response which transmits a JSON message over the network containing
/// a device energy and economy information.
pub struct InfoResponse(Response);

impl InfoResponse {
    /// Creates a [`InfoResponse`].
    #[must_use]
    #[inline]
    pub fn new(device_info: DeviceInfo) -> Self {
        Self(json_to_response(
            Headers::json(),
            ToscaInfoResponse::new(device_info),
        ))
    }
}

/// A response providing details about an error encountered during a
/// device operation.
///
/// Contains the [`tosca::response::ErrorKind`], a general error description,
/// and optional information about the encountered error.
pub struct ErrorResponse(pub(crate) Response);

impl ErrorResponse {
    /// Generates an [`ErrorResponse`].
    ///
    /// Requires specifying the [`ErrorKind`] kind and a general
    /// description.
    #[must_use]
    #[inline]
    pub fn error(error: ErrorKind, description: &str) -> Self {
        Self(json_to_response(
            Headers::json_error(),
            ToscaErrorResponse::with_description(error, description),
        ))
    }

    /// Generates an [`ErrorResponse`].
    ///
    /// Requires specifying the [`ErrorKind`] kind, a general error
    /// description, and optional information about the encountered error.
    #[must_use]
    #[inline]
    pub fn error_with_info(error: ErrorKind, description: &str, info: &str) -> Self {
        Self(json_to_response(
            Headers::json_error(),
            ToscaErrorResponse::with_description_error(error, description, info),
        ))
    }

    /// An alias for the [`Self::error`] API, used to generate
    /// an [`ErrorResponse`] for invalid data.
    ///
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn invalid_data(description: &str) -> Self {
        Self::error(ErrorKind::InvalidData, description)
    }

    /// An alias for the [`Self::error`] API, used to generate
    /// an [`ErrorResponse`] for invalid data.
    ///
    ///
    /// Requires specifying a general error description and optional
    /// information about the encountered error.
    #[must_use]
    #[inline]
    pub fn invalid_data_with_error(description: &str, info: &str) -> Self {
        Self::error_with_info(ErrorKind::InvalidData, description, info)
    }

    /// An alias for the [`Self::error`] API, used to generate
    /// an [`ErrorResponse`] for an internal error.
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn internal(description: &str) -> Self {
        Self::error(ErrorKind::Internal, description)
    }

    /// An alias for the [`Self::error`] API, used to generate
    /// an [`ErrorResponse`] for an internal error.
    ///
    ///
    /// Requires specifying a general error description and optional
    /// information about the encountered error.
    #[must_use]
    #[inline]
    pub fn internal_with_error(description: &str, info: &str) -> Self {
        Self::error_with_info(ErrorKind::Internal, description, info)
    }
}

struct Headers {
    status: u16,
    message: &'static str,
    content_type: &'static [(&'static str, &'static str)],
}

impl Headers {
    const fn not_found() -> Self {
        Self {
            status: 404,
            message: "Not Found",
            content_type: &[],
        }
    }

    const fn not_allowed() -> Self {
        Self {
            status: 405,
            message: "Method Not Allowed",
            content_type: &[],
        }
    }

    const fn json() -> Self {
        Self {
            status: 200,
            message: "Ok",
            content_type: &[("Content-Type", "application/json")],
        }
    }

    const fn json_error() -> Self {
        Self {
            status: 500,
            message: "Error",
            content_type: &[("Content-Type", "application/json")],
        }
    }

    const fn serialization_error() -> Self {
        Self {
            status: 500,
            message: "Error",
            content_type: &[("Content-Type", "text/plain"), (SERIALIZATION_ERROR, "")],
        }
    }
}

struct Body(Cow<'static, [u8]>);

impl Body {
    const fn empty() -> Self {
        Self(Cow::Borrowed(&[]))
    }

    const fn static_ref(v: &'static [u8]) -> Self {
        Self(Cow::Borrowed(v))
    }

    const fn owned(v: Vec<u8>) -> Self {
        Self(Cow::Owned(v))
    }
}

#[inline]
fn json_to_response<T: Serialize>(headers: Headers, value: T) -> Response {
    match serde_json::to_vec(&value) {
        Ok(value) => Response::new(headers, Body::owned(value)),
        Err(e) => Response::new(
            Headers::serialization_error(),
            Body::owned(e.to_string().as_bytes().into()),
        ),
    }
}

pub(crate) struct Response {
    headers: Headers,
    body: Body,
}

impl From<Result<OkResponse, ErrorResponse>> for Response {
    #[inline]
    fn from(result: Result<OkResponse, ErrorResponse>) -> Response {
        match result {
            Ok(value) => value.0,
            Err(err) => err.0,
        }
    }
}

impl From<Result<SerialResponse, ErrorResponse>> for Response {
    #[inline]
    fn from(result: Result<SerialResponse, ErrorResponse>) -> Response {
        match result {
            Ok(value) => value.0,
            Err(err) => err.0,
        }
    }
}

impl From<Result<InfoResponse, ErrorResponse>> for Response {
    #[inline]
    fn from(result: Result<InfoResponse, ErrorResponse>) -> Response {
        match result {
            Ok(value) => value.0,
            Err(err) => err.0,
        }
    }
}

impl Response {
    #[inline]
    pub(crate) fn json<T: Serialize>(value: &T) -> Self {
        json_to_response(Headers::json(), value)
    }

    #[inline]
    pub(crate) async fn write<T, const N: usize>(
        self,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Error<T::Error>>
    where
        T: Read + Write,
    {
        self.write_from_ref(conn).await
    }

    #[inline]
    pub(crate) async fn write_from_ref<T, const N: usize>(
        &self,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Error<T::Error>>
    where
        T: Read + Write,
    {
        conn.initiate_response(
            self.headers.status,
            Some(self.headers.message),
            self.headers.content_type,
        )
        .await?;

        conn.write_all(&self.body.0).await
    }

    pub(crate) const fn not_found() -> Self {
        Response::new(Headers::not_found(), Body::empty())
    }

    pub(crate) const fn not_allowed() -> Self {
        Response::new(
            Headers::not_allowed(),
            Body::static_ref("Method not allowed".as_bytes()),
        )
    }

    const fn new(headers: Headers, body: Body) -> Response {
        Self { headers, body }
    }
}
