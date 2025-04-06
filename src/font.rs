use std::collections::HashMap;

use glam::{Vec2, vec2};
use ttf_parser::{Face, OutlineBuilder};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
}

// A path implemented as a list of quadratic curves
pub struct Path {
    pub segments: Vec<Segment>,
    cursor: Vec2,
    pub offset: Vec2,
    pub size: Vec2,
}

impl Path {
    pub fn new() -> Self {
        Self {
            segments: vec![],
            cursor: Vec2::ZERO,
            offset: Vec2::ZERO,
            size: Vec2::ONE,
        }
    }

    pub fn normalize(&mut self) {
        let mut min = Vec2::MAX;
        let mut max = Vec2::MIN;

        for s in &self.segments {
            min = min.min(s.a).min(s.b).min(s.c);
            max = max.max(s.a).max(s.b).max(s.c);
        }

        let offset = min;
        let size = max - min;

        for s in &mut self.segments {
            s.a = (s.a - offset) / size;
            s.b = (s.b - offset) / size;
            s.c = (s.c - offset) / size;
        }

        self.offset = offset;
        self.size = size;
    }
}

impl OutlineBuilder for Path {
    fn move_to(&mut self, x: f32, y: f32) {
        self.cursor = vec2(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let c = vec2(x, y);
        self.segments.push(Segment {
            a: self.cursor,
            b: self.cursor.lerp(c, 0.5),
            c,
        });
        self.cursor = c;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let c = vec2(x, y);
        self.segments.push(Segment {
            a: self.cursor,
            b: vec2(x1, y1),
            c,
        });
        self.cursor = c;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let a = self.cursor;
        let b = vec2(x1, y1);
        let c = vec2(x2, y2);
        let d = vec2(x, y);

        let pa = a.lerp(b, 0.75);
        let pb = d.lerp(c, 0.75);

        let diff = (d - a) / 16.0;

        let pc1 = a.lerp(b, 3.0 / 8.0);
        let pc2 = pa.lerp(pb, 3.0 / 8.0) - diff;
        let pc3 = pb.lerp(pa, 3.0 / 8.0) + diff;
        let pc4 = d.lerp(c, 3.0 / 8.0);

        let pa1 = 0.5 * (pc1 + pc2);
        let pa2 = 0.5 * (pa + pb);
        let pa3 = 0.5 * (pc3 + pc4);

        self.segments.push(Segment { a, b: pc1, c: pa1 });
        self.segments.push(Segment {
            a: pa1,
            b: pc2,
            c: pa2,
        });
        self.segments.push(Segment {
            a: pa2,
            b: pc3,
            c: pa3,
        });
        self.segments.push(Segment {
            a: pa3,
            b: pc4,
            c: d,
        });

        self.cursor = d;
    }

    fn close(&mut self) {}
}

pub struct Character {
    pub path: Path,
    pub advance: f32,
}

pub struct Font {
    pub chars: HashMap<char, Character>,
    pub height: f32,
    pub descender: f32,
    pub ascender: f32,
    pub units_per_em: f32,
}

impl Font {
    pub fn from_slice(data: &[u8], index: u32) -> Self {
        let face = Face::parse(data, index).expect("parse font file");

        // a useful subset of printable ASCII
        const ALPHABET: &str =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,!?:()-+*/\\ ";

        let mut chars = HashMap::new();

        for c in ALPHABET.chars() {
            let mut path = face
                .glyph_index(c)
                .map(|glyph_id| {
                    let mut path = Path::new();
                    let _ = face.outline_glyph(glyph_id, &mut path);
                    path
                })
                .unwrap_or_else(|| Path::new());
            path.normalize();
            let advance = face
                .glyph_index(c)
                .map(|glyph_id| face.glyph_hor_advance(glyph_id))
                .flatten()
                .map(|advance| advance as f32)
                .unwrap_or(0.0);

            chars.insert(c, Character { path, advance });
        }

        Self {
            chars,
            height: face.height() as f32,
            descender: face.descender() as f32,
            ascender: face.ascender() as f32,
            units_per_em: face.units_per_em() as f32,
        }
    }
}
