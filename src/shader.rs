use web_sys::{WebGl2RenderingContext, WebGlProgram};

use crate::{compile_shader, link_program, texture::Texture};

pub struct Shader {
    context: WebGl2RenderingContext,
    pub program: WebGlProgram,
}

impl Shader {
    pub fn new(context: &WebGl2RenderingContext, vertex: &str, fragment: &str) -> Self {
        let program = link_program(
            context,
            &compile_shader(context, WebGl2RenderingContext::VERTEX_SHADER, vertex)
                .expect("compiled vertex shader"),
            &compile_shader(context, WebGl2RenderingContext::FRAGMENT_SHADER, fragment)
                .expect("compiled fragment shader"),
        )
        .expect("linked shader program");

        Self {
            context: context.clone(),
            program,
        }
    }

    pub fn bind_texture(&self, name: &str, unit: u32, texture: &Texture) {
        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0 + unit);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture.texture));

        self.context.use_program(Some(&self.program));
        self.context.uniform1i(
            self.context
                .get_uniform_location(&self.program, name)
                .as_ref(),
            unit as i32,
        );
    }

    pub fn uniform1f(&self, name: &str, value: f32) {
        self.context.use_program(Some(&self.program));
        self.context.uniform1f(
            self.context
                .get_uniform_location(&self.program, name)
                .as_ref(),
            value,
        );
    }
}
