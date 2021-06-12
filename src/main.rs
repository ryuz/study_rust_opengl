
use std::mem;
use std::os::raw::c_void;
use std::time::Duration;

use cgmath::Array;
use cgmath::Matrix;
use cgmath::perspective;
use cgmath::prelude::SquareMatrix;

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


fn main()
{
    // メッシュ準備
    let mesh:Box<mesh_obj::Mesh<f32>>;
    let mesh_scale: f32;
    if true {
        mesh = mesh_obj::Mesh::<f32>::load("miku.obj").unwrap();
        mesh_scale = 10.0;
    }
    else {
       mesh = mesh_obj::Mesh::<f32>::load("unity_chan.obj").unwrap();
       mesh_scale = 300.0;
    }

    let window_width:u32 = 640;
    let window_height :u32 = 480;

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
    let fragment_shader = draw_gl::Shader::from_code(FRAGMENT_SHADER_CODE, gl::FRAGMENT_SHADER).unwrap();
    let program = draw_gl::Program::from_shaders(vec![vertes_shader, fragment_shader]).unwrap();

    let uniform_model = program.get_uniform_location("matrix_model");
    let uniform_view = program.get_uniform_location("matrix_view");
    let uniform_projection = program.get_uniform_location("matrix_projection");
    let uniform_color = program.get_uniform_location("color");

    let attrib_position = program.get_attrib_location("position");
    let attrib_normal = program.get_attrib_location("normal");
    let attrib_texcoord = program.get_attrib_location("texcoord");

    // 頂点バッファ転送
    let mut vbo: u32 = 0;
    unsafe {
        let mut vertex_array: Vec<f32> = mesh.get_vertex_array();

        gl::GenBuffers(1, &mut vbo);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertex_array.len() * mem::size_of::<f32>()) as isize,
            vertex_array.as_mut_ptr() as *mut c_void,
            gl::STATIC_DRAW,
        );

        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            attrib_position as u32,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 + 3 + 2) * 4,
            0 as *mut c_void,
        );
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 + 3 + 2) * 4,
            (3 * 4) as *mut c_void,
        );
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            (3 + 3 + 2) * 4,
            ((3 + 3) * 4) as *mut c_void,
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }


    let face_info = mesh.get_surface_info();
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
            gl::Viewport(0, 0, window_width as i32, window_height as i32);

            // clear screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);

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

            // shader use matrices
            program.use_program();

            gl::UniformMatrix4fv(uniform_model, 1, gl::FALSE, model_matrix.as_ptr());
            gl::UniformMatrix4fv(uniform_view, 1, gl::FALSE, view_matrix.as_ptr());
            gl::UniformMatrix4fv(uniform_projection, 1, gl::FALSE, projection_matrix.as_ptr());

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let mut array_index: i32 = 0;
            for it in face_info.iter() {
                let (array_size, material_index) = *it;
                let material = mesh.get_matrial(material_index);

                let color = Vector3 {
                    x: material.diffuse.x,
                    y: material.diffuse.y,
                    z: material.diffuse.z,
                };
                gl::Uniform3fv(uniform_color, 1, color.as_ptr());

                gl::DrawArrays(gl::TRIANGLES, array_index, (array_size * 3) as i32);
                array_index += array_size;
            }

            window.gl_swap_window();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    unsafe {
        gl::DeleteBuffers(1, &mut vbo);
    }
}


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
    vary_texcoord  = texcoord;

    vec3 frag_position = vec3(matrix_model * vec4(position, 1.0));
    gl_Position = matrix_projection * matrix_view * vec4(frag_position, 1.0);
}
"#;


const FRAGMENT_SHADER_CODE: &str = r#"
#version 100

varying lowp vec4 vary_color;
varying lowp vec3 vary_norm;
varying lowp vec2 vary_texcoord;

void main()
{
    gl_FragColor = vary_color;
    gl_FragColor[0] += vary_texcoord[0];
}
"#;
