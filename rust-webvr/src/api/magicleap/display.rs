#![allow(unused_variables)]

use atomicbox::AtomicBox;

use rust_webvr_api::VRDisplay;
use rust_webvr_api::VRDisplayCapabilities;
use rust_webvr_api::VRDisplayData;
use rust_webvr_api::VREyeParameters;
use rust_webvr_api::VRFieldOfView;
use rust_webvr_api::VRFrameData;
use rust_webvr_api::VRFramebuffer;
use rust_webvr_api::VRFramebufferAttributes;
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

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::cell::Cell;
use std::cell::RefCell;
use std::mem;

// This object only exists on the main thread, and is used to update
// the VRDisplay each time round the main event loop.
pub struct MagicLeapVRUpdater {
    frame_data_to_vr_thread: Arc<AtomicBox<Option<VRFrameData>>>,
    frame_data_buffer: Box<Option<VRFrameData>>,
    this_struct_isnt_sync_or_send: Cell<()>,
}

pub type MagicLeapVRDisplayPtr = Arc<RefCell<MagicLeapVRDisplay>>;

pub struct MagicLeapVRDisplay {
    id: u32,
    presenting: bool,
    display_data: VRDisplayData,
    frame_data_from_main_thread: Arc<AtomicBox<Option<VRFrameData>>>,
    // There is interior mutability here to work round
    // https://github.com/servo/rust-webvr/issues/53
    frame_data_buffer: RefCell<Box<Option<VRFrameData>>>,
    frame_data: Cell<VRFrameData>,
    near: Cell<f64>,
    far: Cell<f64>,
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
        if (near != self.near.get()) || (far != self.far.get()) {
           self.near.set(near);
           self.far.set(far);
           self.send_near_and_far_to_main_thread();
        }

        self.recv_frame_data_from_main_thread()
    }

    fn synced_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        self.immediate_frame_data(near, far)
    }

    fn reset_pose(&mut self) {
        unimplemented!()
    }

    fn sync_poses(&mut self) {
        unimplemented!()
    }

    fn bind_framebuffer(&mut self, eye_index: u32) {
        unimplemented!()
    }

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        unimplemented!()
    }

    fn render_layer(&mut self, layer: &VRLayer) {
        unimplemented!()
    }

    fn submit_frame(&mut self) {
        unimplemented!()
    }

    fn start_present(&mut self, attributes: Option<VRFramebufferAttributes>) {
        unimplemented!()
    }

    fn stop_present(&mut self) {
        unimplemented!()
    }
}

impl MagicLeapVRDisplay {
    // This function is unsafe because it must be run on the main thread
    // after initializing the perception system.
    pub unsafe fn new() -> Result<(MagicLeapVRDisplay, MagicLeapVRUpdater), MLResult> {
        let frame_data_shared_buffer = Arc::new(AtomicBox::new(Box::new(None)));
        let display = MagicLeapVRDisplay {
            id: utils::new_id(),
            presenting: false,
            display_data: MagicLeapVRDisplay::display_data()?,
	    frame_data_from_main_thread: frame_data_shared_buffer.clone(),
            frame_data_buffer: RefCell::new(Box::new(None)),
            frame_data: Cell::new(VRFrameData::default()),
            near: Cell::new(DEFAULT_NEAR),
            far: Cell::new(DEFAULT_FAR),
        };
        let updater = MagicLeapVRUpdater {
            frame_data_to_vr_thread: frame_data_shared_buffer,
            frame_data_buffer: Box::new(None),
            this_struct_isnt_sync_or_send: Cell::new(()),
        };
        Ok((display, updater))
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
        if self.presenting {
            return Ok(());
        }

        // TODO: initialize GL context
        let options = MLGraphicsOptions {
           color_format: unimplemented!(),
           depth_format: unimplemented!(),
           graphics_flags: unimplemented!(),
        };
        // MLGraphicsCreateClientGL(&options, &self.gl_context, &mut self.graphics_client).ok()?;

        self.presenting = true;
        Ok(())
    }

    unsafe fn destroy_graphics_client(&mut self) -> Result<(), MLResult> {
        if !self.presenting {
            return Ok(());
        }

        // MLGraphicsDestroyClient(&mut self.graphics_client).ok()?;

        self.presenting = false;
        Ok(())
    }

    fn send_near_and_far_to_main_thread(&self) {
        unimplemented!()
    }

    fn recv_frame_data_from_main_thread(&self) -> VRFrameData {
        // Annoyingly, we don't have `&mut self`, so we need to use
        // interior mutability here.
        let mut buffer = self.frame_data_buffer.borrow_mut(); 
        self.frame_data_from_main_thread.swap_mut(&mut *buffer, Ordering::AcqRel);
        let result = self.frame_data_buffer.borrow_mut().take().unwrap_or_else(|| self.frame_data.take());
        self.frame_data.set(result.clone());
        result
   }
}

impl MagicLeapVRUpdater {
    fn get_current_frame_data(&mut self) -> Result<VRFrameData, MLResult> {
        unimplemented!()
    }

    fn send_frame_data_to_vr_thread(&mut self) -> Result<(), MLResult> {
        let frame_data = self.get_current_frame_data()?;
        *self.frame_data_buffer = Some(frame_data);
        self.frame_data_to_vr_thread.swap_mut(&mut self.frame_data_buffer, Ordering::AcqRel);
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), MLResult> {
        self.send_frame_data_to_vr_thread()?;
        Ok(())
    }
}