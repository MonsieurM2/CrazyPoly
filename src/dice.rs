//! Painting the dice: a recorded rigid-body throw, seen from straight above.
//!
//! The throw itself is simulated by the generic physics engine in
//! `dice_physics`: that module steps a real world (a tray, two cubes, gravity,
//! bouncing, friction) and records the pose of both dice. This module only
//! replays those poses.
//!
//! The camera is an orthographic look straight down the world's Z axis, which
//! matches the board: it is a flat table seen from the vertical. Projecting
//! that way, a cube at rest shows exactly one face - the upper one - and the
//! value of that face is the result the simulation produced. Side faces are
//! only visible while a die is tilted in mid-air, and they vanish as it lands.

use macroquad::prelude::*;

use crate::dice_physics::{Frame, FRAME_DT};
use crate::game::Game;

pub type M3 = [[f32; 3]; 3];
pub type V3 = [f32; 3];

pub const UP: V3 = [0.0, 0.0, 1.0];

/// Half edge of a die, in die-edge units.
const HALF: f32 = 0.5;
/// Where a die waits between throws, left and right of the panel centre.
pub const HOME_X: f32 = 0.82;

const INK: Color = Color { r: 0.10, g: 0.10, b: 0.12, a: 1.0 };
const EDGE: Color = Color { r: 0.36, g: 0.34, b: 0.31, a: 0.75 };
const CREAM: Color = Color { r: 0.985, g: 0.975, b: 0.945, a: 1.0 };

// ------------------------------------------------------------------ vectors

pub fn dot3(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross3(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn norm3(v: V3) -> V3 {
    let len = dot3(v, v).sqrt().max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

// ---------------------------------------------------------------- rotations

pub fn m3_apply(m: &M3, v: V3) -> V3 {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

pub fn m3_mul(a: &M3, b: &M3) -> M3 {
    let mut out = [[0.0_f32; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            for k in 0..3 {
                *cell += a[i][k] * b[k][j];
            }
        }
    }
    out
}

pub fn m3_axis_rot(axis: V3, angle: f32) -> M3 {
    let a = norm3(axis);
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;
    [
        [t * a[0] * a[0] + c, t * a[0] * a[1] - s * a[2], t * a[0] * a[2] + s * a[1]],
        [t * a[0] * a[1] + s * a[2], t * a[1] * a[1] + c, t * a[1] * a[2] - s * a[0]],
        [t * a[0] * a[2] - s * a[1], t * a[1] * a[2] + s * a[0], t * a[2] * a[2] + c],
    ]
}

/// Smallest rotation taking `from` onto `to`.
pub fn m3_between(from: V3, to: V3) -> M3 {
    let a = norm3(from);
    let b = norm3(to);
    let axis = cross3(a, b);
    let sin = dot3(axis, axis).sqrt();
    let cos = dot3(a, b);
    if sin < 1e-5 {
        if cos > 0.0 {
            return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        }
        return m3_axis_rot([1.0, 0.0, 0.0], std::f32::consts::PI);
    }
    m3_axis_rot(axis, sin.atan2(cos))
}

/// Interpolate two rotations and re-orthonormalise the result.
pub fn m3_blend(a: &M3, b: &M3, w: f32) -> M3 {
    let mut rows: [V3; 3] = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            rows[i][j] = a[i][j] * (1.0 - w) + b[i][j] * w;
        }
    }
    let r0 = norm3(rows[0]);
    let p = dot3(rows[1], r0);
    let r1 = norm3([rows[1][0] - r0[0] * p, rows[1][1] - r0[1] * p, rows[1][2] - r0[2] * p]);
    let r2 = cross3(r0, r1);
    [r0, r1, r2]
}

// -------------------------------------------------------------------- solid

struct Face {
    n: V3,
    u: V3,
    v: V3,
    value: u8,
}

/// Standard die: opposite faces sum to seven. `n` is the outward normal in the
/// die's own frame, `u`/`v` the in-plane axes the pips are laid out on.
const FACES: [Face; 6] = [
    Face { n: [0.0, 0.0, 1.0], u: [1.0, 0.0, 0.0], v: [0.0, 1.0, 0.0], value: 1 },
    Face { n: [0.0, 0.0, -1.0], u: [1.0, 0.0, 0.0], v: [0.0, -1.0, 0.0], value: 6 },
    Face { n: [0.0, 1.0, 0.0], u: [1.0, 0.0, 0.0], v: [0.0, 0.0, -1.0], value: 2 },
    Face { n: [0.0, -1.0, 0.0], u: [1.0, 0.0, 0.0], v: [0.0, 0.0, 1.0], value: 5 },
    Face { n: [1.0, 0.0, 0.0], u: [0.0, 0.0, -1.0], v: [0.0, 1.0, 0.0], value: 3 },
    Face { n: [-1.0, 0.0, 0.0], u: [0.0, 0.0, 1.0], v: [0.0, 1.0, 0.0], value: 4 },
];

fn pip_grid(value: u8) -> &'static [(f32, f32)] {
    match value {
        1 => &[(0.0, 0.0)],
        2 => &[(-1.0, -1.0), (1.0, 1.0)],
        3 => &[(-1.0, -1.0), (0.0, 0.0), (1.0, 1.0)],
        4 => &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)],
        5 => &[(-1.0, -1.0), (1.0, -1.0), (0.0, 0.0), (-1.0, 1.0), (1.0, 1.0)],
        _ => &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 0.0), (1.0, 0.0), (-1.0, 1.0), (1.0, 1.0)],
    }
}

/// Index of the face pointing at the sky in orientation `rot`.
fn top_face(rot: &M3) -> usize {
    let mut best = (f32::NEG_INFINITY, 0usize);
    for (i, face) in FACES.iter().enumerate() {
        let up = m3_apply(rot, face.n)[2];
        if up > best.0 {
            best = (up, i);
        }
    }
    best.1
}

/// Value of the face pointing at the sky in orientation `rot`.
pub fn top_value(rot: &M3) -> u8 {
    FACES[top_face(rot)].value
}

/// Same upper face, same way round, but lying perfectly flat: this is the pose
/// a thrown die is eased back into before it returns to its home spot.
pub fn flatten(rot: &M3) -> M3 {
    let face = &FACES[top_face(rot)];
    // Local -> flat: this face ends up pointing at the sky, all the others away.
    let base = m3_between(face.n, UP);
    let u = m3_apply(rot, face.u);
    let flat_u = m3_apply(&base, face.u);
    // Turn the flattened face so its pip axis points the same way as on screen.
    let alpha = u[1].atan2(u[0]) - flat_u[1].atan2(flat_u[0]);
    m3_mul(&m3_axis_rot(UP, alpha), &base)
}

/// Resting orientation: `value` facing up, turned by `yaw` around the vertical.
pub fn rest_pose(value: u8, yaw: f32) -> M3 {
    let face = FACES.iter().find(|f| f.value == value).unwrap_or(&FACES[0]);
    m3_mul(&m3_axis_rot(UP, yaw), &m3_between(face.n, UP))
}

// ---------------------------------------------------------------- projection

/// Orthographic, straight-down projection of a board point. `size` is the
/// on-screen length of one die edge, so world units are die edges.
fn project(p: V3, cx: f32, cy: f32, size: f32) -> (f32, f32) {
    (cx + p[0] * size, cy - p[1] * size)
}

fn poly(pts: &[(f32, f32)], color: Color) {
    for k in 1..pts.len() - 1 {
        draw_triangle(
            vec2(pts[0].0, pts[0].1),
            vec2(pts[k].0, pts[k].1),
            vec2(pts[k + 1].0, pts[k + 1].1),
            color,
        );
    }
}

fn quad_color(shade: f32) -> Color {
    Color {
        r: (CREAM.r * shade).min(1.0),
        g: (CREAM.g * shade).min(1.0),
        b: (CREAM.b * shade).min(1.0),
        a: 1.0,
    }
}

/// Pips are real discs lying in the face plane, so the projection squashes
/// them into ellipses as soon as the face tilts.
fn draw_pips(frame: &Frame, face: &Face, n: V3, u: V3, v: V3, cx: f32, cy: f32, size: f32) {
    let center = add(frame.pos, scale(n, HALF));
    let radius = HALF * 0.155;
    for (gx, gy) in pip_grid(face.value) {
        let pip = add(center, scale(add(scale(u, gx * 0.56), scale(v, gy * 0.56)), HALF));
        let (px, py) = project(pip, cx, cy, size);
        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(12);
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU;
            let p = add(pip, add(scale(u, a.cos() * radius), scale(v, a.sin() * radius)));
            pts.push(project(p, cx, cy, size));
        }
        for k in 0..pts.len() {
            let a = pts[k];
            let b = pts[(k + 1) % pts.len()];
            draw_triangle(vec2(px, py), vec2(a.0, a.1), vec2(b.0, b.1), INK);
        }
    }
}

fn draw_face(frame: &Frame, index: usize, cx: f32, cy: f32, size: f32) {
    let face = &FACES[index];
    let n = m3_apply(&frame.rot, face.n);
    let u = m3_apply(&frame.rot, face.u);
    let v = m3_apply(&frame.rot, face.v);

    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let mut pts = [(0.0_f32, 0.0_f32); 4];
    for (k, (a, b)) in corners.iter().enumerate() {
        let p = add(
            frame.pos,
            scale(add(n, add(scale(u, *a), scale(v, *b))), HALF),
        );
        pts[k] = project(p, cx, cy, size);
    }

    // A face seen from straight above is lit in full; the more it tilts away
    // from the camera, the darker it gets.
    let shade = 0.62 + 0.38 * n[2].clamp(0.0, 1.0);
    poly(&pts, quad_color(shade * 0.90));

    // Slightly smaller inner quad: the bevelled top of a real die.
    let mid = (
        (pts[0].0 + pts[1].0 + pts[2].0 + pts[3].0) * 0.25,
        (pts[0].1 + pts[1].1 + pts[2].1 + pts[3].1) * 0.25,
    );
    let mut inner = [(0.0_f32, 0.0_f32); 4];
    for k in 0..4 {
        inner[k] = (
            mid.0 + (pts[k].0 - mid.0) * 0.90,
            mid.1 + (pts[k].1 - mid.1) * 0.90,
        );
    }
    poly(&inner, quad_color(shade));

    for k in 0..4 {
        let a = pts[k];
        let b = pts[(k + 1) % 4];
        draw_line(a.0, a.1, b.0, b.1, 1.0, EDGE);
    }

    draw_pips(frame, face, n, u, v, cx, cy, size);
}

fn draw_die(frame: &Frame, cx: f32, cy: f32, size: f32) {
    let lift = (frame.pos[2] - HALF).max(0.0);

    // Contact shadow on the table: the only hint of height in a flat view.
    let (sx, sy) = project([frame.pos[0], frame.pos[1], 0.0], cx, cy, size);
    let spread = size * (0.56 + lift * 0.08);
    let alpha = 0.22 * (1.0 - (lift * 0.40).min(0.75));
    let ink = Color { r: 0.10, g: 0.08, b: 0.05, a: alpha };
    draw_ellipse(sx + size * 0.04, sy + size * 0.05, spread, spread * 0.95, 0.0, ink);
    let core = Color { r: 0.10, g: 0.08, b: 0.05, a: alpha * 0.8 };
    draw_ellipse(
        sx + size * 0.03,
        sy + size * 0.04,
        spread * 0.64,
        spread * 0.60,
        0.0,
        core,
    );

    // Only faces turned towards the camera exist from straight above; drawing
    // them in increasing height order puts the upper face last.
    let mut visible: Vec<(f32, usize)> = Vec::with_capacity(3);
    for (i, face) in FACES.iter().enumerate() {
        let n = m3_apply(&frame.rot, face.n);
        if n[2] > 0.02 {
            visible.push((frame.pos[2] + n[2] * HALF, i));
        }
    }
    visible.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, i) in visible {
        draw_face(frame, i, cx, cy, size);
    }
}

// -------------------------------------------------------------------- replay

fn sample(track: &[Frame], time: f32) -> Frame {
    if track.is_empty() {
        return Frame { pos: [0.0, 0.0, HALF], rot: rest_pose(1, 0.0) };
    }
    let f = (time / FRAME_DT).max(0.0);
    let i = (f.floor() as usize).min(track.len() - 1);
    let a = track[i];
    if i + 1 >= track.len() {
        return a;
    }
    let w = (f - i as f32).clamp(0.0, 1.0);
    let b = track[i + 1];
    let pos = [
        a.pos[0] + (b.pos[0] - a.pos[0]) * w,
        a.pos[1] + (b.pos[1] - a.pos[1]) * w,
        a.pos[2] + (b.pos[2] - a.pos[2]) * w,
    ];
    Frame { pos, rot: m3_blend(&a.rot, &b.rot, w) }
}

fn poses(g: &Game) -> [Frame; 2] {
    match &g.dice_throw {
        Some(throw) => {
            let time = (throw.duration - g.dice_anim).clamp(0.0, throw.duration);
            [sample(&throw.frames[0], time), sample(&throw.frames[1], time)]
        }
        // A game loaded from disk has no recorded throw: show the two values
        // side by side, resting flat.
        None => [
            Frame { pos: [-HOME_X, 0.0, HALF], rot: rest_pose(g.dice.0, 0.06) },
            Frame { pos: [HOME_X, 0.0, HALF], rot: rest_pose(g.dice.1, -0.05) },
        ],
    }
}

/// Draw both dice centred on `(cx, cy)`, `size` pixels per die edge.
pub fn draw(g: &Game, cx: f32, cy: f32, size: f32) {
    let s = size.max(24.0);
    let frames = poses(g);
    // Further up the board (larger Y) means further from the viewer.
    let mut order = [0usize, 1];
    order.sort_by(|a, b| {
        frames[*b]
            .pos[1]
            .partial_cmp(&frames[*a].pos[1])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for i in order {
        draw_die(&frames[i], cx, cy, s);
    }
}
