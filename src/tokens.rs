//! Player tokens: eight designs drawn with primitives, so a game needs no
//! image assets. Each token is a chip in the player's colour with a light face
//! and a dark silhouette on it; the same drawing is reused on the board, in the
//! side panel and on the setup screen.

use macroquad::prelude::*;

use crate::i18n;

pub const COUNT: usize = 8;

const INK: Color = Color { r: 0.13, g: 0.13, b: 0.15, a: 1.0 };
const FACE_LIGHT: Color = Color { r: 0.97, g: 0.96, b: 0.92, a: 1.0 };

pub fn name(kind: usize) -> &'static str {
    match kind % COUNT {
        0 => i18n::t("Car", "Voiture"),
        1 => i18n::t("Top hat", "Haut-de-forme"),
        2 => i18n::t("Dog", "Chien"),
        3 => i18n::t("Boot", "Botte"),
        4 => i18n::t("Ship", "Bateau"),
        5 => i18n::t("Cat", "Chat"),
        6 => i18n::t("Plane", "Avion"),
        _ => i18n::t("Diamond", "Diamant"),
    }
}

// Helpers work in a unit square centred on the glyph: -0.5 .. 0.5 scaled by `s`.
fn rect(cx: f32, cy: f32, s: f32, x0: f32, y0: f32, x1: f32, y1: f32, col: Color) {
    draw_rectangle(cx + x0 * s, cy + y0 * s, (x1 - x0) * s, (y1 - y0) * s, col);
}

fn tri(
    cx: f32,
    cy: f32,
    s: f32,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    col: Color,
) {
    draw_triangle(
        vec2(cx + a.0 * s, cy + a.1 * s),
        vec2(cx + b.0 * s, cy + b.1 * s),
        vec2(cx + c.0 * s, cy + c.1 * s),
        col,
    );
}

fn disc(cx: f32, cy: f32, s: f32, x: f32, y: f32, r: f32, col: Color) {
    draw_circle(cx + x * s, cy + y * s, r * s, col);
}

fn stroke(
    cx: f32,
    cy: f32,
    s: f32,
    a: (f32, f32),
    b: (f32, f32),
    width: f32,
    col: Color,
) {
    draw_line(
        cx + a.0 * s,
        cy + a.1 * s,
        cx + b.0 * s,
        cy + b.1 * s,
        width * s,
        col,
    );
}

/// Silhouette for token `kind`, drawn inside a box of side `s`.
pub fn glyph(kind: usize, cx: f32, cy: f32, s: f32) {
    match kind % COUNT {
        // Car: cabin on a long body with two wheels.
        0 => {
            rect(cx, cy, s, -0.46, -0.02, 0.46, 0.22, INK);
            rect(cx, cy, s, -0.26, -0.26, 0.20, 0.02, INK);
            rect(cx, cy, s, -0.19, -0.19, 0.02, -0.05, FACE_LIGHT);
            rect(cx, cy, s, 0.06, -0.19, 0.13, -0.05, FACE_LIGHT);
            disc(cx, cy, s, -0.26, 0.24, 0.13, INK);
            disc(cx, cy, s, 0.26, 0.24, 0.13, INK);
            disc(cx, cy, s, -0.26, 0.24, 0.05, FACE_LIGHT);
            disc(cx, cy, s, 0.26, 0.24, 0.05, FACE_LIGHT);
            rect(cx, cy, s, 0.37, 0.04, 0.45, 0.11, FACE_LIGHT);
        }
        // Top hat: brim, tall crown and a light band.
        1 => {
            rect(cx, cy, s, -0.46, 0.16, 0.46, 0.28, INK);
            rect(cx, cy, s, -0.25, -0.40, 0.25, 0.18, INK);
            rect(cx, cy, s, -0.25, 0.04, 0.25, 0.14, FACE_LIGHT);
            rect(cx, cy, s, -0.25, -0.40, 0.25, -0.34, FACE_LIGHT);
        }
        // Dog: head, floppy ears and a snout.
        2 => {
            tri(cx, cy, s, (-0.30, -0.36), (-0.10, -0.20), (-0.30, -0.02), INK);
            tri(cx, cy, s, (0.30, -0.36), (0.10, -0.20), (0.30, -0.02), INK);
            disc(cx, cy, s, 0.0, -0.02, 0.26, INK);
            rect(cx, cy, s, -0.15, 0.12, 0.15, 0.34, INK);
            disc(cx, cy, s, 0.0, 0.22, 0.065, FACE_LIGHT);
            disc(cx, cy, s, -0.11, -0.06, 0.035, FACE_LIGHT);
            disc(cx, cy, s, 0.11, -0.06, 0.035, FACE_LIGHT);
        }
        // Boot: tall shaft, foot and heel.
        3 => {
            rect(cx, cy, s, -0.10, -0.40, 0.16, 0.16, INK);
            rect(cx, cy, s, -0.38, 0.16, 0.16, 0.30, INK);
            disc(cx, cy, s, -0.34, 0.23, 0.11, INK);
            rect(cx, cy, s, -0.10, 0.30, 0.16, 0.40, INK);
            rect(cx, cy, s, -0.38, 0.16, 0.16, 0.21, FACE_LIGHT);
        }
        // Ship: trapezoid hull, mast and two sails.
        4 => {
            tri(cx, cy, s, (-0.46, 0.12), (0.46, 0.12), (0.28, 0.34), INK);
            tri(cx, cy, s, (-0.46, 0.12), (0.28, 0.34), (-0.28, 0.34), INK);
            stroke(cx, cy, s, (0.0, -0.44), (0.0, 0.12), 0.07, INK);
            tri(cx, cy, s, (0.04, -0.38), (0.36, 0.06), (0.04, 0.06), INK);
            tri(cx, cy, s, (-0.04, -0.30), (-0.30, 0.06), (-0.04, 0.06), INK);
        }
        // Cat: round head, pointed ears, whiskers.
        5 => {
            tri(cx, cy, s, (-0.28, -0.38), (-0.06, -0.20), (-0.28, -0.06), INK);
            tri(cx, cy, s, (0.28, -0.38), (0.06, -0.20), (0.28, -0.06), INK);
            disc(cx, cy, s, 0.0, 0.02, 0.26, INK);
            disc(cx, cy, s, -0.10, -0.04, 0.035, FACE_LIGHT);
            disc(cx, cy, s, 0.10, -0.04, 0.035, FACE_LIGHT);
            tri(cx, cy, s, (-0.04, 0.08), (0.04, 0.08), (0.0, 0.14), FACE_LIGHT);
            stroke(cx, cy, s, (0.10, 0.10), (0.42, 0.04), 0.02, INK);
            stroke(cx, cy, s, (0.10, 0.13), (0.42, 0.14), 0.02, INK);
            stroke(cx, cy, s, (-0.10, 0.10), (-0.42, 0.04), 0.02, INK);
            stroke(cx, cy, s, (-0.10, 0.13), (-0.42, 0.14), 0.02, INK);
        }
        // Plane: swept wings, slim body, tail.
        6 => {
            tri(cx, cy, s, (-0.48, 0.08), (0.48, 0.08), (0.0, -0.14), INK);
            rect(cx, cy, s, -0.30, 0.24, 0.30, 0.32, INK);
            rect(cx, cy, s, -0.08, -0.44, 0.08, 0.34, INK);
            tri(cx, cy, s, (-0.08, -0.44), (0.08, -0.44), (0.0, -0.50), INK);
            rect(cx, cy, s, -0.04, -0.02, 0.04, 0.14, FACE_LIGHT);
        }
        // Diamond: faceted gem.
        _ => {
            tri(cx, cy, s, (-0.38, -0.10), (0.38, -0.10), (0.0, 0.44), INK);
            tri(cx, cy, s, (-0.38, -0.10), (0.22, -0.34), (-0.22, -0.34), INK);
            tri(cx, cy, s, (-0.38, -0.10), (0.38, -0.10), (0.22, -0.34), INK);
            stroke(cx, cy, s, (-0.38, -0.10), (0.0, 0.44), 0.03, FACE_LIGHT);
            stroke(cx, cy, s, (0.38, -0.10), (0.0, 0.44), 0.03, FACE_LIGHT);
            stroke(cx, cy, s, (-0.38, -0.10), (0.38, -0.10), 0.03, FACE_LIGHT);
        }
    }
}

/// Full token: coloured ring, light face, dark glyph.
pub fn draw(kind: usize, color: Color, cx: f32, cy: f32, radius: f32) {
    draw_circle(cx, cy + radius * 0.12, radius * 1.04, Color { r: 0.06, g: 0.05, b: 0.05, a: 0.45 });
    draw_circle(cx, cy, radius, color);
    draw_circle_lines(cx, cy, radius - 0.5, (radius * 0.14).max(1.0), Color { r: 0.07, g: 0.07, b: 0.08, a: 0.9 });
    draw_circle(cx, cy, radius * 0.78, FACE_LIGHT);
    glyph(kind, cx, cy, radius * 1.42);
}
