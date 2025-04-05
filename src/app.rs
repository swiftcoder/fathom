use glam::vec2;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{document, scribe::{Color, Scribe}};

pub struct AppState {
    scribe: Scribe,
}

impl AppState {
    pub fn new(context: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        Ok(Self {
            scribe: Scribe::new(context),
        })
    }

    pub fn on_resize(&self, canvas: &HtmlCanvasElement, context: &WebGl2RenderingContext) {
        let document = document();
        let w = document.body().unwrap().client_width();
        let h = document.body().unwrap().client_height();

        canvas.set_width(w as u32);
        canvas.set_height(h as u32);
        context.viewport(0, 0, w, h);
    }

    pub fn draw(&mut self, context: &WebGl2RenderingContext) {
        let document = document();
        let aspect = document.body().unwrap().client_width() as f32
            / document.body().unwrap().client_height() as f32;

        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        let ship = [vec2(-7.0, -7.0), vec2(7.0, -7.0), vec2(0.0, 7.0)];
        self.scribe.draw_poly_line(&ship, 1.0, true, Color::WHITE);

        let exhaust = [vec2(-3.0, -8.0), vec2(3.0, -8.0), vec2(0.0, -12.0)];
        self.scribe.draw_poly_line(&exhaust, 1.0, true, Color::YELLOW);

        self.scribe.render(aspect);
    }
}
