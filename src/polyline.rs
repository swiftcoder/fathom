use glam::{Mat2, Vec2, vec2};
use itertools::Itertools;
use std::f32::consts::PI;

pub fn lines_to_triangles(points: &[Vec2], width: f32) -> Vec<Vec2> {
    let mut verts = Vec::new();
    let half_width = width / 2.0;

    if points.len() < 2 {
        return verts;
    }

    for (p0, p1) in points.iter().tuples() {
        let dir = (p1 - p0).normalize_or_zero();
        let normal = perpendicular(dir);

        // Create two triangles forming a quad
        let a = p0 + normal * half_width;
        let b = p0 - normal * half_width;
        let c = p1 + normal * half_width;
        let d = p1 - normal * half_width;

        // Triangle 1: a, b, c
        push_triangle(&mut verts, a, b, c);
        // Triangle 2: c, b, d
        push_triangle(&mut verts, c, b, d);
    }

    verts
}

/// Converts a polyline into triangles with round joins and round end caps.
/// Triangles are returned as a flat Vec<Vec2> in CW winding order.
pub fn polyline_to_triangles(
    points: &[Vec2],
    width: f32,
    segments_per_arc: usize,
    closed: bool,
) -> Vec<Vec2> {
    let mut verts = Vec::new();
    let half_width = width / 2.0;

    if points.len() < 2 {
        return verts;
    }

    let line_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };

    for i in 0..line_count {
        let p0 = points[i];
        let p1 = if i < points.len() - 1 {
            points[i + 1]
        } else {
            points[0]
        };
        let dir = (p1 - p0).normalize_or_zero();
        let normal = perpendicular(dir);

        // Create two triangles forming a quad
        let a = p0 + normal * half_width;
        let b = p0 - normal * half_width;
        let c = p1 + normal * half_width;
        let d = p1 - normal * half_width;

        // Triangle 1: a, b, c
        push_triangle(&mut verts, a, b, c);
        // Triangle 2: c, b, d
        push_triangle(&mut verts, c, b, d);

        // Handle join (skip if it's the last segment)
        if i < points.len() - 2 || (closed && i < points.len() - 1) {
            let p2 = if i < points.len() - 2 {
                points[i + 2]
            } else {
                points[0]
            };
            let dir_next = (p2 - p1).normalize_or_zero();
            generate_round_join(&mut verts, p1, dir_next, dir, half_width, segments_per_arc);
        }
    }

    let last = points.len() - 1;
    let dir_start = (points[1] - points[0]).normalize_or_zero();
    let dir_end = (points[last] - points[last - 1]).normalize_or_zero();
    if closed {
        let dir = (points[0] - points[last]).normalize_or_zero();
        generate_round_join(
            &mut verts,
            points[0],
            dir_start,
            dir,
            half_width,
            segments_per_arc,
        );
    } else {
        generate_round_cap(
            &mut verts,
            points[0],
            dir_start,
            half_width,
            segments_per_arc,
            true,
        );
        generate_round_cap(
            &mut verts,
            points[last],
            dir_end,
            half_width,
            segments_per_arc,
            false,
        );
    }

    verts
}

fn perpendicular(v: Vec2) -> Vec2 {
    vec2(-v.y, v.x)
}

fn signed_angle(a: Vec2, b: Vec2) -> f32 {
    a.perp_dot(b).atan2(a.dot(b))
}

fn push_triangle(verts: &mut Vec<Vec2>, a: Vec2, b: Vec2, c: Vec2) {
    verts.push(a);
    verts.push(b);
    verts.push(c);
}

fn generate_round_join(
    verts: &mut Vec<Vec2>,
    center: Vec2,
    dir_in: Vec2,
    dir_out: Vec2,
    half_width: f32,
    segments: usize,
) {
    // Outward normals
    let n1 = perpendicular(dir_in);
    let n2 = perpendicular(dir_out);

    // Check turn direction
    let cross = dir_in.perp_dot(dir_out);
    let is_left_turn = cross > 0.0;

    // Start and end arc vectors — these point outward from the join
    let start = if is_left_turn { n1 } else { -n1 };
    let end = if is_left_turn { n2 } else { -n2 };

    // Compute angle to sweep
    let mut angle = signed_angle(start, end);
    if is_left_turn && angle < 0.0 {
        angle += 2.0 * PI;
    } else if !is_left_turn && angle > 0.0 {
        angle -= 2.0 * PI;
    }

    let step = angle / segments as f32;

    for i in 0..segments {
        let a0 = i as f32 * step;
        let a1 = (i + 1) as f32 * step;
        let p0 = center + Mat2::from_angle(a0) * start * half_width;
        let p1 = center + Mat2::from_angle(a1) * start * half_width;
        push_triangle(verts, center, p0, p1);
    }
}

fn generate_round_cap(
    verts: &mut Vec<Vec2>,
    center: Vec2,
    dir: Vec2,
    half_width: f32,
    segments: usize,
    at_start: bool,
) {
    // Flip for start cap
    let dir = if at_start { -dir } else { dir };
    let normal = perpendicular(dir);

    // Arc goes from left side to right side around the front
    let start = -normal;
    let end = normal;
    let angle = signed_angle(start, end); // should be +PI

    let step = angle / segments as f32;

    for i in 0..segments {
        let a0 = i as f32 * step;
        let a1 = (i + 1) as f32 * step;
        let p0 = center + Mat2::from_angle(a0) * start * half_width;
        let p1 = center + Mat2::from_angle(a1) * start * half_width;
        push_triangle(verts, center, p0, p1);
    }
}
