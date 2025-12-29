use std::{ffi, ptr};

use crate::shader;

pub struct Transform {
    pub scale: glam::Vec3,
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Transform {
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: glam::Vec3::ONE,
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
        }
    }
}

// Ensure these are completely packed -- the shader doesn't anticipate alignment
#[repr(C, packed)]
pub struct Vertex {
    position: glam::Vec3,
    color: glam::Vec3,
}

pub struct Mesh {
    vert_array_ob: u32,
    _vert_buffer_ob: u32,
    _ele_buffer_ob: u32,
    index_count: u32,
}

impl Mesh {
    pub fn build(vertices: &[Vertex], indices: &[u32]) -> Self {
        let (mut vao, mut vbo, mut ebo) = Default::default();
        unsafe {
            // Generate the buffers on the GPU
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            // Bind buffers and load data
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertices) as isize,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(indices) as isize,
                indices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            // Configure vertex attributes
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                ptr::null::<ffi::c_void>(),
            );
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                (size_of::<glam::Vec3>()) as *const ffi::c_void,
            );
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        }

        Self {
            vert_array_ob: vao,
            _vert_buffer_ob: vbo,
            _ele_buffer_ob: ebo,
            index_count: indices.len() as u32,
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vert_array_ob);
            gl::DrawElements(gl::TRIANGLES, self.index_count as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}

#[derive(Default)]
pub struct Model {
    meshes: Vec<Mesh>,
    transform: Transform,
}

impl Model {
    pub fn build(path: &'static str) -> Self {
        let (models, ..) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )
        .expect("Failed to load .obj file");

        let mut meshes = Vec::new();
        for model in models {
            let mut vertices = Vec::new();
            let pos = &model.mesh.positions;

            #[allow(clippy::identity_op)]
            for i in 0..(pos.len() / 3) {
                vertices.push(Vertex {
                    position: glam::vec3(pos[i * 3 + 0], pos[i * 3 + 1], pos[i * 3 + 2]),
                    color: glam::vec3(0.7, 0.7, 0.7),
                });
            }

            meshes.push(Mesh::build(&vertices, &model.mesh.indices));
        }

        Model { meshes, ..Default::default() }
    }

    pub fn render(&self, shader: &mut shader::Shader) {
        shader.mat4_uniform(self.transform.to_matrix(), "model");
        for mesh in &self.meshes {
            mesh.render();
        }
    }
}
