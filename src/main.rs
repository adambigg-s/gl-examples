use std::mem;

mod mesh;
mod renderer;
mod shader;

#[rustfmt::skip]
const VERTICES: [f32; 18] = [
    -0.5, -0.5, 0.0, 1.0, 0.7, 0.0,
    0.5 , -0.5, 0.0, 0.0, 1.0, 0.7,
    0.0 , 0.5 , 0.0, 0.7, 0.0, 1.0,
];
const INDICES: [u32; 3] = [0, 1, 2];

fn main() {
    // Setup window and GLFW
    let mut renderer = renderer::Renderer::new();

    // Build and bind shader program
    let shader = shader::Shader::build("src/shaders/vert.glsl", "src/shaders/frag.glsl")
        .expect("Failed to create vert/frag shader program");
    shader.use_shader();

    // Transmute our &[f32] array into &[Vertex]
    let mesh =
        unsafe { mesh::Mesh::build(mem::transmute::<&[f32; 18], &[mesh::Vertex; 3]>(&VERTICES), &INDICES) };

    'main_loop: loop {
        if renderer.window.should_close() {
            break 'main_loop;
        }

        renderer.clear_screen();
        mesh.draw();
        renderer.check_exit();
        renderer.update_frame();
    }
}
