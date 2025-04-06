use glam::{Mat3, Mat4, Vec2, vec2};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, KeyboardEvent, WebGl2RenderingContext, window};

use crate::{
    document,
    font::Font,
    mine_shaft::MineShaft,
    post_processor::PostProcessor,
    scribe::{Color, Scribe},
    text::{Align, Text},
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
    post_process: PostProcessor,
    text: Text,
    thrust: bool,
    turn_left: bool,
    turn_right: bool,
    player_ship: Entity,
    mine_shaft: MineShaft,
    max_depth: usize,
}

const FONT: &[u8] = include_bytes!("../assets/KarmaticArcade-6Yrp1.ttf");

impl AppState {
    pub fn new(context: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        Ok(Self {
            scribe: Scribe::new(context),
            post_process: PostProcessor::new(context)?,
            text: Text::new(context, Font::from_slice(&FONT, 0)),
            thrust: false,
            turn_left: false,
            turn_right: false,
            player_ship: Entity {
                transform: Mat3::IDENTITY,
                vel: Vec2::ZERO,
            },
            mine_shaft: MineShaft::new(340.0, 340.0),
            max_depth: 0,
        })
    }

    pub fn on_resize(&mut self, canvas: &HtmlCanvasElement, context: &WebGl2RenderingContext) {
        let device_pixel_ratio = window().unwrap().device_pixel_ratio();
        let document = document();
        let w = document.body().unwrap().client_width() as f64 * device_pixel_ratio;
        let h = document.body().unwrap().client_height() as f64 * device_pixel_ratio;

        canvas.set_width(w as u32);
        canvas.set_height(h as u32);
        context.viewport(0, 0, w as i32, h as i32);

        self.post_process.on_resize(w as i32, h as i32);
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
        self.player_ship.transform =
            Mat3::from_translation(self.player_ship.vel * dt) * self.player_ship.transform;

        if self.thrust {
            self.player_ship.vel += self.player_ship.forward() * 30.0 * dt;
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
        self.player_ship.vel = self.player_ship.vel.clamp_length_max(40.0);

        self.max_depth = self.max_depth.max(-self.player_ship.pos().y as usize);
    }

    pub fn draw(&mut self, context: &WebGl2RenderingContext) {
        let document = document();
        let aspect = document.body().unwrap().client_width() as f32
            / document.body().unwrap().client_height() as f32;

        let transform =
            Mat4::orthographic_rh_gl(-100.0 * aspect, 100.0 * aspect, -100.0, 100.0, -10.0, 10.0)
                * Mat4::from_translation(-self.player_ship.pos().extend(0.0));

        self.post_process.start_capture();

        context.clear_color(0.0, 0.0, 0.5, 1.0);
        context.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        let pos = self.player_ship.pos();
        let grid_locked_pos = (pos / 40.0).floor() * 40.0;

        // draw background crosses
        {
            for i in 0..20 {
                for j in 0..20 {
                    let p = grid_locked_pos + vec2(i as f32 - 9.0, j as f32 - 9.0) * 40.0;
                    if self.mine_shaft.distance(p) < 0.0 {
                        self.scribe.draw_poly_line(
                            &[p + vec2(-1.0, 0.0), p + vec2(1.0, 0.0)],
                            1.0,
                            false,
                            Color::PaleBlue,
                        );
                        self.scribe.draw_poly_line(
                            &[p + vec2(0.0, -1.0), p + vec2(0.0, 1.0)],
                            1.0,
                            false,
                            Color::PaleBlue,
                        );
                    }
                }
            }
            self.scribe.render(transform);
        }

        // draw mine shaft
        {
            let vertices = self.mine_shaft.marching_squares(5.0, grid_locked_pos);
            self.scribe.draw_lines(&vertices, 1.0, Color::White);
        }

        // draw player ship
        {
            let p = |v| self.player_ship.transform.transform_point2(v);

            let ship = [p(vec2(-7.0, -7.0)), p(vec2(7.0, -7.0)), p(vec2(0.0, 7.0))];
            self.scribe.draw_poly_line(&ship, 1.0, true, Color::White);

            // draw engine exhaust
            if self.thrust {
                let exhaust = [p(vec2(-3.0, -8.0)), p(vec2(3.0, -8.0)), p(vec2(0.0, -12.0))];
                self.scribe
                    .draw_poly_line(&exhaust, 1.0, true, Color::Yellow);
            }
        }
        self.scribe.render(transform);

        self.text.draw(
            pos.x - 120.0,
            pos.y + 80.0,
            6.0,
            Align::Left,
            &format!("{} meters", self.max_depth),
        );

        self.text.render(transform);

        self.post_process.finish();
    }
}
