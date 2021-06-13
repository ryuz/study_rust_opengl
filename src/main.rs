use std::time::Duration;

use cgmath::perspective;
use cgmath::prelude::SquareMatrix;
use cgmath::Array;
use cgmath::Matrix;

use gl::types::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod draw_gl;
mod mesh_obj;

//#[allow(dead_code)]
type Point3 = cgmath::Point3<f32>;
#[allow(dead_code)]
type Vector3 = cgmath::Vector3<f32>;
#[allow(dead_code)]
type Matrix4 = cgmath::Matrix4<f32>;

fn main() {
    // メッシュ準備
    let mesh: Box<mesh_obj::Mesh<f32>>;
    let mesh_scale: f32;
    if false {
        mesh = mesh_obj::Mesh::<f32>::load("miku.obj").unwrap();
        mesh_scale = 10.0;
    } else {
        mesh = mesh_obj::Mesh::<f32>::load("unity_chan.obj").unwrap();
        mesh_scale = 300.0;
    }

    let window_width: u32 = 640;
    let window_height: u32 = 480;

    // SDL open
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // OpenGL ES 2.0 を使う
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
    gl_attr.set_context_version(2, 0);

    // Window open
    let window = video_subsystem
        .window("Study OpenGL", window_width, window_height)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

    // シェーダー準備
    let vertes_shader = draw_gl::Shader::from_code(VERTEX_SHADER_CODE, gl::VERTEX_SHADER).unwrap();
    let fragment_shader =
        draw_gl::Shader::from_code(FRAGMENT_SHADER_CODE, gl::FRAGMENT_SHADER).unwrap();
    let program = draw_gl::Program::from_shaders(vec![vertes_shader, fragment_shader]).unwrap();

    let uniform_model = program.get_uniform_location("matrix_model");
    let uniform_view = program.get_uniform_location("matrix_view");
    let uniform_projection = program.get_uniform_location("matrix_projection");
    let uniform_color = program.get_uniform_location("color");
    let uniform_texture_sampler = program.get_uniform_location("texture_sampler");
    let uniform_texture_enable = program.get_uniform_location("texture_enable");

    let attrib_position = program.get_attrib_location("position");
    let attrib_normal = program.get_attrib_location("normal");
    let attrib_texcoord = program.get_attrib_location("texcoord");


    // 頂点バッファ転送
    let vertex_array_buffer = draw_gl::VertexArrayBuffer::new();
    vertex_array_buffer.buffer_data_f32(&mesh.get_vertex_array(), gl::STATIC_DRAW);
    vertex_array_buffer.vertex_attrib_pointer(attrib_position, 3, gl::FLOAT, 32, 0);
    vertex_array_buffer.vertex_attrib_pointer(attrib_normal, 3, gl::FLOAT, 32, 12);
    vertex_array_buffer.vertex_attrib_pointer(attrib_texcoord, 2, gl::FLOAT, 32, 24);

    // テクスチャロード
    let face_info = mesh.get_surface_info();
    let mut textures = draw_gl::Texturs::new();
    for (_, material_index) in &face_info {
        let material = mesh.get_matrial(*material_index);
        if !material.diffuse_filename.is_empty() {
            textures.load_file(&material.diffuse_filename);
        }
    }

    // 描画ループ
    let mut look_direction: f32 = 0.0f32;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        unsafe {
            // バッファ初期化
            gl::Viewport(0, 0, window_width as i32, window_height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);

            // 視点等設定
            look_direction += 0.1;
            let model_matrix = Matrix4::identity();
            let view_matrix = Matrix4::look_at_rh(
                Point3 {
                    x: look_direction.sin() * mesh_scale,
                    y: 1.0,
                    z: look_direction.cos() * mesh_scale,
                },
                Point3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            );

            let projection_matrix: Matrix4 = perspective(
                cgmath::Deg(45.0f32),
                window_width as f32 / window_height as f32,
                0.1 * mesh_scale,
                100.0 * mesh_scale,
            );

            // シェーダー設定
            program.use_program();
            gl::UniformMatrix4fv(uniform_model, 1, gl::FALSE, model_matrix.as_ptr());
            gl::UniformMatrix4fv(uniform_view, 1, gl::FALSE, view_matrix.as_ptr());
            gl::UniformMatrix4fv(uniform_projection, 1, gl::FALSE, projection_matrix.as_ptr());
            gl::Uniform1i(uniform_texture_sampler, 0);

            //            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            vertex_array_buffer.bind_buffer();

            let mut array_index: i32 = 0;
            for (array_size, material_index) in &face_info {
                let material = mesh.get_matrial(*material_index);

                let texture_enable = !&material.diffuse_filename.is_empty();
                gl::Uniform1i(uniform_texture_enable, if texture_enable { 1 } else { 0 });

                if texture_enable {
                    // テクスチャがあればバインド
                    gl::ActiveTexture(gl::TEXTURE0);
                    textures.get(&material.diffuse_filename).bind_texture();
                    gl::TexParameteri(gl::TEXTURE_2D, gl::AUTO_GENERATE_MIPMAP, gl::TRUE as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
                }

                // 色設定
                let color = Vector3 {
                    x: material.diffuse.x,
                    y: material.diffuse.y,
                    z: material.diffuse.z,
                };
                gl::Uniform3fv(uniform_color, 1, color.as_ptr());

                // 描画
                gl::DrawArrays(gl::TRIANGLES, array_index, (array_size * 3) as i32);
                array_index += array_size;
            }

            // バッファスワップ
            window.gl_swap_window();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}


// バーテックスシェーダー
const VERTEX_SHADER_CODE: &str = r#"
#version 100 

attribute vec3 position;
attribute vec3 normal;
attribute vec2 texcoord;

uniform mat4 matrix_model;
uniform mat4 matrix_view;
uniform mat4 matrix_projection;
uniform vec3 color;

varying lowp vec4 vary_color;
varying lowp vec3 vary_norm;
varying lowp vec2 vary_texcoord;

void main()
{
    vary_color = vec4(color.x, color.y, color.z, 1);
    vary_norm  = normal;
    vary_texcoord = texcoord;

    vec3 frag_position = vec3(matrix_model * vec4(position, 1.0));
    gl_Position = matrix_projection * matrix_view * vec4(frag_position, 1.0);
}
"#;

// フラグメントシェーダー
const FRAGMENT_SHADER_CODE: &str = r#"
#version 100

uniform sampler2D texture_sampler;
uniform bool texture_enable;

varying lowp vec4 vary_color;
varying lowp vec3 vary_norm;
varying lowp vec2 vary_texcoord;

void main()
{
    if ( texture_enable ) {
        gl_FragColor = texture2D(texture_sampler, vary_texcoord);
    }
    else {
        gl_FragColor = vary_color;
    }
}
"#;
