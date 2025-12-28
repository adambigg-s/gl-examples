mod shader;

use std::ffi;

use glfw::*;

const WINDOW_NAME: &str = "GL Examples";
const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 1200;
const WINDOW_RED: f32 = 15.0 / 255.0;
const WINDOW_GRE: f32 = 25.0 / 255.0;
const WINDOW_BLU: f32 = 40.0 / 255.0;
const WINDOW_ALP: f32 = 1.0;

fn main() {
    // Setup GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors!()).expect("GLFW failed");

    // Create context
    glfw.window_hint(WindowHint::Focused(true));
    glfw.window_hint(WindowHint::Resizable(true));
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(6));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    let (mut window, events) = glfw
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
        gl::ClearColor(WINDOW_RED, WINDOW_GRE, WINDOW_BLU, WINDOW_ALP);
        gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
    }

    // Build and bind shader program
    let shader = shader::Shader::new("src/shaders/vert.glsl", "src/shaders/frag.glsl")
        .expect("Failed to create vert/frag shader program");
    shader.use_shader();

    'main_loop: loop {
        if window.should_close() {
            break 'main_loop;
        }

        process_input(&mut window, &events);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        glfw.poll_events();
        window.swap_buffers();
    }
}

fn process_input(window: &mut PWindow, events: &GlfwReceiver<(f64, WindowEvent)>) {
    while let Some((_, event)) = events.receive() {
        println!("{:?}", &event);
        unsafe {
            match event {
                | WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                | WindowEvent::Size(width, height) => {
                    gl::Viewport(0, 0, width, height);
                }
                | _ => {}
            }
        }
    }
}
