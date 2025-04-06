use glam::{Mat4, Vec2, vec2};
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{
    font::{Character, Font, Segment},
    reinterpret_cast_slice,
    shader::Shader,
    texture::Texture,
};

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
    segment_offset: u16,
    len: u16,
}

pub enum Align {
    Left,
    Center,
    Right,
}

// Implements https://www.shadertoy.com/view/sdXBDs for anti-aliased GPU-evaluated quadratic bezier text
pub struct Text {
    context: WebGl2RenderingContext,
    font: Font,
    segments: Vec<Segment>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    texture: Texture,
    shader: Shader,
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    vao: WebGlVertexArrayObject,
}

impl Text {
    pub fn new(context: &WebGl2RenderingContext, font: Font) -> Self {
        let texture = Texture::new(
            context,
            1,
            1,
            WebGl2RenderingContext::RGB32F,
            WebGl2RenderingContext::FLOAT,
        );

        let shader = Shader::new(
            context,
            r#"#version 300 es
        layout(location=0) in vec2 position;
        layout(location=1) in vec2 uv;
        layout(location=2) in uvec2 path;

        uniform mat4 MVPmatrix;

        out vec2 v_uv;
        flat out uvec2 v_path;
    
        void main(void) {
            v_uv = uv;
            v_path = path;
            gl_Position = MVPmatrix * vec4(position, 0.0, 1.0);
        }
        "#,
            r#"#version 300 es
        precision highp float;
        
        uniform sampler2D pathSampler;

        layout(location=0) out vec4 fragColor;
        
        in vec2 v_uv;
        flat in uvec2 v_path;

        // evaluate only the x coordinate of the bezier specified by the control points
        float eval_bezier_x(float t, vec2 p1, vec2 p2, vec2 p3) {
            float s = 1.0 - t;
            return s * s * p1.x + 2.0 * s * t * p2.x + t * t * p3.x;
        }

        // returns the x coordinate of the intersection beteen a horizontal line at y=b, and the
        // bezier specified by the control points.
        float bezier_t_intersect_horizontal_line(float y, vec2 p1, vec2 p2, vec2 p3) {    
            float a = p1.y - 2.0*p2.y + p3.y;

            
            // bezier is a straight line, so we can calculate the intersection directly
            if (abs(a) < 0.0002) {
                return (y - p1.y) / (p3.y - p1.y);
            }

            float q = p1.y - p2.y + sqrt(y*a + p2.y*p2.y - p1.y*p3.y);
            float ta = q / a;
            float tb = (p1.y - y) / q;
            return (0.0 <= ta && ta <= 1.0) ? ta : tb;
        }

        void main(void) {
            vec2 uv = v_uv;

            vec2 ddx = dFdx(uv);
            vec2 ddy = dFdy(uv);
            vec2 pixel_footprint = sqrt(ddx * ddx + ddy * ddy);
        
            float coverage = 0.0;

            for (uint i = v_path.x; i < v_path.x + v_path.y; ++i) {
                vec3 v0 = texelFetch(pathSampler, ivec2(0, i), 0).rgb;
                vec3 v1 = texelFetch(pathSampler, ivec2(1, i), 0).rgb;

                vec2 p1 = v0.xy;
                vec2 p2 = vec2(v0.z, v1.x);
                vec2 p3 = v1.yz;

                // compute the overlap between pixel footprint and bezier in the y-axis
                vec2 footprint_y = uv.y + vec2(-0.5, 0.5) * pixel_footprint.y;
                vec2 window_y = clamp(vec2(p3.y, p1.y), footprint_y.x, footprint_y.y);
                float overlap_y = (window_y.y - window_y.x) / pixel_footprint.y;

                // no overlap, we're done here
                if (overlap_y != 0.0) {
                    // grab the intersection in terms of t as well as x
                    float t = bezier_t_intersect_horizontal_line(0.5 * (window_y.x + window_y.y), p1, p2, p3);
                    float x = eval_bezier_x(t, p1, p2, p3);

                    // use the tangent at t to estimate overlap in the x-axis 
                    vec2 tangent = mix(p2 - p1, p3 - p2, t);
                    float f = ((x - uv.x) * abs(tangent.y)) / length(pixel_footprint * tangent.yx);
                    float overlap_x = clamp(0.5 + 0.7 * f, 0.0, 1.0);

                    // sum up the overlap from each curve
                    coverage += overlap_x * overlap_y;
                }
            }

            fragColor = vec4(sqrt(coverage));
        }
        "#,
        );
        context.use_program(Some(&shader.program));

        let vao = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")
            .unwrap();
        context.bind_vertex_array(Some(&vao));

        let vertex_buffer = context.create_buffer().expect("created buffer");
        let index_buffer = context.create_buffer().expect("created buffer");

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &[],
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        let position_attribute_location = context.get_attrib_location(&shader.program, "position");
        context.vertex_attrib_pointer_with_i32(
            position_attribute_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            0,
        );
        context.enable_vertex_attrib_array(position_attribute_location as u32);

        let uv_attribute_location: i32 = context.get_attrib_location(&shader.program, "uv");
        context.vertex_attrib_pointer_with_i32(
            uv_attribute_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            std::mem::offset_of!(Vertex, uv) as i32,
        );
        context.enable_vertex_attrib_array(uv_attribute_location as u32);

        let path_attribute_location: i32 = context.get_attrib_location(&shader.program, "path");
        context.vertex_attrib_i_pointer_with_i32(
            path_attribute_location as u32,
            2,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            std::mem::size_of::<Vertex>() as i32,
            std::mem::offset_of!(Vertex, segment_offset) as i32,
        );
        context.enable_vertex_attrib_array(path_attribute_location as u32);

        context.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&index_buffer),
        );
        context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &[],
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        context.bind_vertex_array(None);

        Self {
            context: context.clone(),
            font,
            segments: vec![],
            vertices: vec![],
            indices: vec![],
            texture,
            shader,
            vertex_buffer,
            index_buffer,
            vao,
        }
    }

    pub fn measure(&self, font_size: f32, text: &str) -> Vec2 {
        let scale = font_size / self.font.units_per_em;

        vec2(self.compute_string_width(text), self.font.height) * scale
    }

    pub fn draw(&mut self, x: f32, y: f32, font_size: f32, align: Align, text: &str) {
        let width = self.compute_string_width(text);

        let scale = font_size / self.font.units_per_em;
        let descent = self.font.descender as f32 * scale;

        let offset_x = match align {
            Align::Left => 0.0,
            Align::Center => -width * scale / 2.0,
            Align::Right => -width * scale,
        };
        let mut offset = vec2(x + offset_x, y - descent);

        for c in text.chars() {
            if let Some(Character { advance, path }) = self.font.chars.get(&c) {
                let segment_offset = self.segments.len() as u16;
                let len = path.segments.len() as u16;

                self.segments.extend(path.segments.iter());

                let i = self.vertices.len() as u16;
                self.indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);

                const DILATE: Vec2 = vec2(0.5, 0.5);

                let p = offset + path.offset * scale - DILATE;
                let q = offset + (path.offset + path.size) * scale + DILATE;

                let d = DILATE / (path.size * scale);

                self.vertices.extend(&[
                    Vertex {
                        pos: p,
                        uv: vec2(-d.x, -d.y),
                        segment_offset,
                        len,
                    },
                    Vertex {
                        pos: vec2(q.x, p.y),
                        uv: vec2(1.0 + d.x, -d.y),
                        segment_offset,
                        len,
                    },
                    Vertex {
                        pos: q,
                        uv: vec2(1.0 + d.x, 1.0 + d.y),
                        segment_offset,
                        len,
                    },
                    Vertex {
                        pos: vec2(p.x, q.y),
                        uv: vec2(-d.x, 1.0 + d.y),
                        segment_offset,
                        len,
                    },
                ]);

                offset.x += *advance * scale;
            }
        }
    }

    pub fn render(&mut self, transform: Mat4) {
        // log::info!("segments size {}", self.segments.len() * std::mem::size_of::<Segment>());
        self.texture.write(
            2,
            self.segments.len() as i32,
            Some(reinterpret_cast_slice::<Segment, u8>(&self.segments)),
        );

        self.context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.vertex_buffer),
        );
        self.context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            reinterpret_cast_slice(&self.vertices),
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        self.context.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.index_buffer),
        );
        self.context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            reinterpret_cast_slice(&self.indices),
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        self.context.bind_vertex_array(Some(&self.vao));
        self.context.use_program(Some(&self.shader.program));

        self.shader.bind_texture("pathSampler", 0, &self.texture);
        self.shader.uniform_matrix4("MVPmatrix", transform);

        self.context.enable(WebGl2RenderingContext::BLEND);
        self.context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        self.context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.indices.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );

        self.context.disable(WebGl2RenderingContext::BLEND);

        // log::info!("text with {} segments, {} vertices, and {} indices", self.segments.len(), self.vertices.len(), self.indices.len());

        self.segments.clear();
        self.vertices.clear();
        self.indices.clear();
    }

    fn compute_string_width(&self, text: &str) -> f32 {
        text.chars()
            .map(|c| self.font.chars.get(&c).map(|q| q.advance).unwrap_or(0.0))
            .sum()
    }
}
