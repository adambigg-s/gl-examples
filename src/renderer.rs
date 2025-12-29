use std::ffi;

use glfw::*;

pub const FOV_DEGREES: f32 = 90.0;
pub const NEAR_PLANE: f32 = 0.1;
pub const FAR_PLANE: f32 = 500.0;

pub const WINDOW_WIDTH: u32 = 1600;
pub const WINDOW_HEIGHT: u32 = 1200;

const WINDOW_NAME: &str = "GL Examples";
const WINDOW_RED: f32 = 15.0 / 255.0;
const WINDOW_GRE: f32 = 25.0 / 255.0;
const WINDOW_BLU: f32 = 40.0 / 255.0;
const WINDOW_ALP: f32 = 1.0;

pub struct Renderer {
    pub glfw: Glfw,
    pub window: PWindow,
}

impl Renderer {
    pub fn new() -> Self {
        // Setup GLFW
        let mut glfw = glfw::init(glfw::fail_on_errors!()).expect("GLFW failed");

        // Create context
        glfw.window_hint(WindowHint::Focused(true));
        glfw.window_hint(WindowHint::Resizable(false));
        glfw.window_hint(WindowHint::ContextVersionMajor(4));
        glfw.window_hint(WindowHint::ContextVersionMinor(6));
        glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        // We ignore the async events handler, we just query the window for events
        let (mut window, ..) = glfw
            .create_window(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_NAME, WindowMode::Windowed)
            .expect("Window failed");
        window.show();
        window.focus();
        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        // Load OpenGL function pointers
        gl::load_with(|fn_ptr| {
            window.get_proc_address(fn_ptr).expect("Unable to overload Gl function") as *const ffi::c_void
        });

        // Set default view
        unsafe {
            gl::Enable(gl::MIPMAP);
            gl::Enable(gl::DEPTH_TEST);
            gl::ClearColor(WINDOW_RED, WINDOW_GRE, WINDOW_BLU, WINDOW_ALP);
            gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
        }

        Renderer { glfw, window }
    }

    pub fn clear_screen(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    // Draw back-buffer to the screen
    pub fn update_frame(&mut self) {
        self.glfw.poll_events();
        self.window.swap_buffers();
    }

    // Determine if the window should close
    pub fn check_exit(&mut self) {
        if self.window.get_key(Key::Escape) == Action::Press {
            self.window.set_should_close(true);
        }
    }
}
