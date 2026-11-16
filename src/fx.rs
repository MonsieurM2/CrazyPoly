//! Juice: sparkles, coins, confetti, floating numbers, screen shake and
//! flashes.
//!
//! Purely decorative. The interface pushes effects when something happens and
//! this module keeps them alive, moves them and draws them; nothing here can
//! change the rules of the game.

use macroquad::prelude::*;
use macroquad::rand::gen_range;

/// Hard cap on live particles: a burst of events must never cost a frame.
const MAX_PARTS: usize = 1400;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// A round spark that fades.
    Dot,
    /// A gold coin that pops, tumbles and falls.
    Coin,
    /// A coloured strip that flutters down.
    Confetti,
    /// A four-point sparkle.
    Star,
}

struct Part {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max: f32,
    size: f32,
    color: Color,
    gravity: f32,
    drag: f32,
    shape: Shape,
    spin: f32,
    spin_v: f32,
}

struct Label {
    text: String,
    pos: Vec2,
    life: f32,
    max: f32,
    color: Color,
    size: f32,
}

pub struct Fx {
    parts: Vec<Part>,
    labels: Vec<Label>,
    /// Remaining shake, in pixels, at the centre of the table.
    shake: f32,
    clock: f32,
    flash: Color,
    flash_a: f32,
}

impl Fx {
    pub fn new() -> Self {
        Fx {
            parts: Vec::new(),
            labels: Vec::new(),
            shake: 0.0,
            clock: 0.0,
            flash: Color::new(1.0, 1.0, 1.0, 0.0),
            flash_a: 0.0,
        }
    }

    /// Forgets everything, for a fresh game.
    pub fn clear(&mut self) {
        self.parts.clear();
        self.labels.clear();
        self.shake = 0.0;
        self.flash_a = 0.0;
    }

    fn push(&mut self, part: Part) {
        if self.parts.len() >= MAX_PARTS {
            self.parts.remove(0);
        }
        self.parts.push(part);
    }

    // ------------------------------------------------------------- emitters

    /// A burst of round sparks, for something bright and small.
    pub fn spark(&mut self, at: Vec2, color: Color, n: usize) {
        for _ in 0..n {
            let a = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(60.0, 240.0);
            let life = gen_range(0.30, 0.65);
            self.push(Part {
                pos: at,
                vel: vec2(a.cos() * speed, a.sin() * speed - 40.0),
                life,
                max: life,
                size: gen_range(1.6, 3.6),
                color,
                gravity: 220.0,
                drag: 0.10,
                shape: Shape::Dot,
                spin: 0.0,
                spin_v: 0.0,
            });
        }
    }

    /// Gold coins thrown up out of a tile.
    pub fn coins(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let life = gen_range(0.55, 0.95);
            self.push(Part {
                pos: at + vec2(gen_range(-14.0, 14.0), gen_range(-6.0, 6.0)),
                vel: vec2(gen_range(-70.0, 70.0), gen_range(-320.0, -190.0)),
                life,
                max: life,
                size: gen_range(4.0, 7.0),
                color: Color::new(0.98, 0.82, 0.36, 1.0),
                gravity: 760.0,
                drag: 0.02,
                shape: Shape::Coin,
                spin: gen_range(0.0, 6.0),
                spin_v: gen_range(-9.0, 9.0),
            });
        }
    }

    /// Confetti, for a win or a big moment.
    pub fn confetti(&mut self, at: Vec2, n: usize) {
        const COLORS: [Color; 6] = [
            Color::new(0.98, 0.82, 0.36, 1.0),
            Color::new(0.95, 0.42, 0.42, 1.0),
            Color::new(0.42, 0.85, 0.62, 1.0),
            Color::new(0.45, 0.70, 0.98, 1.0),
            Color::new(0.95, 0.62, 0.90, 1.0),
            Color::new(0.99, 0.98, 0.92, 1.0),
        ];
        for k in 0..n {
            let life = gen_range(1.1, 2.0);
            let a = gen_range(-std::f32::consts::PI, 0.0);
            let speed = gen_range(120.0, 420.0);
            self.push(Part {
                pos: at + vec2(gen_range(-16.0, 16.0), gen_range(-8.0, 8.0)),
                vel: vec2(a.cos() * speed, a.sin() * speed - 60.0),
                life,
                max: life,
                size: gen_range(3.5, 7.5),
                color: COLORS[k % COLORS.len()],
                gravity: 420.0,
                drag: 0.55,
                shape: Shape::Confetti,
                spin: gen_range(0.0, 6.0),
                spin_v: gen_range(-12.0, 12.0),
            });
        }
    }

    /// Confetti falling from the top of the screen, for the end of a game.
    pub fn rain(&mut self, width: f32, n: usize) {
        for _ in 0..n {
            let life = gen_range(2.2, 3.6);
            self.push(Part {
                pos: vec2(gen_range(0.0, width), gen_range(-40.0, 10.0)),
                vel: vec2(gen_range(-40.0, 40.0), gen_range(60.0, 170.0)),
                life,
                max: life,
                size: gen_range(4.0, 8.0),
                color: Color::new(
                    gen_range(0.35, 1.0),
                    gen_range(0.35, 1.0),
                    gen_range(0.35, 1.0),
                    1.0,
                ),
                gravity: 60.0,
                drag: 0.25,
                shape: Shape::Confetti,
                spin: gen_range(0.0, 6.0),
                spin_v: gen_range(-10.0, 10.0),
            });
        }
    }

    /// Four-point sparkles, used on purchases and on the banner.
    pub fn stars(&mut self, at: Vec2, color: Color, n: usize) {
        for _ in 0..n {
            let a = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(20.0, 130.0);
            let life = gen_range(0.5, 0.9);
            self.push(Part {
                pos: at + vec2(gen_range(-24.0, 24.0), gen_range(-16.0, 16.0)),
                vel: vec2(a.cos() * speed, a.sin() * speed),
                life,
                max: life,
                size: gen_range(4.0, 9.0),
                color,
                gravity: 0.0,
                drag: 0.02,
                shape: Shape::Star,
                spin: gen_range(0.0, 3.0),
                spin_v: gen_range(-3.0, 3.0),
            });
        }
    }

    /// A number that rises and fades, in the colour of what it means.
    pub fn float(&mut self, text: impl Into<String>, at: Vec2, color: Color, size: f32) {
        self.labels.push(Label {
            text: text.into(),
            pos: at,
            life: 1.35,
            max: 1.35,
            color,
            size,
        });
        if self.labels.len() > 24 {
            self.labels.remove(0);
        }
    }

    /// Shakes the table. Amounts between 2 (a purchase) and 12 (a bankruptcy).
    pub fn shake(&mut self, amount: f32) {
        self.shake = self.shake.max(amount);
    }

    /// Tints the whole screen for a moment.
    pub fn flash(&mut self, color: Color, strength: f32) {
        self.flash = Color::new(color.r, color.g, color.b, 1.0);
        self.flash_a = self.flash_a.max(strength);
    }

    // --------------------------------------------------------------- motion

    pub fn update(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        self.clock += dt;
        self.shake = (self.shake - self.shake * 9.0 * dt - 6.0 * dt).max(0.0);
        if self.shake < 0.05 {
            self.shake = 0.0;
        }
        self.flash_a = (self.flash_a - 2.6 * dt).max(0.0);

        for p in self.parts.iter_mut() {
            p.life -= dt;
            p.vel.y += p.gravity * dt;
            let damp = (1.0 - p.drag * dt).clamp(0.0, 1.0);
            p.vel *= damp;
            p.pos += p.vel * dt;
            p.spin += p.spin_v * dt;
        }
        self.parts.retain(|p| p.life > 0.0);

        for l in self.labels.iter_mut() {
            l.life -= dt;
            l.pos.y -= 34.0 * dt;
        }
        self.labels.retain(|l| l.life > 0.0);
    }

    /// How far the table is displaced this frame.
    pub fn offset(&self) -> (f32, f32) {
        if self.shake <= 0.0 {
            return (0.0, 0.0);
        }
        let t = self.clock * 57.0;
        (
            (t.sin() * self.shake).round(),
            ((t * 0.83 + 1.7).sin() * self.shake * 0.75).round(),
        )
    }

    pub fn busy(&self) -> bool {
        self.shake > 0.0 || self.flash_a > 0.0 || !self.parts.is_empty()
    }

    // ---------------------------------------------------------------- paint

    /// Draws the particles.
    pub fn draw(&self) {
        for p in &self.parts {
            let k = (p.life / p.max).clamp(0.0, 1.0);
            let fade = if k < 0.35 { k / 0.35 } else { 1.0 };
            match p.shape {
                Shape::Dot => {
                    let c = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * fade);
                    draw_circle(p.pos.x, p.pos.y, p.size * (0.4 + 0.6 * k), c);
                }
                Shape::Coin => {
                    // A coin seen edge-on as it tumbles: a narrow ellipse.
                    let squash = p.spin.cos().abs().max(0.18);
                    let c = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * fade);
                    draw_ellipse(
                        p.pos.x,
                        p.pos.y,
                        p.size * squash,
                        p.size,
                        0.0,
                        c,
                    );
                    draw_ellipse(
                        p.pos.x,
                        p.pos.y,
                        p.size * squash * 0.55,
                        p.size * 0.55,
                        0.0,
                        Color::new(1.0, 0.96, 0.78, 0.75 * fade),
                    );
                }
                Shape::Confetti => {
                    let c = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * fade);
                    draw_rectangle_ex(
                        p.pos.x - p.size * 0.5,
                        p.pos.y - p.size * 0.35,
                        p.size,
                        p.size * 0.7,
                        DrawRectangleParams {
                            offset: vec2(0.5, 0.5),
                            rotation: p.spin,
                            color: c,
                        },
                    );
                }
                Shape::Star => {
                    let r = p.size * (0.3 + 0.7 * k);
                    let c = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * fade);
                    draw_line(p.pos.x - r, p.pos.y, p.pos.x + r, p.pos.y, 1.6, c);
                    draw_line(p.pos.x, p.pos.y - r, p.pos.x, p.pos.y + r, 1.6, c);
                    draw_circle(p.pos.x, p.pos.y, 1.3, c);
                }
            }
        }

    }

    /// Draws the floating numbers with the game's own font, on top of the
    /// particles: `text` prints a string at its baseline.
    pub fn draw_labels<F: Fn(&str, f32, f32, f32, Color)>(&self, text: F) {
        for l in &self.labels {
            let k = l.life / l.max;
            let fade = if k < 0.30 { k / 0.30 } else { 1.0 };
            let age = 1.0 - k;
            let pop = if age < 0.16 {
                0.75 + 0.25 * (age / 0.16) + 0.15 * ((age / 0.16) * std::f32::consts::PI).sin()
            } else {
                1.0
            };
            let c = Color::new(l.color.r, l.color.g, l.color.b, l.color.a * fade);
            text(&l.text, l.pos.x - 0.5, l.pos.y, l.size * pop, c);
        }
    }


    /// The full-screen tint of a flash, drawn above everything else.
    pub fn draw_flash(&self) {
        if self.flash_a <= 0.001 {
            return;
        }
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(self.flash.r, self.flash.g, self.flash.b, self.flash_a.min(0.5)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_expire_and_leave_nothing_behind() {
        let mut fx = Fx::new();
        fx.spark(vec2(10.0, 10.0), Color::new(1.0, 1.0, 1.0, 1.0), 6);
        fx.coins(vec2(5.0, 5.0), 3);
        fx.stars(vec2(1.0, 1.0), Color::new(1.0, 1.0, 1.0, 1.0), 4);
        assert_eq!(fx.parts.len(), 13);
        for _ in 0..120 {
            fx.update(0.05);
        }
        assert!(fx.parts.is_empty(), "particles must not pile up");
        assert!(fx.labels.is_empty());
    }

    #[test]
    fn a_shake_settles_quickly() {
        let mut fx = Fx::new();
        fx.shake(10.0);
        fx.update(0.016);
        assert!(fx.offset().0.abs() <= 10.0);
        assert!(fx.offset().0.abs() > 0.0, "a shake must move the table");
        for _ in 0..40 {
            fx.update(0.05);
        }
        assert_eq!(fx.offset(), (0.0, 0.0), "the table must come to rest");
        assert!(!fx.busy());
    }

    #[test]
    fn the_particle_cap_holds() {
        let mut fx = Fx::new();
        for _ in 0..40 {
            fx.rain(400.0, 100);
        }
        assert!(fx.parts.len() <= MAX_PARTS);
    }
}
