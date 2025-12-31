use tosca::response::{InfoResponse, OkResponse, SerialResponse};

use reqwest::Response as ReqwestResponse;

use serde::{Serialize, de::DeserializeOwned};

use crate::error::{Error, ErrorKind, Result};

// TODO:
// OkCollector --> Save Ok responses in order to maintain a history.
// SerialCollector --> Save serial responses in order to maintain a history.
// InfoCollector --> Save Info responses in order to maintain a history.
// StreamCollector --> Save information about a Stream Response before and after

async fn json_response<T>(response: ReqwestResponse) -> Result<T>
where
    T: Serialize + DeserializeOwned,
{
    response
        .json::<T>()
        .await
        .map_err(|e| Error::new(ErrorKind::JsonResponse, format!("Json error caused by {e}")))
}

/// An [`OkResponse`] body parser.
pub struct OkResponseParser(ReqwestResponse);

impl OkResponseParser {
    /// Parses the internal response body to retrieve an [`OkResponse`].
    ///
    /// # Errors
    ///
    /// If the response body does not contain a valid [`OkResponse`], a
    /// parsing error will be raised. This may occur due to an incorrect format
    /// or because the binary data contains syntactic or semantic errors.
    pub async fn parse_body(self) -> Result<OkResponse> {
        json_response::<OkResponse>(self.0).await
    }

    pub(crate) const fn new(response: ReqwestResponse) -> Self {
        Self(response)
    }
}

/// A [`SerialResponse`] body parser.
pub struct SerialResponseParser(ReqwestResponse);

impl SerialResponseParser {
    /// Parses the internal response body to retrieve a [`SerialResponse`].
    ///
    /// # Errors
    ///
    /// If the response body does not contain a valid [`SerialResponse`], a
    /// parsing error will be raised. This may occur due to an incorrect format
    /// or because the binary data contains syntactic or semantic errors.
    pub async fn parse_body<T: Serialize + DeserializeOwned>(self) -> Result<SerialResponse<T>> {
        json_response::<SerialResponse<T>>(self.0).await
    }

    pub(crate) const fn new(response: ReqwestResponse) -> Self {
        Self(response)
    }
}

/// An [`InfoResponse`] body parser.
pub struct InfoResponseParser(ReqwestResponse);

impl InfoResponseParser {
    /// Parses the internal response body to retrieve an [`InfoResponse`].
    ///
    /// # Errors
    ///
    /// If the response body does not contain a valid [`InfoResponse`], a
    /// parsing error will be raised. This may occur due to an incorrect format
    /// or because the binary data contains syntactic or semantic errors.
    pub async fn parse_body(self) -> Result<InfoResponse> {
        json_response::<InfoResponse>(self.0).await
    }

    pub(crate) const fn new(response: ReqwestResponse) -> Self {
        Self(response)
    }
}

/// A byte stream response body parser.
#[cfg(feature = "stream")]
pub struct StreamResponse(ReqwestResponse);

#[cfg(feature = "stream")]
impl StreamResponse {
    /// Opens a bytes stream from the response received from a device.
    ///
    /// # Errors
    ///
    /// Byte stream parsing may fail due to network errors or data corruption.
    pub fn open_stream(self) -> impl futures_util::Stream<Item = Result<bytes::Bytes>> {
        use futures_util::TryStreamExt;
        self.0.bytes_stream().map_err(|e| {
            Error::new(
                ErrorKind::StreamResponse,
                format!("Stream error caused by {e}"),
            )
        })
    }

    pub(crate) const fn new(response: ReqwestResponse) -> Self {
        Self(response)
    }
}

/// All response types supported by a `tosca` device.
///
/// Each response includes a dedicated body parser to extract the embedded data.
pub enum Response {
    /// A skipped response indicates a request that is not sent due to
    /// privacy policy rules.
    Skipped,
    /// An [`OkResponse`] body.
    OkBody(OkResponseParser),
    /// A [`SerialResponse`] body.
    SerialBody(SerialResponseParser),
    /// An [`InfoResponse`] body.
    InfoBody(InfoResponseParser),
    /// A byte stream response body.
    #[cfg(feature = "stream")]
    StreamBody(StreamResponse),
}
