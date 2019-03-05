use gleam::gl;
use gleam::gl::Gl;
use gleam::gl::GLuint;
use rust_webvr_api::VRResolveFrameData;
use rust_webvr_api::VRMainThreadHeartbeat;
use std::mem;
use std::rc::Rc;
use std::time::Duration;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use api::magicleap::c_api::MLHandle;
use api::magicleap::c_api::MLGraphicsCreateClientGL;
use api::magicleap::c_api::MLGraphicsDestroyClient;
use api::magicleap::c_api::MLGraphicsOptions;
use api::magicleap::c_api::MLResult;

const TIMEOUT: Duration = Duration::from_millis(16);

pub struct MagicLeapVRMainThreadHeartbeat {
    receiver: Receiver<MagicLeapVRMessage>,
    gl: Rc<dyn Gl>,
    graphics_client: MLHandle,
    timestamp: f64,
}

impl VRMainThreadHeartbeat for MagicLeapVRMainThreadHeartbeat {
    fn heartbeat(&mut self) {
       debug!("VR heartbeat start");
       while let Ok(msg) = self.receiver.recv_timeout(TIMEOUT) {
           if self.handle_msg(msg) { break; }
        }
        debug!("VR heartbeat stop");
    }

    fn heart_racing(&self) -> bool {
        true
    }
}

impl MagicLeapVRMainThreadHeartbeat {
    // This function is unsafe because it must be called on the main thread
    // after initializing the perception system.
    pub(crate) unsafe fn new(
        receiver: Receiver<MagicLeapVRMessage>, 
        gl: Rc<Gl>,
    ) -> Result<MagicLeapVRMainThreadHeartbeat, MLResult> {
        debug!("Creating VR heartbeat");
        let options = MLGraphicsOptions {
           color_format: unimplemented!(),
           depth_format: unimplemented!(),
           graphics_flags: unimplemented!(),
        };
	let gl_context = unimplemented!();
	let mut graphics_client = mem::zeroed();
        MLGraphicsCreateClientGL(&options, &gl_context, &mut graphics_client).ok()?;
        Ok(MagicLeapVRMainThreadHeartbeat {
            receiver: receiver,
            gl: gl,
	    graphics_client: graphics_client,
            timestamp: 0.0,
        })
    }

    fn handle_msg(&mut self, msg: MagicLeapVRMessage) -> bool {
           match msg {
               MagicLeapVRMessage::StartFrame { near, far, mut resolver } => {
                   debug!("VR start frame");
		   unimplemented!()
                   // let timestamp = self.timestamp;
                   // let size = self.gl_window.get_inner_size().expect("No window size");
                   // let hidpi = self.gl_window.get_hidpi_factor();
                   // let size = size.to_physical(hidpi);
                   // let data = MagicLeapVRDisplay::frame_data(timestamp, size, near, far);
                   // let _ = resolver.resolve(data);
                   // self.timestamp = self.timestamp + 1.0;
                   // false
               },
               MagicLeapVRMessage::StopFrame { width, height, texture_id } => {
                   debug!("VR stop frame {}x{} ({})", width, height, texture_id);
		   unimplemented!()
                   // if let Err(err) = unsafe { self.gl_window.make_current() } {
		   //     error!("VR Display failed to make window current ({:?})", err);
		   //     return true;
		   // }
                   // if self.texture_id == 0 {
                   //     self.texture_id = self.gl.gen_textures(1)[0];
                   //     debug!("Generated texture {}", self.texture_id);
                   // }
                   // if self.framebuffer_id == 0 {
                   //     self.framebuffer_id = self.gl.gen_framebuffers(1)[0];
                   //     debug!("Generated framebuffer {}", self.framebuffer_id);
                   // }

                   // self.gl.clear_color(0.2, 0.3, 0.3, 1.0);
                   // self.gl.clear(gl::COLOR_BUFFER_BIT);

                   // self.gl.bind_texture(gl::TEXTURE_2D, self.texture_id);
                   // self.gl.tex_image_2d(
                   //     gl::TEXTURE_2D,
                   //     0,
                   //     gl::RGBA as gl::GLint,
                   //     width as gl::GLsizei,
                   //     height as gl::GLsizei,
                   //     0,
                   //     gl::RGBA,
                   //     gl::UNSIGNED_BYTE,
                   //     Some(&buffer[..]),
                   // );
                   // self.gl.bind_texture(gl::TEXTURE_2D, 0);

                   // self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_id);
                   // self.gl.framebuffer_texture_2d(
                   //     gl::READ_FRAMEBUFFER, 
                   //     gl::COLOR_ATTACHMENT0,
                   //     gl::TEXTURE_2D,
                   //     self.texture_id,
                   //     0
                   // );
                   // self.gl.viewport(
                   //     0, 0, width as gl::GLsizei, height as gl::GLsizei,
                   // );
                   // self.gl.blit_framebuffer(
                   //     0, 0, width as gl::GLsizei, height as gl::GLsizei,
                   //     0, 0, width as gl::GLsizei, height as gl::GLsizei,
                   //     gl::COLOR_BUFFER_BIT,
                   //     gl::NEAREST,
                   // );
                   // self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, 0);

                   // let _ = self.gl_window.swap_buffers();

                   // let err = self.gl.get_error();
                   // if err != 0 {
                   //     error!("Test VR Display GL error {}.", err);
                   // }

                   // true
               },
           }
    }
}

impl Drop for MagicLeapVRMainThreadHeartbeat {
    fn drop(&mut self) {
        // This is safe because if this drop code is run,
	// then we must be on the main thread (since MagicLeapVRMainThreadHeartbeat
	// doesn't implement Send) and the perception system must have been
	// initialized.
        unsafe { MLGraphicsDestroyClient(&mut self.graphics_client) }; 
    }
}

pub(crate) enum MagicLeapVRMessage {
    StartFrame { near: f64, far: f64, resolver: VRResolveFrameData },
    StopFrame { width: u32, height: u32, texture_id: Arc<GLuint> },
}
