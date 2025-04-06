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

enum GameState {
    SplashScreen,
    InGame,
    GameOver,
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
    health: usize,
    invulnerability_ticks: usize,
    game_state: GameState,
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
            mine_shaft: MineShaft::new(760.0, 340.0),
            max_depth: 0,
            health: 5,
            invulnerability_ticks: 0,
            game_state: GameState::SplashScreen,
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

        match self.game_state {
            GameState::GameOver | GameState::SplashScreen => {
                self.game_state = GameState::InGame;
                self.player_ship.transform = Mat3::IDENTITY;
                self.player_ship.vel = Vec2::ZERO;
                self.max_depth = 0;
                self.health = 5;
            }
            _ => {}
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

        // handle collision
        let distance = self.mine_shaft.distance(self.player_ship.pos());
        if distance < 7.0 {
            if let Some(n) = self.mine_shaft.normal(self.player_ship.pos()) {
                self.player_ship.transform =
                    Mat3::from_translation(n * (7.0 - distance)) * self.player_ship.transform;

                // log::info!("penetration {}", distance + 7.0);

                let vn = self.player_ship.vel.dot(n) * n;
                let vt = self.player_ship.vel - vn;

                const RESTITUTION: f32 = 0.5;
                const FRICTION: f32 = 0.125;

                // Reflect the normal part with restitution (bounce factor)
                let reflected_vn = -vn * RESTITUTION;

                // Apply friction to the tangential (sliding) part
                let friction_vt = vt * (1.0 - FRICTION);

                self.player_ship.vel = reflected_vn + friction_vt;

                // if we aren't invulnerable, apply damage
                if self.invulnerability_ticks == 0 && self.health > 0 {
                    self.health -= 1;

                    // if we run out of health, game over. Otherwise give us 2 seconds of invulnerability
                    if self.health < 1 {
                        self.game_state = GameState::GameOver;
                        self.player_ship.transform = Mat3::from_translation(Vec2::ZERO);
                        self.health = 5;
                    } else {
                        self.invulnerability_ticks = 2 * 120;
                    }
                }
            }
        }

        // if we are invulnerable, count it down
        if self.invulnerability_ticks > 0 {
            self.invulnerability_ticks -= 1;
        }

        // handle player input
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

        // calculate score
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
                    if self.mine_shaft.distance(p) > 0.0 {
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

            if self.invulnerability_ticks % 30 < 15 {
                let ship = [p(vec2(-7.0, -7.0)), p(vec2(7.0, -7.0)), p(vec2(0.0, 7.0))];
                self.scribe.draw_poly_line(&ship, 1.0, true, Color::White);
            }

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

        self.text.draw(
            pos.x - 120.0,
            pos.y - 80.0,
            6.0,
            Align::Left,
            &format!("Health {}", "I".repeat(self.health)),
        );

        match self.game_state {
            GameState::SplashScreen => {
                self.text
                    .draw(pos.x, pos.y + 20.0, 18.0, Align::Center, "FATHOM");

                self.text.draw(
                    pos.x,
                    pos.y - 30.0,
                    4.0,
                    Align::Center,
                    "Press any key to start",
                );
            }
            GameState::GameOver => {
                self.text
                    .draw(pos.x, pos.y + 20.0, 18.0, Align::Center, "Game Over :(");

                self.text.draw(
                    pos.x,
                    pos.y - 30.0,
                    4.0,
                    Align::Center,
                    "Press any key to restart",
                );
            }
            _ => {}
        }

        self.text.render(transform);

        self.post_process.finish();
    }
}
