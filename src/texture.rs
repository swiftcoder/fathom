use web_sys::{WebGl2RenderingContext, WebGlTexture};

pub struct Texture {
    context: WebGl2RenderingContext,
    pub texture: WebGlTexture,
}

impl Texture {
    pub fn new(context: &WebGl2RenderingContext, w: i32, h: i32) -> Self {
        let texture = context.create_texture().expect("failed to create texture");

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            w,
            h,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        ).expect("failed to allocate texture backing");
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Self {
            context: context.clone(),
            texture,
        }
    }

    pub fn resize(&self, w: i32, h: i32) {
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        self.context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                w,
                h,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                None,
            )
            .expect("failed to resize texture");
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }
}
