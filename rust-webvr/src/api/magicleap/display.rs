use gleam::gl;
use gleam::gl::Gl;

use rust_webvr_api::VRDisplay;
use rust_webvr_api::VRDisplayCapabilities;
use rust_webvr_api::VRDisplayData;
use rust_webvr_api::VREyeParameters;
use rust_webvr_api::VRFieldOfView;
use rust_webvr_api::VRFrameData;
use rust_webvr_api::VRFramebuffer;
use rust_webvr_api::VRFramebufferAttributes;
use rust_webvr_api::VRFutureFrameData;
use rust_webvr_api::VRLayer;
use rust_webvr_api::utils;

use super::c_api::MLPerceptionGetSnapshot;
use super::c_api::MLPerceptionReleaseSnapshot;
use super::c_api::MLEyeTrackingCreate;
use super::c_api::MLEyeTrackingDestroy;
use super::c_api::MLEyeTrackingGetStaticData;
use super::c_api::MLSnapshotGetTransform;
use super::c_api::MLGraphicsOptions;
use super::c_api::MLGraphicsCreateClientGL;
use super::c_api::MLGraphicsDestroyClient;
use super::c_api::MLHandle;
use super::c_api::MLResult;
use super::heartbeat::MagicLeapVRMainThreadHeartbeat;
use super::heartbeat::MagicLeapVRMessage;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::cell::Cell;
use std::cell::RefCell;
use std::mem;

pub type MagicLeapVRDisplayPtr = Arc<RefCell<MagicLeapVRDisplay>>;

pub struct MagicLeapVRDisplay {
    id: u32,
    display_data: VRDisplayData,
    sender: Sender<MagicLeapVRMessage>,
    gl_context: MLHandle,
    graphics_client: MLHandle,
}

// This is a lie! It is deeply deeply unsound and probably insta-UB.
// Nevertheless, the rust-webvr API requires us to persist.
unsafe impl Send for MagicLeapVRDisplay {}
unsafe impl Sync for MagicLeapVRDisplay {}

const DISPLAY_NAME: &str = "MagicLeap Display";

// TODO: find these values from the device?
const RENDER_WIDTH: u32 = 1280;
const RENDER_HEIGHT: u32 = 720;
const DEFAULT_NEAR: f64 = 0.35;
const DEFAULT_FAR: f64 = 100.0;

// https://creator.magicleap.com/learn/guides/field-of-view
const VERTICAL_FOV: f64 = 30.0;
const HORIZONTAL_FOV: f64 = 20.0;

impl VRDisplay for MagicLeapVRDisplay {
    fn id(&self) -> u32 {
        self.id
    }

    fn data(&self) -> VRDisplayData {
        self.display_data.clone()
    }

    fn immediate_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        unimplemented!()
    }

    fn synced_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        self.immediate_frame_data(near, far)
    }

    fn reset_pose(&mut self) {}

    fn sync_poses(&mut self) {}

    fn future_frame_data(&mut self, near: f64, far: f64) -> VRFutureFrameData {
        let (resolver, result) = VRFutureFrameData::blocked();
        let _ = self.sender.send(MagicLeapVRMessage::StartFrame { near, far, resolver });
        result
    }

    fn bind_framebuffer(&mut self, eye_index: u32) {}

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        unimplemented!()
    }

    fn render_layer(&mut self, layer: &VRLayer) {
        unreachable!()
    }

    fn submit_frame(&mut self) {
        unreachable!()
    }

    fn submit_layer(&mut self, gl: &Gl, layer: &VRLayer) {
        unimplemented!()
    }
    
    fn start_present(&mut self, _attributes: Option<VRFramebufferAttributes>) {
        unimplemented!()
    }

    fn stop_present(&mut self) {
        unimplemented!()
    }
}

impl MagicLeapVRDisplay {
    // This function is unsafe because it must be run on the main thread
    // after initializing the perception system.
    pub unsafe fn new() -> Result<(MagicLeapVRDisplay, MagicLeapVRMainThreadHeartbeat), MLResult> {
        let (sender, receiver) = mpsc::channel();
        let display = MagicLeapVRDisplay {
            id: utils::new_id(),
            display_data: MagicLeapVRDisplay::display_data()?,
            sender: sender,
	    gl_context: unimplemented!(),
	    graphics_client: mem::zeroed(),
        };
        let heartbeat = MagicLeapVRMainThreadHeartbeat::new(receiver);
        Ok((display, heartbeat))
    }

    // This function is unsafe because it must be run on the main thread
    // after initializing the perception system.
    unsafe fn display_data() -> Result<VRDisplayData, MLResult> {
       let (left, right) = MagicLeapVRDisplay::eye_parameters()?;
       Ok(VRDisplayData {
          capabilities: MagicLeapVRDisplay::capabilities()?,
          // TODO: this should really query the device
          connected: true,
          display_id: utils::new_id(),
          display_name: String::from(DISPLAY_NAME),
          left_eye_parameters: left,
          right_eye_parameters: right,
          stage_parameters: None,
       })
   }

   fn capabilities() -> Result<VRDisplayCapabilities, MLResult> {
        Ok(VRDisplayCapabilities {
            has_position: true,
            has_orientation: true,
            has_external_display: false,
            can_present: true,
            presented_by_browser: true,
            max_layers: 1,
        })
    }

    // This function is unsafe because it must be run on the main thread
    // after initializing the perception system.
    unsafe fn eye_parameters() -> Result<(VREyeParameters, VREyeParameters), MLResult> {
        let mut snapshot = mem::zeroed();
        let mut eye_tracker = mem::zeroed();
        let mut static_data = mem::zeroed();
        let mut left_center = mem::zeroed();
        let mut right_center = mem::zeroed();

        MLPerceptionGetSnapshot(&mut snapshot).ok()?;
        MLEyeTrackingCreate(&mut eye_tracker).ok()?;
        MLEyeTrackingGetStaticData(eye_tracker, &mut static_data).ok()?;
        MLSnapshotGetTransform(snapshot, &static_data.left_center, &mut left_center).ok()?;
        MLSnapshotGetTransform(snapshot, &static_data.right_center, &mut right_center).ok()?;

        let left_offset = (left_center.position - right_center.position) * 0.5;
        let right_offset = left_offset * -1.0;

        let (left_fov, right_fov) = MagicLeapVRDisplay::field_of_view()?;

        let left = VREyeParameters {
            offset: left_offset.to_f32().to_array(),
            render_width: RENDER_WIDTH,
            render_height: RENDER_HEIGHT,
            field_of_view: left_fov,
        };

        let right = VREyeParameters {
            offset: right_offset.to_f32().to_array(),
            render_width: RENDER_WIDTH,
            render_height: RENDER_HEIGHT,
            field_of_view: right_fov,
        };

        let _ = MLPerceptionReleaseSnapshot(snapshot);
        let _ = MLEyeTrackingDestroy(eye_tracker);

        Ok((left, right))
    }

    fn field_of_view() -> Result<(VRFieldOfView, VRFieldOfView), MLResult> {
        // TODO: Get this from the device.
        let left_fov = VRFieldOfView {
            up_degrees: VERTICAL_FOV / 2.0,
            right_degrees: HORIZONTAL_FOV / 2.0,
            down_degrees: VERTICAL_FOV / 2.0,
            left_degrees: HORIZONTAL_FOV / 2.0,
        };
        let right_fov = left_fov.clone();
        Ok((left_fov, right_fov))
    }

    unsafe fn init_graphics_client(&mut self) -> Result<(), MLResult> {
        // TODO: initialize GL context
        let options = MLGraphicsOptions {
           color_format: unimplemented!(),
           depth_format: unimplemented!(),
           graphics_flags: unimplemented!(),
        };
	let gl_context = unimplemented!();
        MLGraphicsCreateClientGL(&options, &self.gl_context, &mut self.graphics_client).ok()?;
        Ok(())
    }

    unsafe fn destroy_graphics_client(&mut self) -> Result<(), MLResult> {
        MLGraphicsDestroyClient(&mut self.graphics_client).ok()?;
        Ok(())
    }
}
