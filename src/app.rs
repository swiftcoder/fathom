use glam::{Mat4, vec2};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram};

use crate::{
    compile_shader, document, link_program, polyline::polyline_to_triangles, reinterpret_cast_slice,
};

pub struct AppState {
    program: WebGlProgram,
    vert_count: i32,
}

impl AppState {
    pub fn new(context: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es
    
        uniform mat4 transform;

        in vec4 position;
    
        void main() {
            gl_Position = transform * vec4(position.xyz, 1.0);
            // gl_Position = position;
        }
        "##,
        )?;

        let frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es
    
        precision highp float;
        out vec4 outColor;
        
        void main() {
            outColor = vec4(1, 1, 1, 1);
        }
        "##,
        )?;
        let program = link_program(&context, &vert_shader, &frag_shader)?;
        context.use_program(Some(&program));

        let vertices = [vec2(-7.0, -7.0), vec2(7.0, -7.0), vec2(0.0, 7.0)];
        let vertices = polyline_to_triangles(&vertices, 1.0, 8, true);

        let position_attribute_location = context.get_attrib_location(&program, "position");
        let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let positions_array_buf_view =
                js_sys::Float32Array::view(reinterpret_cast_slice(&vertices));

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vao = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        context.bind_vertex_array(Some(&vao));

        context.vertex_attrib_pointer_with_i32(
            position_attribute_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            std::mem::size_of::<f32>() as i32 * 2,
            0,
        );
        context.enable_vertex_attrib_array(position_attribute_location as u32);

        context.bind_vertex_array(Some(&vao));

        Ok(Self {
            program,
            vert_count: vertices.len() as i32,
        })
    }

    pub fn on_resize(&self, canvas: &HtmlCanvasElement, context: &WebGl2RenderingContext) {
        let document = document();
        let w = document.body().unwrap().client_width();
        let h = document.body().unwrap().client_height();
        log::info!("resizing {} {}", w, h);
        canvas.set_width(w as u32);
        canvas.set_height(h as u32);
        context.viewport(0, 0, w, h);
    }

    pub fn draw(&self, context: &WebGl2RenderingContext) {
        let document = document();
        let aspect = document.body().unwrap().client_width() as f32
            / document.body().unwrap().client_height() as f32;

        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        context.use_program(Some(&self.program));

        let m = Mat4::orthographic_rh_gl(-10.0 * aspect, 10.0 * aspect, -10.0, 10.0, -10.0, 10.0);
        let transform_loc = context.get_uniform_location(&self.program, "transform");
        context.uniform_matrix4fv_with_f32_array(transform_loc.as_ref(), false, m.as_ref());

        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, self.vert_count);
    }
}
