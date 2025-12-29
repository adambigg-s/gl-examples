use std::{
    ffi::{self},
    ptr,
};

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

pub struct Texture {
    id: u32,
}

impl Texture {
    pub fn new(path: &'static str) -> Result<Self, String> {
        let image = image::open(path).map_err(|err| err.to_string())?.flipv().to_rgba8();
        let (image_width, image_height) = image.dimensions();

        let mut texture = Default::default();
        unsafe {
            // Generate the texture buffer
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Send texture data
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                image_width as i32,
                image_height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.as_ptr().cast(),
            );
            gl::GenerateMipmap(texture);
        }

        Ok(Self { id: texture })
    }

    pub fn use_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

// Ensure these are completely packed -- the shader doesn't anticipate alignment
#[repr(C, packed)]
#[derive(Default)]
pub struct Vertex {
    position: glam::Vec3,
    color_or_normal: glam::Vec3,
    texture_uvs: glam::Vec2,
}

pub struct Mesh {
    vert_array_obj: u32,
    index_count: u32,

    _vert_buffer_obj: u32,
    _elem_buffer_obj: u32,
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
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                (size_of::<glam::Vec3>() * 2) as *const ffi::c_void,
            );
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
        }

        Self {
            vert_array_obj: vao,
            index_count: indices.len() as u32,

            _vert_buffer_obj: vbo,
            _elem_buffer_obj: ebo,
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vert_array_obj);
            gl::DrawElements(gl::TRIANGLES, self.index_count as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}

#[derive(Default)]
pub struct Model {
    pub transform: Transform,
    meshes: Vec<Mesh>,
    texture: Option<Texture>,
}

impl Model {
    pub fn build(model_path: &'static str, texture_path: Option<&'static str>) -> Result<Self, String> {
        // Load the obj with 'tobj'
        let (models, ..) = tobj::load_obj(
            model_path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )
        .map_err(|err| err.to_string())?;

        // Load the texture if one is supplied
        let mut texture = None;
        if let Some(texture_path) = texture_path {
            texture = Some(Texture::new(texture_path).map_err(|err| err.to_string())?);
        }

        // Parse the mesh
        let mut meshes = Vec::new();
        #[allow(clippy::identity_op)]
        models.into_iter().for_each(|model| {
            let mut vertices = Vec::new();
            let pos = &model.mesh.positions;
            (0..(pos.len() / 3)).for_each(|i| {
                vertices.push(Vertex {
                    position: glam::vec3(
                        (&model.mesh.positions)[i * 3 + 0],
                        (&model.mesh.positions)[i * 3 + 1],
                        (&model.mesh.positions)[i * 3 + 2],
                    ),
                    color_or_normal: glam::vec3(
                        (&model.mesh.normals)[i * 3 + 0],
                        (&model.mesh.normals)[i * 3 + 1],
                        (&model.mesh.normals)[i * 3 + 2],
                    ),
                    texture_uvs: glam::vec2(
                        (&model.mesh.texcoords)[i * 2 + 0],
                        (&model.mesh.texcoords)[i * 2 + 1],
                    ),
                });
            });
            meshes.push(Mesh::build(&vertices, &model.mesh.indices));
        });

        Ok(Model { meshes, texture, ..Default::default() })
    }

    pub fn render(&self, shader: &mut shader::Shader) {
        // Send the model transform
        shader.mat4_uniform(self.transform.to_matrix(), "model");
        // If model is textured, bind the texture
        if let Some(texture) = &self.texture {
            texture.use_texture();
        }
        self.meshes.iter().for_each(|mesh| {
            mesh.render();
        });
    }
}
