use std::io::Cursor;

use tosca_os::extract::{Json, State, header};
use tosca_os::responses::error::ErrorResponse;
use tosca_os::responses::stream::StreamResponse;

use image::ImageFormat;

use nokhwa::{
    Camera,
    pixel_format::{RgbAFormat, RgbFormat},
    utils::{RequestedFormat, RequestedFormatType, Resolution},
};

use tokio::task::spawn_blocking;

use tracing::info;

use crate::parameters::{CameraFramerate, CameraInputs, CameraResolution};
use crate::{InternalState, camera_error, camera_format};

async fn run_camera_screenshot(
    state: InternalState,
    format: RequestedFormat<'static>,
) -> Result<StreamResponse, ErrorResponse> {
    let camera = state.camera.lock().await;
    let index = camera.index.clone();

    #[allow(clippy::result_large_err)]
    let buffer = spawn_blocking(move || {
        let mut camera = Camera::new(index.clone(), format)
            .map_err(|e| camera_error("Impossible to create camera", e))?;

        // Open camera stream
        camera
            .open_stream()
            .map_err(|e| camera_error("Impossible to open a stream on camera", e))?;

        // Discard at least 10 camera frame before sending the correct one
        // in order to focus in lens.
        for _ in 0..10 {
            camera
                .frame()
                .map_err(|e| camera_error("Impossible to retrieve a frame for camera", e))?;
        }

        // This also allows to focus in the lens.
        let frame = camera
            .frame()
            .map_err(|e| camera_error("Impossible to retrieve a frame for camera", e))?;

        info!("Capture camera screenshot of size {}", frame.buffer().len());

        // Stop camera stream.
        camera
            .stop_stream()
            .map_err(|e| camera_error("Impossible to stop a stream for camera", e))?;

        // Decode the frame and save its content into an image buffer
        let decoded_frame = frame
            .decode_image::<RgbAFormat>()
            .map_err(|e| camera_error("Impossible to decode a frame for camera", e))?;

        info!(
            "Decoded frame: {}x{} {}",
            decoded_frame.width(),
            decoded_frame.height(),
            decoded_frame.len()
        );

        // Convert the image buffer into a `png` image, and returns a bytes buffer
        let mut bytes = Vec::new();
        decoded_frame
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .map_err(|e| camera_error("Impossible to write a `png` image for camera", e))?;

        info!("Image size {}", bytes.len());

        Ok(bytes)
    })
    .await
    .map_err(|e| camera_error("Impossible to retrieve the `png` image from thread", e))?;

    let headers = [(header::CONTENT_TYPE, "image/png")];
    Ok(StreamResponse::from_headers_reader(
        headers,
        Cursor::new(buffer?),
    ))
}

pub(crate) async fn screenshot_random(
    State(state): State<InternalState>,
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    )
    .await
}

pub(crate) async fn screenshot_absolute_resolution(
    State(state): State<InternalState>,
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    )
    .await
}

pub(crate) async fn screenshot_absolute_framerate(
    State(state): State<InternalState>,
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    )
    .await
}

pub(crate) async fn screenshot_highest_resolution(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraResolution>,
) -> Result<StreamResponse, ErrorResponse> {
    let resolution = Resolution::new(inputs.x, inputs.y);

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(resolution)),
    )
    .await
}

pub(crate) async fn screenshot_highest_framerate(
    State(state): State<InternalState>,
    Json(input): Json<CameraFramerate>,
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(input.fps)),
    )
    .await
}

pub(crate) async fn screenshot_exact(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamResponse, ErrorResponse> {
    let CameraInputs { x, y, fps, fourcc } = inputs;
    let camera_format =
        camera_format(x, y, fps, fourcc).map_err(|e| camera_error("Wrong fourcc value", e))?;

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format)),
    )
    .await
}

pub(crate) async fn screenshot_closest(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamResponse, ErrorResponse> {
    let CameraInputs { x, y, fps, fourcc } = inputs;
    let camera_format =
        camera_format(x, y, fps, fourcc).map_err(|e| camera_error("Wrong fourcc value", e))?;

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format)),
    )
    .await
}
