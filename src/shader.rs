use std::{ffi, fs, io, ptr};

#[derive(PartialEq)]
enum ShaderType {
    Shader,
    Program,
}

pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn build(vpath: &str, fpath: &str) -> Result<Self, io::Error> {
        let vert = fs::read_to_string(vpath)?;
        let frag = fs::read_to_string(fpath)?;

        unsafe {
            let vshader = gl::CreateShader(gl::VERTEX_SHADER);
            let fshader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let program = gl::CreateProgram();

            gl::ShaderSource(vshader, 1, &ffi::CString::new(vert)?.as_ptr(), ptr::null());
            gl::ShaderSource(fshader, 1, &ffi::CString::new(frag)?.as_ptr(), ptr::null());
            gl::CompileShader(vshader);
            gl::CompileShader(fshader);
            Self::check_errors(vshader, ShaderType::Shader)?;
            Self::check_errors(fshader, ShaderType::Shader)?;

            gl::AttachShader(program, vshader);
            gl::AttachShader(program, fshader);
            gl::LinkProgram(program);
            Self::check_errors(program, ShaderType::Program)?;

            gl::DeleteShader(vshader);
            gl::DeleteShader(fshader);

            Ok(Shader { id: program })
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
        let mut why: [u8; _] = [Default::default(); LEN];
        unsafe {
            match s_type {
                | ShaderType::Shader => {
                    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
                    if success == 0 {
                        gl::GetShaderInfoLog(shader, LEN as i32, ptr::null_mut(), why.as_mut_ptr().cast());
                        return Err(io::Error::other(String::from_utf8_unchecked(why.to_vec())));
                    }
                }
                | ShaderType::Program => {
                    gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
                    if success == 0 {
                        gl::GetProgramInfoLog(shader, LEN as i32, ptr::null_mut(), why.as_mut_ptr().cast());
                        return Err(io::Error::other(String::from_utf8_unchecked(why.to_vec())));
                    }
                }
            }
        }

        Ok(())
    }
}
