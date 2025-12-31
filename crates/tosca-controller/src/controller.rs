use std::borrow::Cow;

use tosca::parameters::ParametersValues;

use tokio::sync::mpsc::{self, Receiver};

use tracing::{error, warn};

use crate::device::{Device, Devices};
use crate::discovery::Discovery;
use crate::error::{Error, ErrorKind};
use crate::events::{EventPayload, EventsRunner};
use crate::policy::Policy;
use crate::request::Request;
use crate::response::Response;

// TODO: Use the MAC address as id.

fn sender_error(error: impl Into<Cow<'static, str>>) -> Error {
    Error::new(ErrorKind::Sender, error)
}

/// A request sender.
#[derive(Debug, PartialEq)]
pub struct RequestSender<'controller> {
    controller: &'controller Controller,
    request: &'controller Request,
    skip: bool,
}

impl RequestSender<'_> {
    /// Sends a request to a device and returns a [`Response`].
    ///
    /// # Errors
    ///
    /// Network failures or timeouts may prevent the request from being sent
    /// and affect the returned response as well.
    pub async fn send(&self) -> Result<Response, Error> {
        self.request
            .retrieve_response(self.skip, || async { self.request.plain_send().await })
            .await
    }

    /// Sends a request to a device with the given [`ParametersValues`]
    /// and returns a [`Response`].
    ///
    /// # Errors
    ///
    /// Network failures or timeouts may prevent the request from being sent
    /// and affect the returned response as well.
    pub async fn send_with_parameters(
        &self,
        parameters: &ParametersValues<'_>,
    ) -> Result<Response, Error> {
        if self.request.parameters_data.is_empty() {
            warn!("The request does not have input parameters.");
            return self.send().await;
        }

        self.request
            .retrieve_response(self.skip, || async {
                self.request.create_response(parameters).await
            })
            .await
    }
}

/// A device sender.
///
/// It generates multiple unique request senders for a device.
#[derive(Debug, PartialEq)]
pub struct DeviceSender<'controller> {
    controller: &'controller Controller,
    device: &'controller Device,
    id: usize,
}

impl DeviceSender<'_> {
    /// Builds a [`RequestSender`] for the given route.
    ///
    /// The generated request sender is tightly bound to the device sender and
    /// cannot function independently.
    ///
    /// # Errors
    ///
    /// An error is returned if the given route **does** not exist.
    pub fn request(&self, route: &str) -> Result<RequestSender<'_>, Error> {
        let request = self.device.request(route).ok_or_else(|| {
            sender_error(format!(
                "Error in retrieving the request with route `{route}`."
            ))
        })?;

        let skip = if request.hazards.is_empty() {
            false
        } else {
            self.evaluate_privacy_policy(request, route)
        };

        Ok(RequestSender {
            controller: self.controller,
            request,
            skip,
        })
    }

    fn evaluate_privacy_policy(&self, request: &Request, route: &str) -> bool {
        let mut skip = false;

        let global_blocked_hazards = self
            .controller
            .privacy_policy
            .global_blocked_hazards(&request.hazards);

        let local_blocked_hazards = self
            .controller
            .privacy_policy
            .local_blocked_hazards(self.id, &request.hazards);

        if !global_blocked_hazards.is_empty() {
            warn!(
                "The {route} is skipped because it contains the global blocked hazards: {:?}",
                global_blocked_hazards
            );
            skip = true;
        }

        if !local_blocked_hazards.is_empty() {
            warn!(
                "The {route} is skipped because the device contains the local blocked hazards: {:?}",
                local_blocked_hazards
            );
            skip = true;
        }

        skip
    }
}

/// A controller for interacting with `tosca` devices.
///
/// The main functionalities include:
///
/// - Discovering `tosca` devices on the network and registering them in memory.
/// - Sending requests to a specific device identified by its ID, awaiting a
///   response, and forwarding it directly to the caller.
/// - Controlling request sending by allowing or blocking requests based on the
///   defined privacy policy.
#[derive(Debug, PartialEq)]
pub struct Controller {
    discovery: Discovery,
    devices: Devices,
    privacy_policy: Policy,
}

impl Controller {
    /// Creates a [`Controller`] from a [`Discovery`] configuration.
    #[must_use]
    #[inline]
    pub fn new(discovery: Discovery) -> Self {
        Self {
            discovery,
            devices: Devices::new(),
            privacy_policy: Policy::init(),
        }
    }

    /// Creates a [`Controller`] from a [`Discovery`] configuration and
    /// an initial set of [`Devices`].
    ///
    /// This method is useful when [`Devices`] are retrieved from database.
    #[must_use]
    #[inline]
    pub fn from_devices(discovery: Discovery, devices: Devices) -> Self {
        Self {
            discovery,
            devices,
            privacy_policy: Policy::init(),
        }
    }

    /// Defines a [`Policy`] while constructing a [`Controller`].
    #[must_use]
    #[inline]
    pub fn policy(mut self, privacy_policy: Policy) -> Self {
        self.privacy_policy = privacy_policy;
        self
    }

    /// Changes the [`Policy`].
    #[inline]
    pub fn change_policy(&mut self, privacy_policy: Policy) {
        self.privacy_policy = privacy_policy;
    }

    /// Discovers all available [`Devices`] on the network.
    ///
    /// # Errors
    ///
    /// ## Discovery Errors
    ///
    /// During the discovery process, common errors include:
    ///
    /// - Inability to connect to the network
    /// - Failure to disable a particular network interface
    /// - Issues terminating the discovery process.
    ///
    /// ## Sending Requests Errors
    ///
    /// When sending a request to a device to retrieve its structure description
    /// and routes, network failures or timeouts may prevent the request from
    /// being sent and affect the returned response as well.
    #[inline]
    pub async fn discover(&mut self) -> Result<(), Error> {
        self.devices = self.discovery.discover().await?;
        Ok(())
    }

    /// Starts asynchronous event receiver tasks for all [`Device`]s that
    /// support events.
    ///
    /// An event receiver task connects to the broker of a device
    /// and subscribes to its topic.
    /// When a device transmits an event to the broker, the task retrieves the
    /// event payload from the broker, parses the data, and sends the relevant
    /// content to the [`Receiver`] returned by this method.
    ///
    /// The `buffer_size` parameter specifies how many messages the event
    /// receiver buffer can hold.
    /// When the buffer is full, subsequent send attempts will wait until
    /// a message is consumed from the channel.
    ///
    /// When the [`Receiver`] is dropped, all tasks terminate automatically.
    ///
    /// # Errors
    ///
    /// - No event receiver tasks has started
    /// - An error occurred while subscribing to the broker topic of a device.
    pub async fn start_event_receivers(
        &mut self,
        buffer_size: usize,
    ) -> Result<Receiver<EventPayload>, Error> {
        let (tx, rx) = mpsc::channel(buffer_size);

        let mut started_count = 0;
        for (id, device) in self.devices.iter_mut().enumerate() {
            if device.event_handle.is_some() {
                warn!("Skip device with id `{id}`: event receiver already started");
                continue;
            }

            let Some(ref events) = device.events else {
                warn!("Skip device with id `{id}`: it does not support events");
                continue;
            };

            EventsRunner::run_global_subscriber(events, id, tx.clone()).await?;

            started_count += 1;
        }

        if started_count == 0 {
            return Err(Error::new(
                ErrorKind::Events,
                "No event receiver tasks has started",
            ));
        }

        Ok(rx)
    }

    /// Returns an immutable reference to [`Devices`].
    #[must_use]
    pub const fn devices(&self) -> &Devices {
        &self.devices
    }

    /// Returns a mutable reference to [`Devices`].
    #[must_use]
    pub const fn devices_mut(&mut self) -> &mut Devices {
        &mut self.devices
    }

    /// Builds a [`DeviceSender`] for the [`Device`] with the given identifier.
    ///
    /// # Errors
    ///
    /// An error is returned if no devices are found or if the given index
    /// **does** not exist.
    pub fn device(&self, id: usize) -> Result<DeviceSender<'_>, Error> {
        if self.devices.is_empty() {
            return Err(sender_error("No devices found."));
        }

        let device = self.devices.get(id).ok_or_else(|| {
            sender_error(format!(
                "Error in retrieving the device with identifier {id}."
            ))
        })?;
        Ok(DeviceSender {
            controller: self,
            device,
            id,
        })
    }

    /// Shuts down the [`Controller`], stopping all asynchronous tasks and
    /// releasing all associated resources.
    ///
    /// # Note
    ///
    /// For a graceful shutdown, this method must be called before dropping
    /// the [`Controller`].
    pub async fn shutdown(self) {
        // Stop all events tasks.
        for device in self.devices {
            if let Some(events) = device.events {
                // Stop the infinite loop
                events.cancellation_token.cancel();
            }

            if let Some(event_handle) = device.event_handle {
                // Await the task.
                if let Err(e) = event_handle.await {
                    error!("Failed to await the event task: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use tracing::warn;

    use tosca::hazards::{Hazard, Hazards};
    use tosca::parameters::ParametersValues;
    use tosca::response::{OkResponse, SerialResponse};

    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::json;

    use serial_test::serial;

    use crate::device::Devices;
    use crate::error::Error;
    use crate::policy::Policy;
    use crate::response::Response;

    use crate::device::tests::{create_light, create_unknown};
    use crate::discovery::tests::configure_discovery;
    use crate::tests::{Brightness, check_function_with_device};

    use super::{Controller, DeviceSender, RequestSender, sender_error};

    #[test]
    fn empty_controller() {
        let controller = Controller::new(configure_discovery());

        assert_eq!(
            controller,
            Controller {
                discovery: configure_discovery(),
                devices: Devices::new(),
                privacy_policy: Policy::init(),
            }
        );

        // No devices.
        assert_eq!(controller.device(0), Err(sender_error("No devices found.")));
    }

    #[test]
    fn controller_from_devices() {
        let devices = Devices::from_devices(vec![create_light(), create_unknown()]);

        let controller = Controller::from_devices(configure_discovery(), devices);

        assert_eq!(
            controller,
            Controller {
                discovery: configure_discovery(),
                devices: Devices::from_devices(vec![create_light(), create_unknown()]),
                privacy_policy: Policy::init(),
            }
        );
    }

    async fn check_ok_response_plain(device_sender: &DeviceSender<'_>, route: &str) {
        check_ok_response(device_sender, route, async move |request_sender| {
            request_sender.send().await
        })
        .await;
    }

    async fn check_ok_response_with_parameters(
        device_sender: &DeviceSender<'_>,
        route: &str,
        parameters: &ParametersValues<'_>,
    ) {
        check_ok_response(device_sender, route, async move |request_sender| {
            request_sender.send_with_parameters(parameters).await
        })
        .await;
    }

    async fn check_ok_response<'controller, 'a, F>(
        device_sender: &'a DeviceSender<'controller>,
        route: &'a str,
        get_response: F,
    ) where
        F: AsyncFnOnce(RequestSender<'controller>) -> Result<Response, Error>,
        'a: 'controller,
    {
        let request = device_sender.request(route).unwrap();

        let response = get_response(request).await.unwrap();
        if let Response::OkBody(response) = response {
            let ok_response = response.parse_body().await.unwrap();
            assert_eq!(ok_response, OkResponse::ok());
        } else {
            assert!(
                matches!(response, Response::Skipped),
                "Should be a blocked global `LogEnergyConsumption` for `/off` request"
            );
        }
    }

    async fn check_serial_response_plain<T: Serialize + DeserializeOwned + Debug + PartialEq>(
        device_sender: &DeviceSender<'_>,
        route: &str,
        value: T,
    ) {
        check_serial_response(
            device_sender,
            route,
            async move |request_sender| request_sender.send().await,
            value,
        )
        .await;
    }

    async fn check_serial_response_with_parameters<
        T: Serialize + DeserializeOwned + Debug + PartialEq,
    >(
        device_sender: &DeviceSender<'_>,
        route: &str,
        parameters: &ParametersValues<'_>,
        value: T,
    ) {
        check_serial_response(
            device_sender,
            route,
            async move |request| request.send_with_parameters(parameters).await,
            value,
        )
        .await;
    }

    async fn check_serial_response<'controller, 'a, F, T>(
        device: &'a DeviceSender<'controller>,
        route: &'a str,
        get_response: F,
        value: T,
    ) where
        F: AsyncFnOnce(RequestSender<'controller>) -> Result<Response, Error>,
        T: Serialize + DeserializeOwned + Debug + PartialEq,
        'a: 'controller,
    {
        let request = device.request(route).unwrap();

        let response = get_response(request).await.unwrap();
        if let Response::SerialBody(response) = response {
            let serial_response = response.parse_body::<T>().await.unwrap();
            assert_eq!(serial_response, SerialResponse::new(value));
        } else {
            assert!(
                matches!(response, Response::Skipped),
                "Should be a blocked local `FireHazard` for `/toggle` request"
            );
        }
    }

    async fn controller_checks(controller: Controller) {
        // Wrong device id.
        assert_eq!(
            controller.device(1),
            Err(sender_error(
                "Error in retrieving the device with identifier 1."
            ))
        );

        // Get device.
        let device_sender = controller.device(0).unwrap();

        // Wrong request.
        assert_eq!(
            device_sender.request("/wrong"),
            Err(sender_error(
                "Error in retrieving the request with route `/wrong`."
            ))
        );

        // Run "/on" request and get "Ok" response.
        check_ok_response_plain(&device_sender, "/on").await;

        // Run "/off" request and get "Ok" response.
        check_ok_response_plain(&device_sender, "/off").await;

        // Run "/toggle" request and get "Ok" response.
        check_serial_response_plain(
            &device_sender,
            "/toggle",
            json!({
                "brightness": 0,
            }),
        )
        .await;

        // With parameters
        let mut parameters = ParametersValues::new();
        parameters.u64("brightness", 5);

        // Run "/on" request and get an "Ok" response with parameters.
        check_ok_response_with_parameters(&device_sender, "/on", &parameters).await;

        // Run "/off" request and get an "Ok" response with parameters.
        check_ok_response_with_parameters(&device_sender, "/off", &parameters).await;

        // Run "/toggle" request and get an "Ok" response with parameters.
        check_serial_response_with_parameters(
            &device_sender,
            "/toggle",
            &parameters,
            Brightness { brightness: 5 },
        )
        .await;
    }

    #[inline]
    async fn controller_without_policy() {
        // Create a controller.
        let mut controller = Controller::new(configure_discovery());

        // Run discovery process.
        controller.discover().await.unwrap();

        // Run controller checks.
        controller_checks(controller).await;
    }

    #[inline]
    async fn controller_with_policy() {
        // Global blocked hazards.
        let global_hazards = Hazards::new().insert(Hazard::LogEnergyConsumption);

        // Local blocked hazards for a specific device.
        let local_hazards = Hazards::new().insert(Hazard::FireHazard);

        // Create both a global policy and a local one.
        let policy = Policy::new(global_hazards).block_device_on_hazards(0, local_hazards);

        // Create a controller.
        let mut controller = Controller::new(configure_discovery()).policy(policy);

        // Run discovery process.
        controller.discover().await.unwrap();

        // Run controller checks.
        controller_checks(controller).await;
    }

    #[inline]
    async fn run_controller_function<F, Fut>(name: &str, function: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        if option_env!("CI").is_some() {
            warn!(
                "Skipping test on CI: {} can run only on systems that expose physical MAC addresses.",
                name
            );
        } else {
            check_function_with_device(|| async {
                function().await;
            })
            .await;
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial]
    async fn test_without_policy_controller() {
        run_controller_function("controller_without_policy", || async {
            controller_without_policy().await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial]
    async fn test_with_policy_controller() {
        run_controller_function("controller_with_policy", || async {
            controller_with_policy().await;
        })
        .await;
    }
}
