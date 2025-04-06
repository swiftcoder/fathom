use crate::{float_32_array, reinterpret_cast_slice};
use web_sys::{WebGl2RenderingContext, WebGlTexture};

pub struct Texture {
    context: WebGl2RenderingContext,
    pub texture: WebGlTexture,
    format: u32,
    format2: u32,
    _type: u32,
}

impl Texture {
    pub fn new(context: &WebGl2RenderingContext, w: i32, h: i32, format: u32, _type: u32) -> Self {
        let texture = context.create_texture().expect("failed to create texture");

        let format2 = match format {
            WebGl2RenderingContext::RGB32F => WebGl2RenderingContext::RGB,
            _ => WebGl2RenderingContext::RGBA,
        };

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                format as i32,
                w,
                h,
                0,
                format2,
                _type,
                None,
            )
            .expect("failed to allocate texture backing");
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Self {
            context: context.clone(),
            texture,
            format,
            format2,
            _type,
        }
    }

    pub fn write(&self, w: i32, h: i32, data: Option<&[u8]>) {
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        match self._type {
            WebGl2RenderingContext::FLOAT if data.is_some() => {
                let src = data.unwrap_or(&[]);
                let dst = reinterpret_cast_slice::<u8, f32>(src);
                // log::info!("dst len {} src len {} w*h {}", src.len(), dst.len()*4, w*h*3*4);

                self.context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                self.format as i32,
                w as i32,
                h as i32,
                0,
                self.format2,
                self._type,
                &float_32_array!(dst).into(),
                0,
            )
            .expect("failed to resize texture")
            }
            _ => self
                .context
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    self.format as i32,
                    w,
                    h,
                    0,
                    self.format2,
                    self._type,
                    data,
                )
                .expect("failed to resize texture"),
        }
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }
}

#[macro_export]
macro_rules! float_32_array {
    ($arr:expr) => {{
        use js_sys::WebAssembly;
        use wasm_bindgen::JsCast;

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .expect("memory to WASM")
            .buffer();
        let arr_location = $arr.as_ptr() as u32 / 4;
        let array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + $arr.len() as u32);
        array
    }};
}
