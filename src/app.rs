use glam::{Mat3, Mat4, Vec2, vec2, vec3};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlCanvasElement, KeyboardEvent, WebGl2RenderingContext, window};

use crate::{
    document,
    scribe::{Color, Scribe},
};

pub struct Entity {
    pub transform: Mat3,
    pub vel: Vec2,
}

impl Entity {
    pub fn pos(&self) -> Vec2 {
        self.transform.transform_point2(Vec2::ZERO)
    }

    pub fn forward(&self) -> Vec2 {
        self.transform.transform_vector2(vec2(0.0, 1.0))
    }
}

pub struct AppState {
    scribe: Scribe,
    thrust: bool,
    turn_left: bool,
    turn_right: bool,
    player_ship: Entity,
}

impl AppState {
    pub fn new(context: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        Ok(Self {
            scribe: Scribe::new(context),
            thrust: false,
            turn_left: false,
            turn_right: false,
            player_ship: Entity {
                transform: Mat3::IDENTITY,
                vel: Vec2::ZERO,
            },
        })
    }

    pub fn on_resize(&self, canvas: &HtmlCanvasElement, context: &WebGl2RenderingContext) {
        let device_pixel_ratio = window().unwrap().device_pixel_ratio();
        let document = document();
        let w = document.body().unwrap().client_width() as f64 * device_pixel_ratio;
        let h = document.body().unwrap().client_height() as f64 * device_pixel_ratio;

        canvas.set_width(w as u32);
        canvas.set_height(h as u32);
        context.viewport(0, 0, w as i32, h as i32);
    }

    pub fn on_keydown(&mut self, key: KeyboardEvent) {
        match key.code().as_str() {
            "KeyW" | "ArrowUp" => self.thrust = true,
            "KeyA" | "ArrowLeft" => self.turn_left = true,
            "KeyD" | "ArrowRight" => self.turn_right = true,
            _ => log::info!("key down {:?}", key.code()),
        }
    }

    pub fn on_keyup(&mut self, key: KeyboardEvent) {
        match key.code().as_str() {
            "KeyW" | "ArrowUp" => self.thrust = false,
            "KeyA" | "ArrowLeft" => self.turn_left = false,
            "KeyD" | "ArrowRight" => self.turn_right = false,
            _ => {}
        }
    }

    pub fn fixed_update(&mut self, dt: f32) {
        self.player_ship.transform *= Mat3::from_translation(self.player_ship.vel * dt);

        if self.thrust {
            self.player_ship.vel += self.player_ship.forward() * 15.0 * dt;
        }
        if self.turn_left {
            self.player_ship.transform *= Mat3::from_angle(dt);
        }
        if self.turn_right {
            self.player_ship.transform *= Mat3::from_angle(-dt);
        }

        // gravity
        self.player_ship.vel += vec2(0.0, -10.0) * dt;

        // clamp speed
        self.player_ship.vel = self.player_ship.vel.clamp_length_max(20.0);
    }

    pub fn draw(&mut self, context: &WebGl2RenderingContext) {
        let document = document();
        let aspect = document.body().unwrap().client_width() as f32
            / document.body().unwrap().client_height() as f32;

        let transform =
            Mat4::orthographic_rh_gl(-100.0 * aspect, 100.0 * aspect, -100.0, 100.0, -10.0, 10.0)
                * Mat4::from_translation(-self.player_ship.pos().extend(0.0));

        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        // draw background crosses
        {
            let c = (self.player_ship.pos() / 40.0).floor() * 40.0;
            for i in 0..20 {
                for j in 0..20 {
                    let p = c + vec2(i as f32 - 9.0, j as f32 - 9.0) * 40.0;
                    self.scribe.draw_poly_line(
                        &[p + vec2(-1.0, 0.0), p + vec2(1.0, 0.0)],
                        1.0,
                        false,
                        Color::PALE_BLUE,
                    );
                    self.scribe.draw_poly_line(
                        &[p + vec2(0.0, -1.0), p + vec2(0.0, 1.0)],
                        1.0,
                        false,
                        Color::PALE_BLUE,
                    );
                }
            }
            self.scribe.render(transform);
        }

        // draw player ship
        {
            let p = |v| self.player_ship.transform.transform_point2(v);

            let ship = [p(vec2(-7.0, -7.0)), p(vec2(7.0, -7.0)), p(vec2(0.0, 7.0))];
            self.scribe.draw_poly_line(&ship, 1.0, true, Color::WHITE);

            // draw engine exhaust
            if self.thrust {
                let exhaust = [p(vec2(-3.0, -8.0)), p(vec2(3.0, -8.0)), p(vec2(0.0, -12.0))];
                self.scribe
                    .draw_poly_line(&exhaust, 1.0, true, Color::YELLOW);
            }
        }
        self.scribe.render(transform);
    }
}
