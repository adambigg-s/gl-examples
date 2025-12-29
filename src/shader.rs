use std::{collections as coll, ffi, fs, io, ptr};

#[derive(PartialEq)]
enum ShaderType {
    Shader,
    Program,
}

pub struct Shader {
    // Integer ID of the shader
    id: u32,

    // Integer locations of uniform values
    uniform_locations: coll::HashMap<&'static str, i32>,
}

impl Shader {
    pub fn build(vpath: &str, fpath: &str) -> Result<Self, io::Error> {
        // Read in the glsl shader source
        let vert = fs::read_to_string(vpath)?;
        let frag = fs::read_to_string(fpath)?;

        unsafe {
            // Create the shader programs
            let vshader = gl::CreateShader(gl::VERTEX_SHADER);
            let fshader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let program = gl::CreateProgram();

            // Compile the shaders and check for errors
            gl::ShaderSource(vshader, 1, &ffi::CString::new(vert)?.as_ptr(), ptr::null());
            gl::ShaderSource(fshader, 1, &ffi::CString::new(frag)?.as_ptr(), ptr::null());
            gl::CompileShader(vshader);
            gl::CompileShader(fshader);

            Self::check_errors(vshader, ShaderType::Shader)?;
            Self::check_errors(fshader, ShaderType::Shader)?;

            // Attach the shaders into a program and check for errors
            gl::AttachShader(program, vshader);
            gl::AttachShader(program, fshader);
            gl::LinkProgram(program);

            Self::check_errors(program, ShaderType::Program)?;

            // Delete the shaders after linking into a program
            gl::DeleteShader(vshader);
            gl::DeleteShader(fshader);

            Ok(Shader { id: program, uniform_locations: coll::HashMap::new() })
        }
    }

    pub fn mat4_uniform(&mut self, mat4: glam::Mat4, name: &'static str) {
        unsafe {
            if let Some(loc) = self.uniform_locations.get(name) {
                // If we have applied it already, reapply to the same location
                gl::UniformMatrix4fv(*loc, 1, gl::FALSE, mat4.to_cols_array().as_ptr());
            }
            else {
                // Otherwise, query and stash the location
                self.uniform_locations.insert(
                    name,
                    gl::GetUniformLocation(self.id, ffi::CString::new(name).unwrap().as_c_str().as_ptr()),
                );
                self.mat4_uniform(mat4, name);
            }
        }
    }

    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    fn check_errors(shader: u32, s_type: ShaderType) -> io::Result<()> {
        const LEN: usize = 1024;

        let mut success = Default::default();
        let mut failure_reason: [u8; _] = [Default::default(); LEN];
        unsafe {
            match s_type {
                | ShaderType::Shader => {
                    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
                    if success == Default::default() {
                        gl::GetShaderInfoLog(
                            shader,
                            LEN as i32,
                            ptr::null_mut(),
                            failure_reason.as_mut_ptr().cast(),
                        );
                        return Err(io::Error::other(String::from_utf8(failure_reason.to_vec()).unwrap()));
                    }
                }
                | ShaderType::Program => {
                    gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
                    if success == Default::default() {
                        gl::GetProgramInfoLog(
                            shader,
                            LEN as i32,
                            ptr::null_mut(),
                            failure_reason.as_mut_ptr().cast(),
                        );
                        return Err(io::Error::other(String::from_utf8(failure_reason.to_vec()).unwrap()));
                    }
                }
            }
        }

        Ok(())
    }
}
