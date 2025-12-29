mod camera;
mod mesh;
mod renderer;
mod shader;

use std::{io, mem};

#[rustfmt::skip]
const VERTICES: [f32; 18] = [
    -0.5, -0.5, -1.0, 1.0, 0.7, 0.0,
    0.5 , -0.5, -1.0, 0.0, 1.0, 0.7,
    0.0 , 0.5 , -1.0, 0.7, 0.0, 1.0,
];
const INDICES: [u32; 3] = [0, 1, 2];

fn hello_triangle() {
    // Setup window and GLFW
    let mut renderer = renderer::Renderer::new();

    // Build and bind shader program
    let shader = shader::Shader::build("src/shaders/simple_vert.glsl", "src/shaders/simple_frag.glsl")
        .expect("Failed to create vert/frag shader program");
    shader.use_shader();

    // Transmute our &[f32] array into an &[Vertex] array
    let mesh =
        unsafe { mesh::Mesh::build(mem::transmute::<&[f32; 18], &[mesh::Vertex; 3]>(&VERTICES), &INDICES) };

    'main_loop: loop {
        if renderer.window.should_close() {
            break 'main_loop;
        }

        renderer.clear_screen();
        mesh.render();
        renderer.check_exit();
        renderer.update_frame();
    }
}

fn hello_model() {
    // Setup window and GLFW
    let mut renderer = renderer::Renderer::new();

    // Build and bind shader program
    let mut shader = shader::Shader::build("src/shaders/model_vert.glsl", "src/shaders/model_frag.glsl")
        .expect("Failed to create vert/frag shader program");
    shader.use_shader();

    // Transmute our &[f32] array into an &[Vertex] array
    let mesh =
        unsafe { mesh::Mesh::build(mem::transmute::<&[f32; 18], &[mesh::Vertex; 3]>(&VERTICES), &INDICES) };

    // Make a simple camera
    let camera = camera::Camera::new();

    'main_loop: loop {
        if renderer.window.should_close() {
            break 'main_loop;
        }

        // Apply uniforms
        shader.mat4_uniform(camera.get_proj(), "proj");
        shader.mat4_uniform(camera.get_view(), "view");
        shader.mat4_uniform(mesh::Transform::default().to_matrix(), "model");

        renderer.clear_screen();
        mesh.render();
        renderer.check_exit();
        renderer.update_frame();
    }
}

fn main() {
    println!("Enter 1 for 'Hello, Triangle' or 2 for 3D 'Hello, Model'");
    let mut input = Default::default();
    io::stdin().read_line(&mut input).expect("Enter only 1 or 2");
    if 1 == input.trim_end().parse().unwrap() {
        hello_triangle();
    }
    if 2 == input.trim_end().parse().unwrap() {
        hello_model();
    }
}
