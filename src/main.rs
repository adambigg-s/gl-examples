mod camera;
mod mesh;
mod renderer;
mod shader;

use std::{io, mem};

#[rustfmt::skip]
const TRI_VERTICES: [f32; 24] = [
    // Position           Color              UV padding
    -0.5, -0.5, -1.0,     1.0, 0.7, 0.0,     0.0, 0.0,
    0.5 , -0.5, -1.0,     0.0, 1.0, 0.7,     0.0, 0.0,
    0.0 , 0.5 , -1.0,     0.7, 0.0, 1.0,     0.0, 0.0,
];
const TRI_INDICES: [u32; 3] = [0, 1, 2];

fn hello_triangle() {
    // Setup window and GLFW
    let mut renderer = renderer::Renderer::new();

    // Build and bind shader program
    let shader = shader::Shader::build("src/shaders/simple_vert.glsl", "src/shaders/simple_frag.glsl")
        .expect("Failed to create vert/frag shader program");
    shader.use_shader();

    // Transmute our &[f32] array into an &[Vertex] array
    let mesh = unsafe {
        mesh::Mesh::build(mem::transmute::<&[f32; 24], &[mesh::Vertex; 3]>(&TRI_VERTICES), &TRI_INDICES)
    };

    'main_loop: loop {
        if renderer.window.should_close() {
            break 'main_loop;
        }
        renderer.check_exit();

        // Draw
        renderer.clear_screen();
        mesh.render();
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

    // Load a 3D model of a penguin and position it in view of the camera
    let mut model = mesh::Model::build("assets/emperor.obj", Some("assets/emperor.jpg"))
        .expect("Failed to create 3D model");
    model.transform.position += glam::vec3(0.0, -70.0, -70.0);
    model.transform.rotation *= glam::Quat::from_rotation_x(-90.0f32.to_radians());

    // Make a simple camera
    let mut camera = camera::Camera::default();

    let light = glam::vec3(-3.0, -7.0, 10.0);

    'main_loop: loop {
        if renderer.window.should_close() {
            break 'main_loop;
        }
        // Update user inputs
        camera.update_inputs(&renderer.window);
        renderer.check_exit();

        // Apply uniforms
        shader.vec3_uniform(light, "light");
        shader.mat4_uniform(camera.get_proj(), "proj");
        shader.mat4_uniform(camera.get_view(), "view");

        // Draw
        renderer.clear_screen();
        model.render(&mut shader);
        renderer.update_frame();
    }
}

fn main() {
    println!("Enter 1 for 'Hello, Triangle' or 2 for 'Hello, Model'");
    let mut input = Default::default();
    io::stdin().read_line(&mut input).expect("Enter only 1 or 2");
    if 1 == input.trim_end().parse().unwrap() {
        hello_triangle();
    }
    if 2 == input.trim_end().parse().unwrap() {
        hello_model();
    }
}
