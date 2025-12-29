use std::{ffi, ptr};

pub struct Transform {
    pub scl: glam::Vec3,
    pub pos: glam::Vec3,
    pub rot: glam::Quat,
}

impl Transform {
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scl, self.rot, self.pos)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scl: glam::Vec3::ONE,
            pos: glam::Vec3::ZERO,
            rot: glam::Quat::IDENTITY,
        }
    }
}

// Ensure these are completely packed -- the shader doesn't anticipate alignment
#[repr(C, packed)]
pub struct Vertex {
    pos: glam::Vec3,
    col: glam::Vec3,
}

pub struct Mesh {
    vao: u32,
    _vbo: u32,
    _ebo: u32,
    icount: u32,
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

        Self { vao, _vbo: vbo, _ebo: ebo, icount: indices.len() as u32 }
    }

    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.icount as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}
