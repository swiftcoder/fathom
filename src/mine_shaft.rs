use glam::{Vec2, vec2};
use noise::{NoiseFn, Perlin};

pub struct MineShaft {
    pub width: f32,
    pub height: f32,
    pub shaft_radius: f32,
    pub noise_scale: f32,
    pub noise_amplitude: f32,
    pub noise: Perlin,
}

impl MineShaft {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            shaft_radius: 60.0,
            noise_scale: 1.0 / 80.0,
            noise_amplitude: 60.0,
            noise: Perlin::new(0),
        }
    }

    /// Signed distance from vertical shaft wall
    fn shaft_distance(&self, p: Vec2, radius: f32) -> f32 {
        (p.x - (p.y * 0.01).sin() * 35.0).abs() - radius
    }

    fn secondary_shaft_distance(&self, p: Vec2) -> f32 {
        self.shaft_distance(p, 14.0)
    }

    /// Signed distance from a circular starting zone
    fn starting_zone_distance(&self, p: Vec2) -> f32 {
        p.length() - 100.0
    }

    fn noise(&self, mut p: Vec2) -> f32 {
        p *= self.noise_scale;
        self.noise.get([p.x as f64, p.y as f64]) as f32 * self.noise_amplitude
    }

    /// Final combined distance field at a point
    pub fn distance(&self, p: Vec2) -> f32 {
        let shaft = self.shaft_distance(p, self.shaft_radius);
        let noise = self.noise(p);
        let starting_zone = self.starting_zone_distance(p);
        let shaft_clear_zone = self.secondary_shaft_distance(p);

        f32::min(shaft_clear_zone, f32::min(starting_zone, shaft + noise))
    }

    pub fn marching_squares(&self, resolution: f32, center: Vec2) -> Vec<Vec2> {
        let mut segments = Vec::new();

        let offset = center - vec2(self.width, self.height) * 0.5;

        let cols = (self.width / resolution) as i32;
        let rows = (self.height / resolution) as i32;

        for y in 0..rows {
            for x in 0..cols {
                let p0 = Vec2::new(x as f32 * resolution, y as f32 * resolution) + offset;
                let p1 = p0 + Vec2::new(resolution, 0.0);
                let p2 = p0 + Vec2::new(resolution, resolution);
                let p3 = p0 + Vec2::new(0.0, resolution);

                let d0 = self.distance(p0);
                let d1 = self.distance(p1);
                let d2 = self.distance(p2);
                let d3 = self.distance(p3);

                let cell = [d0 < 0.0, d1 < 0.0, d2 < 0.0, d3 < 0.0];
                let index = (cell[0] as u8) << 0
                    | (cell[1] as u8) << 1
                    | (cell[2] as u8) << 2
                    | (cell[3] as u8) << 3;

                // Interpolate edge intersections
                let interp = |a: Vec2, da: f32, b: Vec2, db: f32| {
                    if (da - db).abs() < 0.0001 {
                        (a + b) * 0.5
                    } else {
                        let t = (0.0 - da) / (db - da);
                        a + (b - a) * t
                    }
                };

                let e0 = interp(p0, d0, p1, d1);
                let e1 = interp(p1, d1, p2, d2);
                let e2 = interp(p2, d2, p3, d3);
                let e3 = interp(p3, d3, p0, d0);

                let center = (p0 + p2) * 0.5;
                let dc = self.distance(center);
                let center_sign = dc < 0.0;

                let edges = resolve_case(index, center_sign);
                for (a, b) in edges {
                    let pa = match a {
                        0 => e0,
                        1 => e1,
                        2 => e2,
                        3 => e3,
                        _ => continue,
                    };
                    let pb = match b {
                        0 => e0,
                        1 => e1,
                        2 => e2,
                        3 => e3,
                        _ => continue,
                    };
                    segments.push(pa);
                    segments.push(pb);
                }
            }
        }

        segments
    }
}

pub fn resolve_case(index: u8, center_sign: bool) -> &'static [(u8, u8)] {
    match index {
        0 | 15 => &[],
        1 => &[(3, 0)],
        2 => &[(0, 1)],
        3 => &[(3, 1)],
        4 => &[(1, 2)],
        5 => {
            if center_sign {
                &[(0, 1), (2, 3)]
            } else {
                &[(3, 0), (1, 2)]
            }
        }
        6 => &[(0, 2)],
        7 => &[(3, 2)],
        8 => &[(2, 3)],
        9 => &[(0, 2)],
        10 => {
            if center_sign {
                &[(1, 2), (3, 0)]
            } else {
                &[(0, 1), (2, 3)]
            }
        }
        11 => &[(1, 2)],
        12 => &[(1, 3)],
        13 => &[(0, 1)],
        14 => &[(3, 0)],
        _ => &[],
    }
}
