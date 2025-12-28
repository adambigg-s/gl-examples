mod shader;

use std::{ffi, ptr};

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

    #[rustfmt::skip]
    let vertices: [f32; _] = [
        -0.5, -0.5, 0.0, 1.0, 0.7, 0.0,
        0.5, -0.5, 0.0, 0.0, 1.0, 0.7,
        0.0, 0.5, 0.0, 0.7, 0.0, 1.0,
    ];
    let indices: [u32; _] = [0, 1, 2];
    let (mut vao, mut vbo, mut ebo) = Default::default();
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val(&vertices) as isize,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            size_of_val(&indices) as isize,
            indices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * size_of::<f32>() as i32,
            ptr::null::<ffi::c_void>(),
        );
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * size_of::<f32>() as i32,
            (3 * size_of::<f32>()) as *const ffi::c_void,
        );

        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
    }

    'main_loop: loop {
        if window.should_close() {
            break 'main_loop;
        }

        process_input(&mut window, &events);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, ptr::null());
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
