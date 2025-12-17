use core::fmt::{Debug, Display};
use core::net::SocketAddr;
use core::pin::Pin;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use alloc::str::SplitTerminator;
use alloc::string::ToString;
use alloc::vec::Vec;

use tosca::parameters::{
    ParameterKind, ParameterPayload, ParameterValue, ParametersPayloads as ToscaParametersPayloads,
    ParametersValues,
};
use tosca::route::{RestKind, RouteConfig};

use edge_http::io::server::{Connection, Handler, Server as EdgeServer};
use edge_http::io::Body;
use edge_http::{Headers, Method};
use edge_nal::{TcpBind, WithTimeout};
use edge_nal_embassy::{Tcp, TcpBuffers};

use embassy_executor::Spawner;
use embassy_net::Stack;

use embedded_io_async::{Read, Write};

use log::{error, info};

use crate::device::{Device, InternalDevice};
use crate::error::Error;
use crate::mdns::Mdns;
use crate::net::get_ip;
use crate::parameters::ParametersPayloads;
use crate::response::{ErrorResponse, InfoResponse, OkResponse, Response, SerialResponse};
use crate::state::{State, ValueFromRef};

// Default port.
const DEFAULT_SERVER_PORT: u16 = 80;

// The maximum number of clients the HTTP server can support simultaneously.
//
// Referring to the official ESP example at:
// https://github.com/esp-rs/esp-mbedtls/blob/main/examples/edge_server.rs#L56-L57,
// the server is configured to handle a maximum of 2 simultaneous
// open connections (sockets).
const NUMBER_OF_CLIENTS: usize = 2;

// Maximum request size in bytes.
const MAXIMUM_REQUEST_SIZE: usize = 128;

pub(crate) type OkFn = Box<
    dyn Fn(
            ParametersPayloads,
        ) -> Pin<
            Box<dyn Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type OkStateFn<S> = Box<
    dyn Fn(
            State<S>,
            ParametersPayloads,
        ) -> Pin<
            Box<dyn Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type SerialFn = Box<
    dyn Fn(
            ParametersPayloads,
        ) -> Pin<
            Box<dyn Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type SerialStateFn<S> = Box<
    dyn Fn(
            State<S>,
            ParametersPayloads,
        ) -> Pin<
            Box<dyn Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type InfoFn = Box<
    dyn Fn(
            ParametersPayloads,
        ) -> Pin<
            Box<dyn Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type InfoStateFn<S> = Box<
    dyn Fn(
            State<S>,
            ParametersPayloads,
        ) -> Pin<
            Box<dyn Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type Functions<S> = (
    Vec<OkFn>,
    Vec<OkStateFn<S>>,
    Vec<SerialFn>,
    Vec<SerialStateFn<S>>,
    Vec<InfoFn>,
    Vec<InfoStateFn<S>>,
);

#[derive(Clone, Copy)]
pub(crate) enum FuncType {
    OkStateless,
    OkStateful,
    SerialStateless,
    SerialStateful,
    InfoStateless,
    InfoStateful,
}

#[derive(Clone, Copy)]
pub(crate) struct FuncIndex {
    func_type: FuncType,
    index: usize,
}

impl FuncIndex {
    pub(crate) const fn new(func_type: FuncType, index: usize) -> Self {
        Self { func_type, index }
    }
}

fn with_timeout<T>(timeout_ms: u32, io: T) -> WithTimeout<T> {
    WithTimeout::new(timeout_ms, io)
}

/// The `tosca` server.
///
/// ## Parameters
///
/// - **`port`**
///   The TCP port on which the server listens for incoming connections.
///   Defaults to `80`.
///   See [`Server::port()`] to configure this.
///
/// - **`keepalive_timeout_ms`**
///   Optional timeout (in milliseconds) for detecting an idle persistent
///   HTTP keep-alive connection, where multiple requests can be sent over
///   the same TCP connection without reopening it.
///   The default value is `None`, meaning that idle connections are never
///   closed due to inactivity; if no other timeouts are set, they remain
///   open indefinitely.
///   See [`Server::keepalive_timeout()`] to configure this.
///
/// - **`io_timeout_ms`**
///   Optional timeout (in milliseconds) for socket I/O operations.
///   The default value is `None`, meaning that read and write operations
///   never time out.
///   See [`Server::io_timeout()`].
///
/// - **`handler_timeout_ms`**
///   Optional timeout (in milliseconds) for handler execution.
///   The default value is `None`, meaning that request handlers are not
///   interrupted by timeouts.
///   See [`Server::handler_timeout()`].
///
/// ## Known Issue
///
/// In `edge-net`
/// ([issue #62](https://github.com/sysgrok/edge-net/issues/62)),
/// connection reuse may cause some clients — notably `curl` —
/// to fail when sending multiple requests over a single
/// keep-alive session using `keepalive_timeout_ms`.
/// To ensure correct sequential request handling with `curl`,
/// include `Connection: close` in the request headers
/// until the issue is resolved.
pub struct Server<const TX_SIZE: usize, const RX_SIZE: usize, const MAXIMUM_HEADERS_COUNT: usize, S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    // Server port.
    port: u16,
    // HTTP handler.
    handler: ServerHandler<S>,
    // mDNS
    mdns: Mdns,
    // Keepalive timeout.
    keepalive_timeout_ms: Option<u32>,
    // Socket I/O operations timeout.
    io_timeout_ms: Option<u32>,
    // Handler timeout.
    handler_timeout_ms: Option<u32>,
    // Https scheme.
    is_https: bool,
}

impl<const TX_SIZE: usize, const RX_SIZE: usize, const MAXIMUM_HEADERS_COUNT: usize, S>
    Server<TX_SIZE, RX_SIZE, MAXIMUM_HEADERS_COUNT, S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a [`Server`].
    #[inline]
    pub fn new(device: Device<S>, mdns: Mdns) -> Self {
        Self {
            port: DEFAULT_SERVER_PORT,
            handler: ServerHandler::new(device.into_internal()),
            mdns,
            keepalive_timeout_ms: None,
            io_timeout_ms: None,
            handler_timeout_ms: None,
            is_https: false,
        }
    }

    /// Sets the port number for the server to listen on.
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the timeout (in milliseconds) for persistent HTTP keep-alive connections.
    #[must_use]
    pub const fn keepalive_timeout(mut self, timeout_ms: u32) -> Self {
        self.keepalive_timeout_ms = Some(timeout_ms);
        self
    }

    /// Sets the timeout (in milliseconds) for socket I/O operations.
    #[must_use]
    pub const fn io_timeout(mut self, timeout_ms: u32) -> Self {
        self.io_timeout_ms = Some(timeout_ms);
        self
    }

    /// Sets the timeout (in milliseconds) for handler execution.
    #[must_use]
    pub const fn handler_timeout(mut self, timeout_ms: u32) -> Self {
        self.handler_timeout_ms = Some(timeout_ms);
        self
    }

    /// Sets the scheme to `HTTPS`.
    #[must_use]
    pub const fn https(mut self) -> Self {
        self.is_https = true;
        self
    }

    /// Runs the [`Server`] and the [`Mdns`] task.
    ///
    /// # Errors
    ///
    /// - Failure to bind TCP protocol buffers to the underlying socket
    /// - Failure to spawn the `mDNS` task
    /// - Failure to run the server
    pub async fn run(self, stack: Stack<'static>, spawner: Spawner) -> Result<(), Error> {
        let Server {
            port,
            handler,
            mdns,
            keepalive_timeout_ms,
            io_timeout_ms,
            handler_timeout_ms,
            is_https,
        } = self;

        let buffers = TcpBuffers::<NUMBER_OF_CLIENTS, TX_SIZE, RX_SIZE>::new();
        let tcp = Tcp::new(stack, &buffers);

        let address = get_ip(stack).await;
        let socket = SocketAddr::new(address.into(), port);

        let acceptor = tcp.bind(socket).await?;

        let mdns = if is_https {
            mdns.properties(&[("scheme", "https")])
        } else {
            mdns
        };

        // Run mdns.
        //
        // NOTE: Use the same server port for the mDNS-SD service
        mdns.run(stack, address, port, spawner)?;

        info!("Starting server on address `{address}` and port `{port}`");

        match (io_timeout_ms, handler_timeout_ms) {
            (Some(ta), Some(th)) => {
                Self::run_server(
                    keepalive_timeout_ms,
                    with_timeout(ta, acceptor),
                    with_timeout(th, handler),
                )
                .await
            }
            (Some(ta), None) => {
                Self::run_server(keepalive_timeout_ms, with_timeout(ta, acceptor), handler).await
            }
            (None, Some(th)) => {
                Self::run_server(keepalive_timeout_ms, acceptor, with_timeout(th, handler)).await
            }
            (None, None) => Self::run_server(keepalive_timeout_ms, acceptor, handler).await,
        }
    }

    async fn run_server<A, H>(
        keepalive_timeout_ms: Option<u32>,
        acceptor: A,
        handler: H,
    ) -> Result<(), Error>
    where
        A: edge_nal::TcpAccept,
        H: Handler,
        Error: From<A::Error>,
    {
        let mut server = EdgeServer::<NUMBER_OF_CLIENTS, RX_SIZE, MAXIMUM_HEADERS_COUNT>::new();

        // Run server.
        server
            .run(keepalive_timeout_ms, acceptor, handler)
            .await
            .map_err(core::convert::Into::into)
    }
}

const fn method_map(ascot_method: RestKind) -> Method {
    match ascot_method {
        RestKind::Get => Method::Get,
        RestKind::Put => Method::Put,
        RestKind::Post => Method::Post,
        RestKind::Delete => Method::Delete,
    }
}

#[inline]
fn error_response_with_error(description: &str, error: &str) -> Response {
    error!("{description}: {error}");
    ErrorResponse::internal_with_error(description, error).0
}

#[inline]
fn error_response(description: &str) -> Response {
    error!("{description}");
    ErrorResponse::internal(description).0
}

#[inline]
fn invalid_data_response(description: &str) -> Response {
    invalid_data(description).0
}

#[inline]
pub(crate) fn invalid_data(description: &str) -> ErrorResponse {
    error!("{description}");
    ErrorResponse::invalid_data(description)
}

struct RouteInfo {
    index: usize,
    parameters_payloads: ParametersPayloads,
}

impl RouteInfo {
    const fn new(index: usize, parameters_payloads: ToscaParametersPayloads<'static>) -> Self {
        Self {
            index,
            parameters_payloads: ParametersPayloads(parameters_payloads),
        }
    }
}

struct ServerHandler<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    device: InternalDevice<S>,
}

impl<S> ServerHandler<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    #[inline]
    fn new(device: InternalDevice<S>) -> Self {
        Self { device }
    }

    async fn analyze_route<const N: usize, T: Read>(
        &self,
        method: Method,
        path: &str,
        headers: &Headers<'_, N>,
        body: &mut Body<'_, T>,
    ) -> Result<RouteInfo, Response> {
        // If the last character of a path ends with '/', remove it.
        let path = path.strip_suffix('/').unwrap_or(path);

        info!("Complete path: {path}");

        // Create an iterator to parse the route in an iterative way. This
        // function removes the trailing '/' which might be present in route
        // definition.
        let mut route_iter = path.split_terminator('/');

        // Every time a `nth(0)` function is being called, the iterator
        // consumes the current element.
        //
        // The first 0-element of a route is always an empty path
        // because **each** path begins with a '/'.
        //
        // In case of error, return a not found route.
        let empty_path = route_iter.nth(0).ok_or_else(Response::not_found)?;

        // If the empty path is equal to the route path,
        // the route is not correct. This might happen when a route
        // path makes use of a wrong separator, both for route subpaths, but
        // also for its parameters.
        //
        // In case of error, return a not found route.
        if empty_path == path {
            return Err(Response::not_found());
        }

        // Retrieve the main route.
        let main_route_path = route_iter.nth(0).ok_or_else(Response::not_found)?;

        // If the subpath is not equal to the main route,
        // the route is not correct. Starts from the 1-index
        // in order to skip the "/" placed before the main route.
        if main_route_path != &self.device.main_route[1..] {
            return Err(Response::not_found());
        }

        let mut route_index = self.device.route_configs.len();
        for (index, route) in self.device.route_configs.iter().enumerate() {
            // If the request REST method is different from the route
            // method, skip to the next route.
            if method != method_map(route.rest_kind) {
                continue;
            }

            let route_path = &route.data.path[1..];

            // Create iterators for both the route and request paths
            // to compare each segment.
            let mut route_path_iter = route_path.split_terminator('/');
            let mut path_iter = path.split_terminator('/');

            // Skip the empty segment "" and the main route.
            let _ = path_iter.next();
            let _ = path_iter.next();

            // Compare route segments with the corresponding path segments.
            // If all segments match in order, this is the correct route.
            if !route_path_iter.all(|seg| path_iter.next() == Some(seg)) {
                continue;
            }

            info!("Route path: {route_path}");

            for _ in 0..route_path.split_terminator('/').count() {
                route_iter.nth(0).ok_or_else(Response::not_found)?;
            }

            // If the route has no parameters, return its index.
            //
            // Otherwise, save the index and break the loop,
            // as there are parameters to analyze.
            if route.data.parameters.is_empty() {
                return Ok(RouteInfo::new(index, ToscaParametersPayloads::new()));
            }

            route_index = index;
            break;
        }

        // Retrieve the route configuration.
        let route_config = self
            .device
            .route_configs
            .get_index(route_index)
            .ok_or_else(Response::not_found)?;

        match method {
            Method::Get => Self::parse_get_parameters(route_config, route_iter),
            // NOTE: We include the disallowed methods here as well, since
            // the check has already been performed earlier.
            _ => Self::parse_headers_parameters(route_config, headers, body).await,
        }
        .map(|parameters_payloads| RouteInfo::new(route_index, parameters_payloads))
    }

    #[inline]
    fn parse_get_parameters(
        route_config: &RouteConfig,
        mut route_iter: SplitTerminator<'_, char>,
    ) -> Result<ToscaParametersPayloads<'static>, Response> {
        // Create parameters payloads.
        let mut parameters_payloads = ToscaParametersPayloads::new();

        for (index, parameter) in route_config.data.parameters.iter().enumerate() {
            let parameter_value = route_iter.nth(0).ok_or_else(|| {
                invalid_data_response(&format!(
                    "Passed route path is too short, missing parameters: {:?}",
                    route_config
                        .data
                        .parameters
                        .iter()
                        .skip(index)
                        .map(|parameter| parameter.0.as_str())
                        .collect::<Vec<&str>>()
                ))
            })?;

            info!("Parameter value as string: {parameter_value}");
            let parameter_value = Self::parse_parameter_value(parameter_value, parameter.1)?;

            parameters_payloads.add(
                parameter.0.clone().into(),
                ParameterPayload::new(parameter.1.clone(), parameter_value),
            );
        }

        // NOTE: We do not check whether a route path still contains other ,
        // as this is unnecessary since all parameters have been taken.
        Ok(parameters_payloads)
    }

    #[inline]
    async fn parse_headers_parameters<const N: usize, T: Read>(
        route_config: &RouteConfig,
        headers: &Headers<'_, N>,
        body: &mut Body<'_, T>,
    ) -> Result<ToscaParametersPayloads<'static>, Response> {
        info!("Headers: {headers:?}");

        let content_length = headers
            .get("Content-Length")
            .ok_or_else(|| invalid_data_response("No `Content-Length` found"))?;

        let content_length = content_length.parse::<usize>().map_err(|e| {
            error_response_with_error(
                "Unable to convert the `Content-Length` header into a number",
                &format!("{e}"),
            )
        })?;

        if content_length > MAXIMUM_REQUEST_SIZE {
            return Err(error_response(&format!(
                "The request exceeds the maximum allowed size of {MAXIMUM_REQUEST_SIZE} and cannot be processed"
            )));
        }

        let content_type = headers
            .content_type()
            .ok_or_else(|| invalid_data_response("No `Content-Type` found"))?;

        if content_type != "application/json" {
            return Err(invalid_data_response(
                "The request body does not have a JSON format as content type",
            ));
        }

        let mut bytes = [0; MAXIMUM_REQUEST_SIZE];
        body.read(&mut bytes).await.map_err(|e| {
            error_response_with_error("Error reading the request bytes", &format!("{e:?}"))
        })?;

        let route_parameters =
            serde_json::from_slice::<ParametersValues>(&bytes[0..content_length]).map_err(|e| {
                error_response_with_error(
                    "Failed to convert bytes into a sequence of parameters",
                    &format!("{e}"),
                )
            })?;

        info!("Route parameters: {route_parameters:?}");

        let mut parameters_payloads = ToscaParametersPayloads::new();
        for (parameter_name, parameter_value) in route_parameters {
            let parameter_kind = route_config
                .data
                .parameters
                .get(&parameter_name)
                .ok_or_else(|| {
                    invalid_data_response(&format!("Parameter `{parameter_name}` not found"))
                })?;

            if !parameter_value.match_kind(parameter_kind) {
                return Err(invalid_data_response(&format!(
                    "Found type `{}` for `{parameter_name}`, expected type `{}`",
                    parameter_value.as_type(),
                    parameter_kind.as_type(),
                )));
            }

            parameters_payloads.add(
                parameter_name,
                ParameterPayload::new(parameter_kind.clone(), parameter_value),
            );
        }

        Ok(parameters_payloads)
    }

    fn parse_parameter_value(
        parameter_value: &str,
        parameter_kind: &ParameterKind,
    ) -> Result<ParameterValue, Response> {
        match parameter_kind {
            ParameterKind::Bool { .. } => {
                Self::into_value::<bool, _>(parameter_value, "bool", ParameterValue::Bool)
            }
            ParameterKind::U8 { .. } => {
                Self::into_value::<u8, _>(parameter_value, "u8", ParameterValue::U8)
            }
            ParameterKind::U16 { .. } => {
                Self::into_value::<u16, _>(parameter_value, "u16", ParameterValue::U16)
            }
            ParameterKind::U32 { .. } | ParameterKind::RangeU32 { .. } => {
                Self::into_value::<u32, _>(parameter_value, "u32", ParameterValue::U32)
            }
            ParameterKind::U64 { .. } | ParameterKind::RangeU64 { .. } => {
                Self::into_value::<u64, _>(parameter_value, "u64", ParameterValue::U64)
            }
            ParameterKind::F32 { .. } => {
                Self::into_value::<f32, _>(parameter_value, "f32", ParameterValue::F32)
            }
            ParameterKind::F64 { .. } | ParameterKind::RangeF64 { .. } => {
                Self::into_value::<f64, _>(parameter_value, "f64", ParameterValue::F64)
            }
            ParameterKind::CharsSequence { .. } => Ok(ParameterValue::CharsSequence(Cow::Owned(
                parameter_value.to_string(),
            ))),
        }
    }

    #[inline]
    fn into_value<T, F>(
        parameter_value: &str,
        type_msg: &str,
        parameter_value_generator: F,
    ) -> Result<ParameterValue, Response>
    where
        T: core::str::FromStr,
        <T as core::str::FromStr>::Err: Display,
        F: FnOnce(T) -> ParameterValue,
    {
        parameter_value
            .parse::<T>()
            .map(parameter_value_generator)
            .map_err(|e| {
                error_response_with_error(
                    &format!("Failed to parse `{parameter_value}` into `{type_msg}` type"),
                    &format!("{e}"),
                )
            })
    }

    #[inline]
    async fn run_function(
        &self,
        index: usize,
        parameters_payloads: ParametersPayloads,
    ) -> Response {
        let func_index = self.device.index_array[index];

        match func_index.func_type {
            FuncType::OkStateless => {
                let func = &self.device.routes_functions.0[func_index.index];
                func(parameters_payloads).await.into()
            }
            FuncType::OkStateful => {
                let func = &self.device.routes_functions.1[func_index.index];
                func(
                    State(S::value_from_ref(&self.device.state.0)),
                    parameters_payloads,
                )
                .await
                .into()
            }
            FuncType::SerialStateless => {
                let func = &self.device.routes_functions.2[func_index.index];
                func(parameters_payloads).await.into()
            }
            FuncType::SerialStateful => {
                let func = &self.device.routes_functions.3[func_index.index];
                func(
                    State(S::value_from_ref(&self.device.state.0)),
                    parameters_payloads,
                )
                .await
                .into()
            }
            FuncType::InfoStateless => {
                let func = &self.device.routes_functions.4[func_index.index];
                func(parameters_payloads).await.into()
            }
            FuncType::InfoStateful => {
                let func = &self.device.routes_functions.5[func_index.index];
                func(
                    State(S::value_from_ref(&self.device.state.0)),
                    parameters_payloads,
                )
                .await
                .into()
            }
        }
    }

    const fn is_method_allowed(method: Method) -> bool {
        !matches!(
            method,
            Method::Get | Method::Post | Method::Put | Method::Delete
        )
    }
}

impl<S: ValueFromRef + Send + Sync + 'static> Handler for ServerHandler<S> {
    type Error<E>
        = edge_http::io::Error<E>
    where
        E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl Display + Copy,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let (headers, body) = conn.split();

        if headers.path == "/" {
            return self.device.main_route_response.write_from_ref(conn).await;
        }

        if Self::is_method_allowed(headers.method) {
            return Response::not_allowed().write(conn).await;
        }

        let route_info = match self
            .analyze_route(headers.method, headers.path, &headers.headers, body)
            .await
        {
            Ok(index) => index,
            Err(response) => return response.write(conn).await,
        };

        let RouteInfo {
            index,
            parameters_payloads,
        } = route_info;

        let response = self.run_function(index, parameters_payloads).await;
        response.write(conn).await
    }
}
