use tosca_os::extract::State;
use tosca_os::responses::error::ErrorResponse;
use tosca_os::responses::serial::SerialResponse;

use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    query,
    utils::{
        ApiBackend, CameraControl, CameraIndex, CameraInfo, FrameFormat, RequestedFormat,
        Resolution, frame_formats,
    },
};

use serde::{Deserialize, Serialize};

use tracing::info;

use crate::{InternalState, camera_error};

#[derive(Serialize, Deserialize)]
pub(crate) struct ViewCamerasResponse {
    cameras: Vec<CameraInfo>,
}

// Not a computationally intensive route, just some matches.
pub(crate) async fn show_available_cameras()
-> Result<SerialResponse<ViewCamerasResponse>, ErrorResponse> {
    // Retrieve all cameras present on a system
    let cameras = query(ApiBackend::Auto).map_err(|e| camera_error("No cameras found", e))?;

    info!("There are {} available cameras.", cameras.len());

    for camera in &cameras {
        info!("{camera}");
    }

    Ok(SerialResponse::new(ViewCamerasResponse { cameras }))
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CameraDataResponse {
    camera_index: CameraIndex,
    controls: Vec<CameraControl>,
    frame_formats: Vec<CameraFrameFormat>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CameraFrameFormat {
    frame_format: FrameFormat,
    format_data: Vec<FormatData>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FormatData {
    resolution: Resolution,
    fps: String,
}

pub(crate) async fn show_camera_info(
    State(state): State<InternalState>,
) -> Result<SerialResponse<CameraDataResponse>, ErrorResponse> {
    let config = state.camera.lock().await;

    let mut camera = Camera::new(
        config.index.clone(),
        RequestedFormat::new::<RgbFormat>(config.format_type),
    )
    .map_err(|e| camera_error("Impossible to create camera", e))?;

    // Get controls for a camera
    let controls = camera
        .camera_controls()
        .map_err(|e| camera_error("Impossible to retrieve controls for camera", e))?;

    info!("Control for camera with index {}", config.index);

    // Show controls
    for control in &controls {
        info!("{control}");
    }

    // Iterate over frame formats.
    let frame_formats = frame_formats()
        .iter()
        .filter_map(|frame_format| {
            // Among the frame formats, save the ones compatible with the camera
            camera
                .compatible_list_by_resolution(*frame_format)
                .map(|compatible| {
                    info!("{frame_format}:");

                    let mut formats = compatible
                        .into_iter()
                        .collect::<Vec<(Resolution, Vec<u32>)>>();

                    // Sort formats by name
                    formats.sort_by(|a, b| a.0.cmp(&b.0));

                    // Show sorted formats.
                    let format_data = formats
                        .into_iter()
                        .map(|(resolution, fps)| {
                            let fps = format!("{fps:?}");
                            info!(" - {resolution}: {fps}");
                            FormatData { resolution, fps }
                        })
                        .collect::<Vec<FormatData>>();

                    CameraFrameFormat {
                        frame_format: *frame_format,
                        format_data,
                    }
                })
                .ok()
        })
        .collect::<Vec<CameraFrameFormat>>();

    Ok(SerialResponse::new(CameraDataResponse {
        camera_index: config.index.clone(),
        controls,
        frame_formats,
    }))
}
