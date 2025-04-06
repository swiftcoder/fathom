use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlFramebuffer};

use crate::{shader::Shader, texture::Texture};

pub struct PostProcessor {
    context: WebGl2RenderingContext,
    scene_fbo: WebGlFramebuffer,
    scene_texture: Texture,
    ping_fbo: WebGlFramebuffer,
    ping_texture: Texture,
    pong_fbo: WebGlFramebuffer,
    pong_texture: Texture,
    threshold_shader: Shader,
    blur_shader_h: Shader,
    blur_shader_v: Shader,
    composite_shader: Shader,
    crt_shader: Shader,
    w: i32,
    h: i32,
}

impl PostProcessor {
    pub fn new(context: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        let scene_texture = Texture::new(context, 1, 1);
        let ping_texture = Texture::new(context, 1, 1);
        let pong_texture = Texture::new(context, 1, 1);

        let scene_fbo = context
            .create_framebuffer()
            .ok_or("failed to create framebuffer")?;
        context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&scene_fbo));
        context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&scene_texture.texture),
            0,
        );
        log::info!(
            "framebuffer status {}",
            context.check_framebuffer_status(WebGl2RenderingContext::FRAMEBUFFER)
                == WebGl2RenderingContext::FRAMEBUFFER_COMPLETE
        );

        let ping_fbo = context
            .create_framebuffer()
            .ok_or("failed to create framebuffer")?;
        context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&ping_fbo));
        context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&ping_texture.texture),
            0,
        );

        let pong_fbo = context
            .create_framebuffer()
            .ok_or("failed to create framebuffer")?;
        context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&pong_fbo));
        context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&pong_texture.texture),
            0,
        );

        let fullscreen_quad_vs = r##"#version 300 es
    
        in vec4 position;

        out vec2 v_uv;

        void main() {
            // Hardcoded positions for a fullscreen quad in clip space
            // (x, y) pairs: (-1,-1), (1,-1), (-1,1), (1,1)
            vec2 positions[4] = vec2[](
                vec2(-1.0, -1.0),
                vec2( 1.0, -1.0),
                vec2(-1.0,  1.0),
                vec2( 1.0,  1.0)
            );

            vec2 pos = positions[gl_VertexID];
            v_uv = pos * 0.5 + 0.5;
            gl_Position = vec4(pos, 0.0, 1.0);
        }
        "##;

        let threshold_shader = Shader::new(
            context,
            fullscreen_quad_vs,
            r##"#version 300 es
    
        precision highp float;

        uniform sampler2D u_texture;
        uniform float u_texel_width;

        in vec2 v_uv;

        out vec4 outColor;
        
        void main() {
            vec3 color = texture(u_texture, v_uv).rgb;
            float brightness = max(max(color.r, color.g), color.b); // simple luminance
            vec3 bloom = brightness > 0.5 ? color : vec3(0.0);
            outColor = vec4(bloom, 1.0);
        }
        "##,
        );

        let blur_shader_h = Shader::new(
            context,
            fullscreen_quad_vs,
            r##"#version 300 es
    
        precision highp float;

        uniform sampler2D u_texture;
        uniform float u_texel_width;

        in vec2 v_uv;

        out vec4 outColor;
        
        void main() {
            vec4 color = vec4(0.0);
            float weights[5] = float[](0.227, 0.194, 0.121, 0.054, 0.016); // Gaussian weights
            for (int i = -4; i <= 4; ++i) {
                float w = weights[abs(i)];
                vec2 offset = float(i*2) * vec2(u_texel_width, 0.0);
                color += texture(u_texture, v_uv + offset) * w;
            }
            outColor = color;
        }
        "##,
        );

        let blur_shader_v = Shader::new(
            context,
            fullscreen_quad_vs,
            r##"#version 300 es
    
        precision highp float;

        uniform sampler2D u_texture;
        uniform float u_texel_height;

        in vec2 v_uv;

        out vec4 outColor;
        
        void main() {
            vec4 color = vec4(0.0);
            float weights[5] = float[](0.227, 0.194, 0.121, 0.054, 0.016); // Gaussian weights
            for (int i = -4; i <= 4; ++i) {
                float w = weights[abs(i)];
                vec2 offset = float(i*2) * vec2(0.0, u_texel_height);
                color += texture(u_texture, v_uv + offset) * w;
            }
            outColor = color;
        }
        "##,
        );

        let composite_shader = Shader::new(
            context,
            fullscreen_quad_vs,
            r##"#version 300 es
    
        precision highp float;

        uniform sampler2D u_texture;
        uniform sampler2D u_blur;

        in vec2 v_uv;

        out vec4 outColor;
        
        vec3 reinhard_extended(vec3 v, float max_white) {
            vec3 numerator = v * (1.0f + (v / vec3(max_white * max_white)));
            return numerator / (1.0f + v);
        }

        void main() {
            vec4 scene = texture(u_texture, v_uv);
            vec4 bloom = texture(u_blur, v_uv);
            vec4 color = max(scene, bloom);
            color.rgb = reinhard_extended(color.rgb, 1.0);
            outColor = color;
        }
        "##,
        );

        let crt_shader = Shader::new(
            context,
            fullscreen_quad_vs,
            r##"#version 300 es
    
        precision highp float;

        uniform sampler2D u_texture;

        in vec2 v_uv;

        out vec4 outColor;
        
        float curvature = 3.0;

        vec2 curveRemap(vec2 uv) {
            uv = uv * 2.0 - 1.0;
            vec2 offset = abs(uv.yx) / vec2(curvature, curvature);
            uv = uv + uv * offset * offset;
            uv = uv * 0.5 + 0.5;
            return uv;
        }

        void main() {
            vec2 uv = curveRemap(v_uv);

            vec3 color = texture(u_texture, uv).rgb;

            // Slight chromatic aberration
            float aberr = 0.001;
            float r = texture(u_texture, uv + vec2(aberr, 0.0)).r;
            float g = texture(u_texture, uv).g;
            float b = texture(u_texture, uv - vec2(aberr, 0.0)).b;
            color = vec3(r, g, b);

            // Horizontal scanlines
            float scanline = sin(uv.y * 200.0 * 2.0 * 3.14159) * 0.1;
            color -= scanline;

            // Vignette
            float dist = distance(v_uv, vec2(0.5));
            float vignette = pow(1.0 - dist, 1.5);
            color *= vignette;

            outColor = vec4(color, 1.0);
        }
        "##,
        );

        Ok(Self {
            context: context.clone(),
            scene_fbo,
            scene_texture,
            ping_fbo,
            ping_texture,
            pong_fbo,
            pong_texture,
            threshold_shader,
            blur_shader_h,
            blur_shader_v,
            composite_shader,
            crt_shader,
            w: 1,
            h: 1,
        })
    }

    pub fn on_resize(&mut self, w: i32, h: i32) {
        self.w = w;
        self.h = h;

        self.scene_texture.resize(w, h);
        self.ping_texture.resize(w, h);
        self.pong_texture.resize(w, h);
    }

    pub fn start_capture(&self) {
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.scene_fbo));
    }

    pub fn finish(&self) {
        // ping
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.ping_fbo));
        self.threshold_shader
            .bind_texture("u_texture", 0, &self.scene_texture);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        // pong
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.pong_fbo));
        self.blur_shader_v
            .bind_texture("u_texture", 0, &self.ping_texture);
        self.blur_shader_v
            .uniform1f("u_texel_height", 1.0 / self.h as f32);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        // ping
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.ping_fbo));
        self.blur_shader_h
            .bind_texture("u_texture", 0, &self.pong_texture);
        self.blur_shader_h
            .uniform1f("u_texel_width", 1.0 / self.w as f32);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        // pong
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.pong_fbo));
        self.blur_shader_v
            .bind_texture("u_texture", 0, &self.ping_texture);
        self.blur_shader_v
            .uniform1f("u_texel_height", 1.0 / self.h as f32);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        // ping
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.ping_fbo));
        self.blur_shader_h
            .bind_texture("u_texture", 0, &self.pong_texture);
        self.blur_shader_h
            .uniform1f("u_texel_width", 1.0 / self.w as f32);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        // composite
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.pong_fbo));
        self.composite_shader
            .bind_texture("u_texture", 0, &self.scene_texture);
        self.composite_shader
            .bind_texture("u_blur", 1, &self.ping_texture);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        // CRT effect
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        self.crt_shader
            .bind_texture("u_texture", 0, &self.scene_texture);
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
    }
}
