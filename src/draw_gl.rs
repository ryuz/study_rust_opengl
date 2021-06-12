use gl;
use gl::types::*;
use std::ffi::CString;
use std::ptr;
//use std::str;

#[allow(dead_code)]
type Vector3 = cgmath::Vector3<f32>;
#[allow(dead_code)]
type Matrix4 = cgmath::Matrix4<f32>;

// Shader
pub struct Shader {
    pub shader: GLuint,
}

#[allow(dead_code)]
impl Shader {
    pub fn new(type_: GLenum) -> Self {
        let mut shader = Shader { shader: 0 };
        unsafe {
            shader.shader = gl::CreateShader(type_);
        }
        shader
    }
    
    pub fn get(&self) -> GLuint {
        self.shader
    }

    fn get_iv(&self, pname: GLuint) -> GLint {
        let mut param: GLint = 0;
        unsafe {
            gl::GetShaderiv(self.get(), pname, &mut param);
        }
        param
    }

    pub fn source(&self, shader_code: &str) {
        let cstr_shader_code = CString::new(shader_code.as_bytes()).unwrap();
        unsafe {
            gl::ShaderSource(self.get(), 1, &cstr_shader_code.as_ptr(), ptr::null());
        }
    }

    pub fn compile(&self) {
        unsafe {
            gl::CompileShader(self.get());
        }
    }

    pub fn get_info_log(&self) -> String {
        // ログ領域確保
        let max_length: GLsizei = self.get_iv(gl::INFO_LOG_LENGTH) as GLsizei;
        let mut info_log = Vec::with_capacity(max_length as usize + 1);

        let mut log_length: GLsizei = 0;
        unsafe {
            info_log.set_len(max_length as usize);
            gl::GetShaderInfoLog(
                self.get(),
                max_length,
                &mut log_length,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            info_log.set_len(log_length as usize);
        }

        String::from_utf8(info_log).unwrap()
    }

    pub fn from_code(shader_code: &str, type_: GLenum) -> Result<Self, String>
    {
        let shader = Shader::new(type_);
        shader.source(shader_code);
        shader.compile();
        let status = shader.get_iv(gl::COMPILE_STATUS);
        if status == gl::TRUE as GLint {
            Ok(shader)
        }
        else {
            Err(shader.get_info_log())
        }
    }
}

impl Drop for Shader {
    #[allow(dead_code)]
    fn drop(&mut self) {
        if self.shader > 0 {
            unsafe {
                gl::DeleteShader(self.shader);
            }
        }
    }
}


// Program
pub struct Program {
    pub program: GLuint,
}

#[allow(dead_code)]
impl Program {
    pub fn new() -> Self {
        let mut program = Program { program: 0 };
        unsafe {
            program.program = gl::CreateProgram();
        }
        program
    }

    pub fn get(&self) -> GLuint
    {
        self.program
    }

    fn get_iv(&self, pname: GLuint) -> GLint {
        let mut param: GLint = 0;
        unsafe {
            gl::GetShaderiv(self.get(), pname, &mut param);
        }
        param
    }

    pub fn attach_shader(&self, shader: Shader)
    {
        unsafe {
            gl::AttachShader(self.get(), shader.get());
        }
    }

    pub fn link(&self)
    {
        unsafe {
            gl::LinkProgram(self.get());
        }
    }
    
    pub fn get_info_log(&self) -> String {
        // ログ領域確保
        let max_length: GLsizei = self.get_iv(gl::INFO_LOG_LENGTH) as GLsizei;
        let mut info_log = Vec::with_capacity(max_length as usize + 1);

        let mut log_length: GLsizei = 0;
        unsafe {
            info_log.set_len(max_length as usize);
            gl::GetShaderInfoLog(
                self.get(),
                max_length,
                &mut log_length,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            info_log.set_len(log_length as usize);
        }

        String::from_utf8(info_log).unwrap()
    }


    pub fn from_shaders(shaders: Vec<Shader>) -> Result<Self, String>
    {
        let program = Program::new();
        for shader in shaders {
            program.attach_shader(shader);
        }
        program.link();
        let status = program.get_iv(gl::LINK_STATUS);
        if status == gl::TRUE as GLint {
            Ok(program)
        }
        else {
            Err(program.get_info_log())
        }
    }
}


