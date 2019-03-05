use gleam::gl::Gl;

use rust_webvr_api::VRDisplayPtr;
use rust_webvr_api::VREvent;
use rust_webvr_api::VRGamepadPtr;
use rust_webvr_api::VRService;

use super::display::MagicLeapVRDisplay;
use super::heartbeat::MagicLeapVRMainThreadHeartbeat;
use super::display::MagicLeapVRDisplayPtr;
use super::gamepad::MagicLeapVRGamepadPtr;

use super::c_api::MLResult;

use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MagicLeapVRService {
    display: MagicLeapVRDisplayPtr,
}

impl MagicLeapVRService {
    // This function is unsafe, because it has to be called from the main
    // thread after initializing the perception system.
    pub unsafe fn new(gl_context: Rc<Gl>) -> Result<(MagicLeapVRService, MagicLeapVRMainThreadHeartbeat), MLResult> {
        let (display, heartbeat) = MagicLeapVRDisplay::new(gl_context)?;
        let service = MagicLeapVRService {
            display: Arc::new(RefCell::new(display)),
        };
        Ok((service, heartbeat))
    }
}

// This is unsound, and can lead to UB, but is required by the API.
// https://github.com/servo/rust-webvr/issues/18
unsafe impl Send for MagicLeapVRService {}

impl VRService for MagicLeapVRService {
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        Ok(vec![self.display.clone()])
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        // TODO: gamepads
        Ok(vec![])
    }

    fn is_available(&self) -> bool {
        true
    }

    fn poll_events(&self) -> Vec<VREvent> {
        // TODO: event polling
        Vec::new()
    }
}