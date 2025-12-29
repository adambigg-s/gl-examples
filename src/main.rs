mod camera;
mod mesh;
mod renderer;
mod shader;

use std::mem;

#[rustfmt::skip]
const VERTICES: [f32; 18] = [
    -0.5, -0.5, -1.0, 1.0, 0.7, 0.0,
    0.5 , -0.5, -1.0, 0.0, 1.0, 0.7,
    0.0 , 0.5 , -1.0, 0.7, 0.0, 1.0,
];
const INDICES: [u32; 3] = [0, 1, 2];

fn main() {
    // Setup window and GLFW
    let mut renderer = renderer::Renderer::new();

    // Create a simple camera
    let camera = camera::Camera::new();

    // Build and bind shader program
    let mut shader = shader::Shader::build("src/shaders/simple_vert.glsl", "src/shaders/simple_frag.glsl")
        .expect("Failed to create vert/frag shader program");
    shader.use_shader();

    // Transmute our &[f32] array into an &[Vertex] array
    let mesh =
        unsafe { mesh::Mesh::build(mem::transmute::<&[f32; 18], &[mesh::Vertex; 3]>(&VERTICES), &INDICES) };

    'main_loop: loop {
        if renderer.window.should_close() {
            break 'main_loop;
        }

        shader.mat4_uniform(camera.get_proj(), "proj");
        shader.mat4_uniform(camera.get_view(), "view");
        shader.mat4_uniform(glam::Mat4::IDENTITY, "model");

        renderer.clear_screen();
        mesh.render();
        renderer.check_exit();
        renderer.update_frame();
    }
}
