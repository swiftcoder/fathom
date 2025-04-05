use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec4, vec4};
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlVertexArrayObject};

use crate::{
    compile_shader, link_program, polyline::polyline_to_triangles, reinterpret_cast_slice,
};

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Color {
    White,
    Yellow,
    PaleBlue,
}

impl Color {
    pub fn to_gl(&self) -> Vec4 {
        match self {
            Color::White => Vec4::ONE,
            Color::Yellow => vec4(1.0, 1.0, 0.0, 1.0),
            Color::PaleBlue => vec4(0.6, 0.6, 0.8, 1.0),
        }
    }
}

pub struct Scribe {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    vao: WebGlVertexArrayObject,
    buffer: WebGlBuffer,
    vertices: HashMap<Color, Vec<Vec2>>,
}

impl Scribe {
    pub fn new(context: &WebGl2RenderingContext) -> Self {
        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es
    
        uniform mat4 transform;

        in vec4 position;
    
        void main() {
            gl_Position = transform * vec4(position.xyz, 1.0);
        }
        "##,
        )
        .expect("vertex shader to compile");

        let frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es
    
        precision highp float;

        uniform vec4 color;

        out vec4 outColor;
        
        void main() {
            outColor = color; // vec4(1, 1, 1, 1);
        }
        "##,
        )
        .expect("fragment shader to compile");
        let program = link_program(&context, &vert_shader, &frag_shader).expect("program to link");
        context.use_program(Some(&program));

        let vao = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")
            .unwrap();
        context.bind_vertex_array(Some(&vao));

        let buffer = context
            .create_buffer()
            .ok_or("Failed to create buffer")
            .unwrap();
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        let position_attribute_location = context.get_attrib_location(&program, "position");
        context.vertex_attrib_pointer_with_i32(
            position_attribute_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            std::mem::size_of::<f32>() as i32 * 2,
            0,
        );
        context.enable_vertex_attrib_array(position_attribute_location as u32);

        Self {
            context: context.clone(),
            program,
            vao,
            buffer,
            vertices: HashMap::new(),
        }
    }

    pub fn draw_poly_line(&mut self, points: &[Vec2], width: f32, closed: bool, color: Color) {
        let vertices = polyline_to_triangles(points, width, 12, closed);
        self.vertices.entry(color).or_default().extend(&vertices);
    }

    pub fn render(&mut self, transform: Mat4) {
        self.context.bind_vertex_array(Some(&self.vao));
        self.context.use_program(Some(&self.program));

        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));

        for (color, vertices) in &self.vertices {
            unsafe {
                let positions_array_buf_view =
                    js_sys::Float32Array::view(reinterpret_cast_slice(&vertices));

                self.context.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    &positions_array_buf_view,
                    WebGl2RenderingContext::STATIC_DRAW,
                );
            }

            self.context.uniform_matrix4fv_with_f32_array(
                self.context
                    .get_uniform_location(&self.program, "transform")
                    .as_ref(),
                false,
                transform.as_ref(),
            );
            self.context.uniform4fv_with_f32_array(
                self.context
                    .get_uniform_location(&self.program, "color")
                    .as_ref(),
                &color.to_gl().to_array(),
            );

            self.context
                .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);
        }

        self.vertices.clear();
    }
}
