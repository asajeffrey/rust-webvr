use rust_webvr_api::VRDisplayPtr;
use rust_webvr_api::VREvent;
use rust_webvr_api::VRGamepadPtr;
use rust_webvr_api::VRService;

use super::display::MagicLeapVRDisplay;
use super::display::MagicLeapVRUpdater;
use super::display::MagicLeapVRDisplayPtr;
use super::gamepad::MagicLeapVRGamepadPtr;

use super::c_api::MLResult;

use std::sync::Arc;
use std::cell::RefCell;

pub struct MagicLeapVRService {
    display: MagicLeapVRDisplayPtr,
}

impl MagicLeapVRService {
    // This function is unsafe, because it has to be called from the main
    // thread after initializing the perception system.
    pub unsafe fn new() -> Result<(MagicLeapVRService, MagicLeapVRUpdater), MLResult> {
        let (display, updater) = MagicLeapVRDisplay::new()?;
        let service = MagicLeapVRService {
            display: Arc::new(RefCell::new(display)),
        };
        Ok((service, updater))
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