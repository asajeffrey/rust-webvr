// Rust bindings to the subset of the MagicLeap C API needed for WebVR.
use euclid::Point3D;
use euclid::Rotation3D;

#[repr(C)]
#[must_use]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MLResult(i32);

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MLHandle(u64);

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MLCoordinateFrameUID([u64; 2]);

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MLTransform {
   // TODO: used typed units to remember the coordinate space
   pub position: Point3D<f64>,
   pub rotation: Rotation3D<f64>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MLEyeTrackingStaticData {
    pub fixation: MLCoordinateFrameUID,
    pub left_center: MLCoordinateFrameUID,
    pub right_center: MLCoordinateFrameUID,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MLGraphicsOptions {
    pub color_format: MLSurfaceFormat,
    pub depth_format: MLSurfaceFormat,
    pub graphics_flags: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MLSurfaceFormat(u32);

#[repr(C)]
pub struct MLSnapshot(usize);

extern {
    pub fn MLPerceptionGetSnapshot (out_snapshot: *mut *mut MLSnapshot) -> MLResult;
    pub fn MLPerceptionReleaseSnapshot (snap: *mut MLSnapshot) -> MLResult;
    pub fn MLSnapshotGetTransform (snapshot: *const MLSnapshot, id: *const MLCoordinateFrameUID, out_transform: *mut MLTransform) -> MLResult;
    pub fn MLEyeTrackingCreate (out_handle: *mut MLHandle) -> MLResult;
    pub fn MLEyeTrackingDestroy (handle: MLHandle) -> MLResult;
    pub fn MLEyeTrackingGetStaticData (eye_tracker: MLHandle, out_data: *mut MLEyeTrackingStaticData) -> MLResult;
    pub fn MLGraphicsCreateClientGL(options: *const MLGraphicsOptions, opengl_context: *const MLHandle, out_graphics_client: *mut MLHandle) -> MLResult;
    pub fn MLGraphicsDestroyClient(graphics_client: *mut MLHandle) -> MLResult;
}

impl MLResult {
    pub fn ok(self) -> Result<(), MLResult> {
        if self.0 == 0 {
            Ok(())
        } else {
            Err(self)
        }
    }
}