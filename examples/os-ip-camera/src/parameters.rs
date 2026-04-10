use tosca_os::extract::{Json, Path, State};
use tosca_os::responses::error::ErrorResponse;
use tosca_os::responses::ok::OkResponse;
use tosca_os::responses::serial::SerialResponse;

use nokhwa::{
    query,
    utils::{ApiBackend, CameraIndex, RequestedFormatType, Resolution},
};

use serde::{Deserialize, Serialize};

use tracing::{error, info};

use crate::{InternalState, camera_error, camera_format};

#[derive(Serialize, Deserialize)]
pub(crate) struct CameraResponse {
    message: String,
}

pub(crate) async fn change_camera(
    State(state): State<InternalState>,
    Path(index): Path<String>,
) -> Result<SerialResponse<CameraResponse>, ErrorResponse> {
    let index = match index.parse() {
        Ok(value) => CameraIndex::Index(value),
        Err(_) => CameraIndex::String(index),
    };

    let mut config = state.camera.lock().await;

    // Check whether the given index is equal to the previous one.
    if config.index == index {
        let message = format!("Camera index remained unchanged at index `{index}`");
        info!("{message}");
        return Ok(SerialResponse::new(CameraResponse { message }));
    }

    // Retrieve all cameras present on a system
    let cameras = query(ApiBackend::Auto).map_err(|e| camera_error("No cameras found", e))?;

    for camera in &cameras {
        if *camera.index() == index {
            config.index = index.clone();
            break;
        }
    }

    if config.index == index {
        let message = format!("Camera changed to index `{index}`");
        info!("{message}");
        Ok(SerialResponse::new(CameraResponse { message }))
    } else {
        let message = format!("`{index}` index does not exist");
        error!("{message}");
        Err(ErrorResponse::invalid_data(&message))
    }
}

pub(crate) async fn format_random(
    State(state): State<InternalState>,
) -> Result<OkResponse, ErrorResponse> {
    let mut config = state.camera.lock().await;
    config.format_type = RequestedFormatType::None;
    info!("Changed to random format");
    Ok(OkResponse::ok())
}

pub(crate) async fn format_absolute_resolution(
    State(state): State<InternalState>,
) -> Result<OkResponse, ErrorResponse> {
    let mut config = state.camera.lock().await;
    config.format_type = RequestedFormatType::AbsoluteHighestResolution;
    info!("Changed to absolute highest resolution format");
    Ok(OkResponse::ok())
}

pub(crate) async fn format_absolute_framerate(
    State(state): State<InternalState>,
) -> Result<OkResponse, ErrorResponse> {
    let mut config = state.camera.lock().await;
    config.format_type = RequestedFormatType::AbsoluteHighestFrameRate;
    info!("Changed to absolute highest frame rate format");
    Ok(OkResponse::ok())
}

#[derive(Deserialize)]
pub(crate) struct CameraResolution {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

pub(crate) async fn format_highest_resolution(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraResolution>,
) -> Result<OkResponse, ErrorResponse> {
    let mut config = state.camera.lock().await;
    config.format_type =
        RequestedFormatType::HighestResolution(Resolution::new(inputs.x, inputs.y));
    info!(
        "Changed to highest resolution ({}, {}) format",
        inputs.x, inputs.y
    );
    Ok(OkResponse::ok())
}

#[derive(Deserialize)]
pub(crate) struct CameraFramerate {
    pub(crate) fps: u32,
}

pub(crate) async fn format_highest_framerate(
    State(state): State<InternalState>,
    Json(input): Json<CameraFramerate>,
) -> Result<OkResponse, ErrorResponse> {
    let mut config = state.camera.lock().await;
    config.format_type = RequestedFormatType::HighestFrameRate(input.fps);
    info!("Changed to highest frame rate ({} fps) format", input.fps);
    Ok(OkResponse::ok())
}

#[derive(Deserialize)]
pub(crate) struct CameraInputs {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) fps: u32,
    pub(crate) fourcc: String,
}

pub(crate) async fn format_exact(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<OkResponse, ErrorResponse> {
    let CameraInputs { x, y, fps, fourcc } = inputs;
    let camera_format =
        camera_format(x, y, fps, &fourcc).map_err(|e| camera_error("Wrong fourcc value", e))?;

    let mut config = state.camera.lock().await;
    config.format_type = RequestedFormatType::Exact(camera_format);
    info!("Changed to resolution ({x}, {y}) and {fps} fps and fourcc `{fourcc}` exact format");
    Ok(OkResponse::ok())
}

pub(crate) async fn format_closest(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<OkResponse, ErrorResponse> {
    let CameraInputs { x, y, fps, fourcc } = inputs;
    let camera_format =
        camera_format(x, y, fps, &fourcc).map_err(|e| camera_error("Wrong fourcc value", e))?;

    let mut config = state.camera.lock().await;
    config.format_type = RequestedFormatType::Closest(camera_format);
    info!("Changed to resolution ({x}, {y}) and {fps} fps and fourcc `{fourcc}` closest format");
    Ok(OkResponse::ok())
}
