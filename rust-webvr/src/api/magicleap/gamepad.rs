use rust_webvr_api::VRGamepad;
use rust_webvr_api::VRGamepadData;
use rust_webvr_api::VRGamepadState;

use std::sync::Arc;
use std::cell::RefCell;

pub type MagicLeapVRGamepadPtr = Arc<RefCell<MagicLeapVRGamepad>>;

pub struct MagicLeapVRGamepad {
}

impl VRGamepad for MagicLeapVRGamepad {
    fn id(&self) -> u32 {
        unimplemented!()
    }

    fn data(&self) -> VRGamepadData {
        unimplemented!()
    }

    fn state(&self) -> VRGamepadState {
        unimplemented!()
    }

}