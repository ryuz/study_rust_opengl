use gl;
use gl::types::*;
use image::DynamicImage;
use image::RgbaImage;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

// Shader
pub struct Shader {
    shader: GLuint,
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

    pub fn from_code(shader_code: &str, type_: GLenum) -> Result<Self, String> {
        let shader = Shader::new(type_);
        shader.source(shader_code);
        shader.compile();
        let status = shader.get_iv(gl::COMPILE_STATUS);
        if status == gl::TRUE as GLint {
            Ok(shader)
        } else {
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
    program: GLuint,
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

    pub fn get(&self) -> GLuint {
        self.program
    }

    fn get_iv(&self, pname: GLuint) -> GLint {
        let mut param: GLint = 0;
        unsafe {
            gl::GetProgramiv(self.get(), pname, &mut param);
        }
        param
    }

    pub fn attach_shader(&self, shader: Shader) {
        unsafe {
            gl::AttachShader(self.get(), shader.get());
        }
    }

    pub fn link(&self) {
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
            gl::GetProgramInfoLog(
                self.get(),
                max_length,
                &mut log_length,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            info_log.set_len(log_length as usize);
        }

        String::from_utf8(info_log).unwrap()
    }

    pub fn from_shaders(shaders: Vec<Shader>) -> Result<Self, String> {
        let program = Program::new();
        for shader in shaders {
            program.attach_shader(shader);
        }
        program.link();
        let status = program.get_iv(gl::LINK_STATUS);
        if status == gl::TRUE as GLint {
            Ok(program)
        } else {
            Err(program.get_info_log())
        }
    }

    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.get()) }
    }

    pub fn get_uniform_location(&self, name: &str) -> GLint {
        let cstr_name = CString::new(name.as_bytes()).unwrap();
        unsafe { gl::GetUniformLocation(self.get(), cstr_name.as_ptr()) }
    }

    pub fn get_attrib_location(&self, name: &str) -> GLint {
        let cstr_name = CString::new(name.as_bytes()).unwrap();
        unsafe { gl::GetAttribLocation(self.get(), cstr_name.as_ptr()) }
    }
}

impl Drop for Program {
    #[allow(dead_code)]
    fn drop(&mut self) {
        if self.program > 0 {
            unsafe {
                gl::DeleteProgram(self.program);
            }
        }
    }
}

pub struct Textur {
    texture: GLuint,
}

#[allow(dead_code)]
impl Textur {
    pub fn new() -> Self {
        let mut texture = Textur { texture: 0 };
        unsafe {
            gl::GenTextures(1, &mut texture.texture);
        }
        texture
    }

    pub fn get(&self) -> GLuint {
        self.texture
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.get());
        }
    }

    pub fn loda_image_rgba(img: &RgbaImage) -> Self {
        let texture = Textur::new();
        unsafe {
            texture.bind();
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_ptr() as *const c_void,
            );
        }
        texture
    }

    pub fn loda_image(img: &DynamicImage) -> Self {
        let img = img.to_rgba8();
        Textur::loda_image_rgba(&img)
    }
}

impl Drop for Textur {
    #[allow(dead_code)]
    fn drop(&mut self) {
        if self.texture > 0 {
            unsafe {
                gl::DeleteTextures(1, &mut self.texture);
            }
        }
    }
}

pub struct Texturs {
    textures: HashMap<String, Textur>,
}

#[allow(dead_code)]
impl Texturs {
    pub fn new() -> Self {
        Texturs {
            textures: HashMap::<String, Textur>::new(),
        }
    }

    pub fn get(&self, filename: &str) -> &Textur {
        self.textures.get(&filename.to_string()).unwrap()
    }

    pub fn load_file(&mut self, filename: &str) {
        let name = filename.to_string();
        match self.textures.get(&name) {
            None => {
                let img = image::io::Reader::open(&name).unwrap().decode().unwrap();
                let texture = Textur::loda_image(&img);
                self.textures.insert(name, texture);
            }
            Some(_) => {}
        }
    }
}

pub struct VertexArrayBuffer {
    vbo: GLuint,
}

#[allow(dead_code)]
impl VertexArrayBuffer {
    pub fn new() -> Self {
        let mut vertex_array = VertexArrayBuffer { vbo: 0 };
        unsafe {
            gl::GenBuffers(1, &mut vertex_array.vbo);
        }
        vertex_array
    }

    pub fn bind_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }
    }

    pub fn buffer_data_f32(&self, vertex_array: &Vec<f32>, usage: GLenum) {
        self.bind_buffer();
        unsafe {
            let unit_szie = std::mem::size_of::<f32>();
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_array.len() * unit_szie) as isize,
                vertex_array.as_ptr() as *mut c_void,
                usage,
            );
        }
    }

    pub fn vertex_attrib_pointer(
        &self,
        index: GLint,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        offset: GLsizei,
    ) {
        if index >= 0 {
            self.bind_buffer();
            unsafe {
                gl::VertexAttribPointer(
                    index as GLuint,
                    size,
                    type_,
                    gl::FALSE,
                    stride,
                    offset as *mut c_void,
                );
                gl::EnableVertexAttribArray(index as GLuint);
            }
        }
    }

    /*
    pub fn enable_vertex_attrib_array(&self, index: GLint) {
        if index >= 0 {
            self.bind_buffer();
            unsafe {
                gl::EnableVertexAttribArray(index as GLuint);
            }
        }
    }
    */
}

impl Drop for VertexArrayBuffer {
    #[allow(dead_code)]
    fn drop(&mut self) {
        if self.vbo > 0 {
            unsafe {
                gl::DeleteBuffers(1, &mut self.vbo);
            }
        }
    }
}
