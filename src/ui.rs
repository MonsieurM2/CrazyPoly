//! Rendering and input: setup screen, board, side panel and modal dialogs.
//!
//! The interface uses a bundled Lato font (regular + bold) for legibility and
//! accented French, scales with the window, and fills the width next to the
//! square board instead of leaving dead space.

use std::cell::RefCell;

use macroquad::prelude::*;

use crate::board::{Group, Kind, Space};
use crate::game::{Game, Phase, TradeState, MAX_HOUSES};
use crate::i18n;
use crate::tokens;
use crate::{App, Screen};

pub use crate::game::PLAYER_COLORS as TOKENS;

// ------------------------------------------------------------------ palette
//
// The whole game sits on a dark green felt table, with cream cards, gold
// lettering and one wine red accent, like a deluxe edition of the board game.

const BG_DEEP: Color = Color { r: 0.035, g: 0.110, b: 0.086, a: 1.0 };
const BG_LIGHT: Color = Color { r: 0.086, g: 0.216, b: 0.165, a: 1.0 };
const CARD: Color = Color { r: 0.063, g: 0.176, b: 0.133, a: 1.0 };
const CARD_SOFT: Color = Color { r: 0.086, g: 0.216, b: 0.165, a: 1.0 };
const CARD_EDGE: Color = Color { r: 0.216, g: 0.384, b: 0.302, a: 1.0 };
const GOLD: Color = Color { r: 0.851, g: 0.643, b: 0.251, a: 1.0 };
const GOLD_SOFT: Color = Color { r: 0.945, g: 0.835, b: 0.588, a: 1.0 };
const GOLD_DIM: Color = Color { r: 0.694, g: 0.529, b: 0.247, a: 1.0 };
const WINE: Color = Color { r: 0.545, g: 0.114, b: 0.114, a: 1.0 };
const WOOD: Color = Color { r: 0.396, g: 0.267, b: 0.161, a: 1.0 };
const WOOD_EDGE: Color = Color { r: 0.263, g: 0.169, b: 0.094, a: 1.0 };
const CREAM: Color = Color { r: 0.961, g: 0.929, b: 0.855, a: 1.0 };
const CREAM_DIM: Color = Color { r: 0.804, g: 0.776, b: 0.694, a: 1.0 };
const DIM_GREEN: Color = Color { r: 0.639, g: 0.678, b: 0.635, a: 1.0 };

// Names kept for the rest of the module.
const PANEL_BG: Color = CARD;
const MODAL_BG: Color = CARD;
const MODAL_HEADER: Color = CARD_SOFT;
const PANEL_EDGE: Color = CARD_EDGE;
const TEXT: Color = CREAM;
const TEXT_DIM: Color = CREAM_DIM;
const MUTED: Color = DIM_GREEN;
const ACCENT: Color = GOLD;
const ACCENT_SOFT: Color = GOLD_SOFT;
const WARN: Color = Color { r: 0.949, g: 0.706, b: 0.302, a: 1.0 };
const BTN: Color = CARD_SOFT;
const BTN_HOVER: Color = Color { r: 0.117, g: 0.278, b: 0.212, a: 1.0 };
const BTN_OFF: Color = Color { r: 0.051, g: 0.141, b: 0.106, a: 1.0 };
const BOARD_BG: Color = Color { r: 0.965, g: 0.937, b: 0.867, a: 1.0 };
const CENTER_BG: Color = Color { r: 0.980, g: 0.957, b: 0.894, a: 1.0 };
const TILE: Color = Color { r: 0.988, g: 0.973, b: 0.929, a: 1.0 };
const INK: Color = Color { r: 0.106, g: 0.235, b: 0.180, a: 1.0 };
const INK_SOFT: Color = Color { r: 0.502, g: 0.388, b: 0.235, a: 1.0 };
const INK_LINE: Color = Color { r: 0.812, g: 0.769, b: 0.675, a: 1.0 };
const ROW_BG: Color = CARD_SOFT;
const HOUSE_COL: Color = Color { r: 0.09, g: 0.451, b: 0.259, a: 1.0 };
const HOTEL_COL: Color = Color { r: 0.545, g: 0.114, b: 0.114, a: 1.0 };
const HOVER: Color = Color { r: 0.851, g: 0.643, b: 0.251, a: 0.20 };
const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
const SELECT: Color = Color { r: 0.129, g: 0.310, b: 0.235, a: 1.0 };

// -------------------------------------------------------------------- fonts

thread_local! {
    static FONTS: RefCell<(Option<Font>, Option<Font>, Option<Font>)> = RefCell::new((None, None, None));
}

/// Load the embedded faces once the window exists: Lato for the interface and
/// Playfair Display for the logo and the big headings.
pub fn load_fonts() {
    let regular = load_ttf_font_from_bytes(&include_bytes!("../assets/Lato-Regular.ttf")[..]).ok();
    let bold = load_ttf_font_from_bytes(&include_bytes!("../assets/Lato-Bold.ttf")[..]).ok();
    let serif = load_ttf_font_from_bytes(&include_bytes!("../assets/PlayfairDisplay.ttf")[..]).ok();
    FONTS.with(|cell| *cell.borrow_mut() = (regular, bold, serif));
}

fn font_size(size: f32) -> u16 {
    size.max(1.0).round() as u16
}

fn with_font<R>(bold: bool, f: impl FnOnce(Option<&Font>) -> R) -> R {
    FONTS.with(|cell| {
        let borrow = cell.borrow();
        let font = if bold { borrow.1.as_ref() } else { borrow.0.as_ref() };
        f(font)
    })
}

fn with_serif<R>(f: impl FnOnce(Option<&Font>) -> R) -> R {
    FONTS.with(|cell| f(cell.borrow().2.as_ref()))
}

fn serif(text: &str, x: f32, y: f32, size: f32, color: Color) {
    with_serif(|font| {
        let params = TextParams {
            font,
            font_size: font_size(size),
            color,
            ..Default::default()
        };
        draw_text_ex(text, x, y, params);
    });
}

/// The bundled Playfair is a single weight: a hair of extra ink fakes the bold
/// cut the design calls for.
fn serif_bold(text: &str, x: f32, y: f32, size: f32, color: Color) {
    serif(text, x, y, size, color);
    serif(text, x + size * 0.013, y, size, color);
}

fn serif_w(text: &str, size: f32) -> f32 {
    with_serif(|font| measure_text(text, font, font_size(size), 1.0).width)
}

fn serif_center(text: &str, cx: f32, cy: f32, size: f32, color: Color) {
    with_serif(|font| {
        let d = measure_text(text, font, font_size(size), 1.0);
        let params = TextParams {
            font,
            font_size: font_size(size),
            color,
            ..Default::default()
        };
        draw_text_ex(text, cx - d.width * 0.5, cy + d.height * 0.34, params.clone());
        draw_text_ex(
            text,
            cx - d.width * 0.5 + size * 0.013,
            cy + d.height * 0.34,
            params,
        );
    });
}

fn serif_left(text: &str, x: f32, baseline: f32, size: f32, color: Color) {
    serif_bold(text, x, baseline, size, color);
}

fn serif_right(text: &str, right: f32, baseline: f32, size: f32, color: Color) {
    serif_bold(text, right - serif_w(text, size) - size * 0.013, baseline, size, color);
}

fn measure(text: &str, size: f32, bold: bool) -> TextDimensions {
    with_font(bold, |font| measure_text(text, font, font_size(size), 1.0))
}

fn draw_raw(text: &str, x: f32, y: f32, size: f32, color: Color, bold: bool) {
    with_font(bold, |font| {
        let params = TextParams {
            font,
            font_size: font_size(size),
            color,
            ..Default::default()
        };
        draw_text_ex(text, x, y, params);
    });
}

fn text_w(text: &str, size: f32) -> f32 {
    measure(text, size, false).width
}

fn txt(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_raw(text, x, y, size, color, false);
}

fn txt_bold(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_raw(text, x, y, size, color, true);
}

fn centered(text: &str, cx: f32, cy: f32, size: f32, color: Color) {
    let d = measure(text, size, false);
    draw_raw(text, cx - d.width * 0.5, cy + d.height * 0.34, size, color, false);
}

fn centered_bold(text: &str, cx: f32, cy: f32, size: f32, color: Color) {
    let d = measure(text, size, true);
    draw_raw(text, cx - d.width * 0.5, cy + d.height * 0.34, size, color, true);
}

fn text_r(text: &str, right: f32, baseline: f32, size: f32, color: Color) {
    draw_raw(text, right - measure(text, size, false).width, baseline, size, color, false);
}

fn text_r_bold(text: &str, right: f32, baseline: f32, size: f32, color: Color) {
    draw_raw(text, right - measure(text, size, true).width, baseline, size, color, true);
}

fn inr(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h
}

fn money(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    let digits = v.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    format!("{}{}$", sign, out)
}

fn wrap(text: &str, max_w: f32, size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };
        if text_w(&candidate, size) <= max_w || current.is_empty() {
            current = candidate;
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn fit_wrap(text: &str, max_w: f32, max_lines: usize, start: f32) -> (Vec<String>, f32) {
    let mut size = start;
    loop {
        let lines = wrap(text, max_w, size);
        let fits = lines.len() <= max_lines && lines.iter().all(|l| text_w(l, size) <= max_w + 0.6);
        if fits || size <= 6.0 {
            return (lines, size);
        }
        size -= 0.4;
    }
}

fn truncate(text: &str, max_w: f32, size: f32) -> String {
    if text_w(text, size) <= max_w {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}", out, ch);
        if text_w(&format!("{}...", candidate), size) > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out)
}

// ------------------------------------------------------------------- shapes

fn round_rect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    let r = r.min(w * 0.5).min(h * 0.5).max(0.0);
    if r < 1.0 {
        draw_rectangle(x, y, w, h, color);
        return;
    }
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, r, h - 2.0 * r, color);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, color);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

fn round_outline(x: f32, y: f32, w: f32, h: f32, r: f32, thickness: f32, color: Color) {
    let r = r.min(w * 0.5).min(h * 0.5).max(0.0);
    draw_line(x + r, y, x + w - r, y, thickness, color);
    draw_line(x + r, y + h, x + w - r, y + h, thickness, color);
    draw_line(x, y + r, x, y + h - r, thickness, color);
    draw_line(x + w, y + r, x + w, y + h - r, thickness, color);
    let corners = [
        (x + r, y + r, std::f32::consts::PI),
        (x + w - r, y + r, 1.5 * std::f32::consts::PI),
        (x + w - r, y + h - r, 0.0),
        (x + r, y + h - r, 0.5 * std::f32::consts::PI),
    ];
    for (cx, cy, a0) in corners {
        let mut prev = (cx + r * a0.cos(), cy + r * a0.sin());
        for k in 1..=6 {
            let a = a0 + 0.5 * std::f32::consts::PI * k as f32 / 6.0;
            let p = (cx + r * a.cos(), cy + r * a.sin());
            draw_line(prev.0, prev.1, p.0, p.1, thickness, color);
            prev = p;
        }
    }
}

fn round_card(x: f32, y: f32, w: f32, h: f32, r: f32, fill: Color, edge: Color) {
    round_rect(x, y, w, h, r, edge);
    round_rect(x + 1.0, y + 1.0, w - 2.0, h - 2.0, (r - 1.0).max(0.0), fill);
}

/// The felt table behind everything: a soft vertical gradient.
fn felt_background() {
    let h = screen_height();
    let steps = 40;
    let mut y = 0.0;
    for k in 0..steps {
        let t = k as f32 / (steps - 1) as f32;
        let color = Color {
            r: BG_DEEP.r + (BG_LIGHT.r - BG_DEEP.r) * t,
            g: BG_DEEP.g + (BG_LIGHT.g - BG_DEEP.g) * t,
            b: BG_DEEP.b + (BG_LIGHT.b - BG_DEEP.b) * t,
            a: 1.0,
        };
        let next = (k + 1) as f32 * (h / steps as f32);
        draw_rectangle(0.0, y, screen_width(), next - y + 1.0, color);
        y = next;
    }
}

fn button(r: Rect, label: &str, enabled: bool, primary: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = enabled && inr(r, mx, my);
    let radius = (r.h * 0.30).min(12.0);
    let mut size = (r.h * 0.36).clamp(10.0, 16.0);
    let label = if measure(label, size, true).width > r.w - 16.0 {
        while size > 9.5 && measure(label, size, true).width > r.w - 16.0 {
            size -= 0.5;
        }
        truncate(label, r.w - 16.0, size)
    } else {
        label.to_string()
    };
    let dims = measure(&label, size, true);
    let tx = r.x + (r.w - dims.width) * 0.5;
    let ty = r.y + r.h * 0.5 + dims.height * 0.34;
    if primary {
        let fill = if !enabled {
            Color { r: 0.235, g: 0.235, b: 0.161, a: 1.0 }
        } else if hover {
            GOLD_SOFT
        } else {
            GOLD
        };
        round_rect(r.x, r.y, r.w, r.h, radius, fill);
        let fg = if enabled {
            Color { r: 0.075, g: 0.157, b: 0.114, a: 1.0 }
        } else {
            Color { r: 0.545, g: 0.541, b: 0.451, a: 1.0 }
        };
        draw_raw(&label, tx, ty, size, fg, true);
    } else {
        if hover {
            round_rect(r.x, r.y, r.w, r.h, radius, Color { r: 0.145, g: 0.310, b: 0.231, a: 1.0 });
        }
        let edge = if !enabled {
            Color { r: 0.153, g: 0.251, b: 0.204, a: 1.0 }
        } else if hover {
            GOLD
        } else {
            GOLD_DIM
        };
        round_outline(r.x, r.y, r.w, r.h, radius, 1.2, edge);
        let fg = if enabled { CREAM } else { Color { r: 0.357, g: 0.435, b: 0.376, a: 1.0 } };
        draw_raw(&label, tx, ty, size, fg, true);
    }
    let clicked = enabled && hover && is_mouse_button_pressed(MouseButton::Left);
    if clicked {
        crate::audio::sfx(crate::audio::Sfx::Click);
    }
    clicked
}

fn small_button(r: Rect, label: &str, enabled: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = enabled && inr(r, mx, my);
    if hover {
        round_rect(r.x, r.y, r.w, r.h, 6.0, Color { r: 0.145, g: 0.310, b: 0.231, a: 1.0 });
    }
    round_outline(
        r.x,
        r.y,
        r.w,
        r.h,
        6.0,
        1.0,
        if enabled { GOLD_DIM } else { Color { r: 0.153, g: 0.251, b: 0.204, a: 1.0 } },
    );
    let size = (r.h * 0.55).clamp(9.0, 15.0);
    centered(label, r.x + r.w * 0.5, r.y + r.h * 0.5, size, if enabled { CREAM } else { MUTED });
    let clicked = enabled && hover && is_mouse_button_pressed(MouseButton::Left);
    if clicked {
        crate::audio::sfx(crate::audio::Sfx::Click);
    }
    clicked
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Icon {
    Music,
    Sfx,
    Pause,
}

/// A pill that stays lit while its option is on.
fn toggle_button(r: Rect, label: &str, on: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = inr(r, mx, my);
    if on {
        round_rect(
            r.x,
            r.y,
            r.w,
            r.h,
            8.0,
            if hover { GOLD_SOFT } else { GOLD },
        );
        centered_bold(label, r.x + r.w * 0.5, r.y + r.h * 0.5, 12.0, BG_DEEP);
    } else {
        if hover {
            round_rect(
                r.x,
                r.y,
                r.w,
                r.h,
                8.0,
                Color { r: 0.145, g: 0.310, b: 0.231, a: 1.0 },
            );
        }
        round_outline(r.x, r.y, r.w, r.h, 8.0, 1.2, GOLD_DIM);
        centered_bold(label, r.x + r.w * 0.5, r.y + r.h * 0.5, 12.0, CREAM);
    }
    let clicked = hover && is_mouse_button_pressed(MouseButton::Left);
    if clicked {
        crate::audio::sfx(crate::audio::Sfx::Click);
    }
    clicked
}

/// Square icon button for the music and the sound effects: a note and an
/// equaliser, struck through when the channel is off.
fn sound_button(r: Rect, icon: Icon, on: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = inr(r, mx, my);
    if hover {
        round_rect(
            r.x,
            r.y,
            r.w,
            r.h,
            8.0,
            Color { r: 0.145, g: 0.310, b: 0.231, a: 1.0 },
        );
    }
    round_outline(
        r.x,
        r.y,
        r.w,
        r.h,
        8.0,
        1.0,
        if on {
            GOLD_DIM
        } else {
            Color { r: 0.153, g: 0.251, b: 0.204, a: 1.0 }
        },
    );
    let c = if on { GOLD_SOFT } else { MUTED };
    let cx = r.x + r.w * 0.5;
    let cy = r.y + r.h * 0.5;
    match icon {
        Icon::Music => {
            draw_rectangle(cx - 1.2, cy - 7.0, 1.8, 11.0, c);
            draw_rectangle(cx + 0.4, cy - 7.0, 5.2, 2.0, c);
            draw_circle(cx - 3.6, cy + 4.4, 3.2, c);
        }
        Icon::Pause => {
            round_rect(cx - 5.0, cy - 6.5, 3.2, 13.0, 1.4, c);
            round_rect(cx + 1.8, cy - 6.5, 3.2, 13.0, 1.4, c);
        }
        Icon::Sfx => {
            for (k, h) in [7.0f32, 12.0, 9.0].iter().enumerate() {
                let hh = if on { *h } else { 4.0 };
                round_rect(
                    cx - 7.4 + k as f32 * 5.0,
                    cy - hh * 0.5,
                    2.6,
                    hh,
                    1.2,
                    c,
                );
            }
        }
    }
    if !on {
        draw_line(cx - 8.0, cy + 8.0, cx + 8.0, cy - 8.0, 1.5, MUTED);
    }
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn modal_rect(w: f32, h: f32) -> Rect {
    let w = w.min(screen_width() - 24.0);
    let h = h.min(screen_height() - 24.0);
    Rect::new((screen_width() - w) * 0.5, (screen_height() - h) * 0.5, w, h)
}

fn modal_shell(r: Rect, title: &str) -> Rect {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color { r: 0.012, g: 0.043, b: 0.031, a: 0.72 },
    );
    // Drop shadow, card, gold rule under a serif title.
    round_rect(r.x + 4.0, r.y + 8.0, r.w, r.h, 14.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.28 });
    round_card(r.x, r.y, r.w, r.h, 14.0, MODAL_BG, CARD_EDGE);
    round_rect(r.x, r.y, r.w, 54.0, 14.0, MODAL_HEADER);
    draw_rectangle(r.x, r.y + 40.0, r.w, 14.0, MODAL_HEADER);
    serif_center(title, r.x + r.w * 0.5, r.y + 27.0, 23.0, GOLD_SOFT);
    draw_line(r.x + 26.0, r.y + 54.0, r.x + r.w - 26.0, r.y + 54.0, 1.0, GOLD_DIM);
    Rect::new(r.x + 22.0, r.y + 66.0, r.w - 44.0, r.h - 86.0)
}

fn group_title(group: Group) -> &'static str {
    match group {
        Group::Brown => i18n::t("Brown", "Marron"),
        Group::LightBlue => i18n::t("Light blue", "Bleu clair"),
        Group::Pink => i18n::t("Pink", "Rose"),
        Group::Orange => i18n::t("Orange", "Orange"),
        Group::Red => i18n::t("Red", "Rouge"),
        Group::Yellow => i18n::t("Yellow", "Jaune"),
        Group::Green => i18n::t("Green", "Vert"),
        Group::DarkBlue => i18n::t("Dark blue", "Bleu foncé"),
        Group::Railroad => i18n::t("Railroad", "Chemin de fer"),
        Group::Utility => i18n::t("Utility", "Service"),
        Group::Special => "",
    }
}

fn list_names(g: &Game, tiles: &[usize]) -> String {
    if tiles.is_empty() {
        return i18n::t("nothing", "rien").to_string();
    }
    tiles
        .iter()
        .map(|t| g.spaces[*t].name)
        .collect::<Vec<&str>>()
        .join(", ")
}

// -------------------------------------------------------------------- state

pub struct Ui {
    pub asset_open: bool,
    pub asset_page: usize,
    /// Lot whose title deed is being read in the asset window.
    pub asset_card: Option<usize>,
    pub confirm_bankrupt: bool,
    pub rules_open: bool,
    pub rules_page: usize,
    /// Escape menu: the whole table holds still while it is open.
    pub pause_open: bool,
    /// Save picker: open when several saves compete for CONTINUE.
    pub load_open: bool,
    pub load_list: Vec<crate::persist::SaveInfo>,
    pub load_note: String,
    /// The click that opened the picker must not pick a save.
    pub load_armed: bool,
    /// Vote to end the game early: one "agreed" flag per seat.
    pub vote_open: bool,
    pub vote_agree: Vec<bool>,
    pub quit_confirm: bool,
}

impl Ui {
    pub fn new() -> Self {
        Ui {
            asset_open: false,
            asset_page: 0,
            asset_card: None,
            confirm_bankrupt: false,
            rules_open: false,
            rules_page: 0,
            pause_open: false,
            load_open: false,
            load_list: Vec::new(),
            load_note: String::new(),
            load_armed: false,
            vote_open: false,
            vote_agree: Vec::new(),
            quit_confirm: false,
        }
    }
}

pub struct Slot {
    pub name: String,
    pub ai: bool,
    pub token: usize,
}

pub struct Setup {
    pub count: usize,
    pub slots: Vec<Slot>,
    pub editing: Option<usize>,
    /// Seat whose token is being chosen, if the picker is open.
    pub token_slot: Option<usize>,
    pub error: String,
}

impl Default for Setup {
    fn default() -> Self {
        let mut slots = Vec::new();
        for i in 0..8 {
            slots.push(Slot {
                name: format!("{} {}", i18n::t("Player", "Joueur"), i + 1),
                ai: i >= 4,
                token: i % tokens::COUNT,
            });
        }
        Setup {
            count: 2,
            slots,
            editing: None,
            token_slot: None,
            error: String::new(),
        }
    }
}

impl Setup {
    /// Keeps the generated names in the language currently on screen.
    pub fn relabel_defaults(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            let n = i + 1;
            if slot.name == format!("Player {}", n) || slot.name == format!("Joueur {}", n) {
                slot.name = format!("{} {}", i18n::t("Player", "Joueur"), n);
            }
        }
    }

    pub fn build_players(&self) -> Vec<crate::game::Player> {
        (0..self.count)
            .map(|i| {
                let slot = &self.slots[i];
                let trimmed = slot.name.trim();
                let name = if trimmed.is_empty() {
                    format!("{} {}", i18n::t("Player", "Joueur"), i + 1)
                } else {
                    trimmed.to_string()
                };
                let mut player = crate::game::Player::new(name, TOKENS[i % TOKENS.len()], slot.ai);
                player.token = slot.token % tokens::COUNT;
                player
            })
            .collect()
    }
}

// ------------------------------------------------------------- entry point

pub fn frame(app: &mut App) {
    if is_key_pressed(KeyCode::Escape) {
        if app.ui.rules_open {
            app.ui.rules_open = false;
        } else if app.ui.vote_open {
            app.ui.vote_open = false;
        } else if app.ui.quit_confirm {
            app.ui.quit_confirm = false;
        } else if app.ui.pause_open {
            app.ui.pause_open = false;
        } else if let (Screen::Play, Some(game)) = (app.screen, app.game.as_ref()) {
            if game.phase != Phase::Over {
                app.ui.pause_open = true;
            }
        }
    }
    match app.screen {
        Screen::Menu => draw_menu(app),
        Screen::Setup => draw_setup(app),
        Screen::Play => draw_play(app),
    }
}

// ------------------------------------------------------------ setup screen

fn draw_setup(app: &mut App) {
    felt_background();
    let sw = screen_width();
    let sh = screen_height();
    let w = 830.0_f32.min(sw - 32.0);
    let h = (374.0 + app.setup.count as f32 * 54.0).min(sh - 32.0);
    let x = (sw - w) * 0.5;
    let y = (sh - h) * 0.5;
    // While the token picker is open it owns every click.
    let picker_open = app.setup.token_slot.is_some() || app.ui.load_open;

    round_card(x, y, w, h, 16.0, PANEL_BG, CARD_EDGE);
    serif_center("Crazyopoly", x + w * 0.5, y + 46.0, 50.0, GOLD_SOFT);
    centered(
        i18n::t(
            "The classic property trading game - 1 to 8 players, bots fill empty seats",
            "Le jeu de société classique - 1 à 8 joueurs, des bots complètent la table",
        ),
        x + w * 0.5,
        y + 78.0,
        13.5,
        TEXT_DIM,
    );
    if !picker_open && button(Rect::new(x + 18.0, y + 14.0, 76.0, 30.0), i18n::t("MENU", "MENU"), true, false) {
        app.screen = Screen::Menu;
    }
    if !picker_open && button(Rect::new(x + w - 70.0, y + 14.0, 54.0, 30.0), i18n::code(), true, false) {
        i18n::toggle();
        app.setup.relabel_defaults();
    }
    if !picker_open
        && sound_button(
            Rect::new(x + w - 114.0, y + 14.0, 34.0, 30.0),
            Icon::Music,
            crate::audio::music_on(),
        )
    {
        crate::audio::toggle_music();
    }
    if !picker_open
        && sound_button(
            Rect::new(x + w - 156.0, y + 14.0, 34.0, 30.0),
            Icon::Sfx,
            crate::audio::sfx_on(),
        )
    {
        crate::audio::toggle_sfx();
    }

    // player count
    let cy = y + 144.0;
    txt_bold(i18n::t("PLAYERS", "JOUEURS"), x + 32.0, cy + 5.0, 13.0, MUTED);
    let mut count = app.setup.count;
    if !picker_open && button(Rect::new(x + w - 226.0, cy - 16.0, 38.0, 34.0), "-", count > 2, false) {
        count -= 1;
    }
    centered_bold(&count.to_string(), x + w - 166.0, cy, 24.0, TEXT);
    if !picker_open && button(Rect::new(x + w - 130.0, cy - 16.0, 38.0, 34.0), "+", count < 8, false) {
        count += 1;
    }
    app.setup.count = count.clamp(2, 8);
    if let Some(ed) = app.setup.editing {
        if ed >= app.setup.count {
            app.setup.editing = None;
        }
    }

    let mut ry = y + 180.0;
    for i in 0..app.setup.count {
        let row = Rect::new(x + 28.0, ry, w - 56.0, 48.0);
        let editing = app.setup.editing == Some(i);
        round_card(
            row.x,
            row.y,
            row.w,
            row.h,
            10.0,
            if editing { Color { r: 0.117, g: 0.278, b: 0.212, a: 1.0 } } else { ROW_BG },
            if editing { GOLD } else { CARD_EDGE },
        );
        let token_rect = Rect::new(row.x + 8.0, row.y + 2.0, 44.0, row.h - 4.0);
        let (tmx, tmy) = mouse_position();
        if inr(token_rect, tmx, tmy) {
            round_rect(
                token_rect.x,
                token_rect.y,
                token_rect.w,
                token_rect.h,
                8.0,
                BTN_HOVER,
            );
        }
        tokens::draw(
            app.setup.slots[i].token,
            TOKENS[i % TOKENS.len()],
            row.x + 30.0,
            row.y + row.h * 0.5,
            17.0,
        );
        if !picker_open && inr(token_rect, tmx, tmy) && is_mouse_button_pressed(MouseButton::Left) {
            app.setup.token_slot = Some(i);
        }

        let name_rect = Rect::new(row.x + 62.0, row.y + 8.0, row.w - 300.0, row.h - 16.0);
        round_rect(
            name_rect.x,
            name_rect.y,
            name_rect.w,
            name_rect.h,
            8.0,
            BTN_OFF,
        );
        if editing {
            round_outline(
                name_rect.x,
                name_rect.y,
                name_rect.w,
                name_rect.h,
                8.0,
                1.5,
                GOLD,
            );
        } else if inr(name_rect, mouse_position().0, mouse_position().1) {
            round_outline(
                name_rect.x,
                name_rect.y,
                name_rect.w,
                name_rect.h,
                8.0,
                1.0,
                GOLD_DIM,
            );
        }
        // A pencil at the far end of the field: this text can be changed.
        let pen_x = name_rect.x + name_rect.w - 18.0;
        let pen_y = name_rect.y + name_rect.h * 0.5;
        draw_line(pen_x - 4.0, pen_y + 4.0, pen_x + 4.0, pen_y - 4.0, 1.6, GOLD_DIM);
        draw_line(pen_x - 6.0, pen_y + 6.0, pen_x - 4.0, pen_y + 4.0, 1.6, GOLD_DIM);
        let shown = truncate(&app.setup.slots[i].name, name_rect.w - 16.0, 16.0);
        txt(&shown, name_rect.x + 9.0, name_rect.y + name_rect.h * 0.68, 16.0, TEXT);
        if editing && (get_time() * 2.0) as i64 % 2 == 0 {
            let caret = name_rect.x + 10.0 + text_w(&shown, 16.0);
            draw_line(caret, name_rect.y + 8.0, caret, name_rect.y + name_rect.h - 8.0, 1.5, TEXT);
        }
        if !picker_open
            && inr(name_rect, mouse_position().0, mouse_position().1)
            && is_mouse_button_pressed(MouseButton::Left)
        {
            app.setup.editing = Some(i);
        }

        let seg = 112.0;
        let human_rect = Rect::new(row.x + row.w - 238.0, row.y + 8.0, seg, row.h - 16.0);
        let bot_rect = Rect::new(row.x + row.w - 120.0, row.y + 8.0, seg, row.h - 16.0);
        let human_on = !app.setup.slots[i].ai;
        let (mx, my) = mouse_position();
        round_rect(
            human_rect.x,
            human_rect.y,
            human_rect.w,
            human_rect.h,
            8.0,
            if human_on {
                GOLD
            } else if inr(human_rect, mx, my) {
                BTN_HOVER
            } else {
                BTN
            },
        );
        round_rect(
            bot_rect.x,
            bot_rect.y,
            bot_rect.w,
            bot_rect.h,
            8.0,
            if !human_on {
                GOLD
            } else if inr(bot_rect, mx, my) {
                BTN_HOVER
            } else {
                BTN
            },
        );
        centered_bold(
            i18n::t("HUMAN", "HUMAIN"),
            human_rect.x + seg * 0.5,
            human_rect.y + human_rect.h * 0.5,
            12.0,
            if human_on { BG_DEEP } else { CREAM },
        );
        centered_bold(
            "BOT",
            bot_rect.x + seg * 0.5,
            bot_rect.y + bot_rect.h * 0.5,
            12.0,
            if !human_on { BG_DEEP } else { CREAM },
        );
        if !picker_open && inr(human_rect, mx, my) && is_mouse_button_pressed(MouseButton::Left) {
            app.setup.slots[i].ai = false;
        }
        if !picker_open && inr(bot_rect, mx, my) && is_mouse_button_pressed(MouseButton::Left) {
            app.setup.slots[i].ai = true;
        }
        ry += 54.0;
    }

    if let Some(ed) = app.setup.editing.filter(|_| !picker_open) {
        if ed < app.setup.count {
            let mut name = app.setup.slots[ed].name.clone();
            if is_key_pressed(KeyCode::Backspace) {
                name.pop();
            }
            while let Some(ch) = macroquad::input::get_char_pressed() {
                let ok = ch.is_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' || ch == '.';
                if ok && name.chars().count() < 16 {
                    name.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Enter) {
                app.setup.editing = None;
            }
            app.setup.slots[ed].name = name;
        }
    }

    let humans = (0..app.setup.count).filter(|i| !app.setup.slots[*i].ai).count();
    let bots = app.setup.count - humans;
    let summary = if humans == 0 {
        i18n::t(
            "Bots only: the game plays itself.",
            "Bots uniquement : la partie se joue seule.",
        )
        .to_string()
    } else {
        crate::tr!(
            "{} human(s) + {} bot(s)",
            "{} humain(s) + {} bot(s)",
            humans,
            bots
        )
    };
    centered(&summary, x + w * 0.5, y + h - 132.0, 12.5, MUTED);
    if !app.setup.error.is_empty() {
        centered(&app.setup.error, x + w * 0.5, y + h - 150.0, 12.5, WARN);
    }

    let btn_w = ((w - 140.0) * 0.5).min(250.0);
    let start_y = y + h - 112.0;
    if !picker_open
        && button(
            Rect::new(x + w * 0.5 - btn_w - 6.0, start_y, btn_w, 50.0),
            i18n::t("START GAME", "LANCER LA PARTIE"),
            true,
            true,
        )
    {
        app.setup.error.clear();
        app.game = Some(Game::new(app.setup.build_players()));
        app.ui = Ui::new();
        app.screen = Screen::Play;
    }
    let has_save = crate::persist::has_saves();
    if !picker_open
        && button(
            Rect::new(x + w * 0.5 + 6.0, start_y, btn_w, 50.0),
            i18n::t("CONTINUE", "REPRENDRE"),
            has_save,
            false,
        )
    {
        if let Some(game) = open_saves(&mut app.ui) {
            app.setup.error.clear();
            app.game = Some(game);
            app.ui = Ui::new();
            app.screen = Screen::Play;
        } else if !app.ui.load_open {
            app.setup.error = i18n::t("Load failed", "Échec du chargement").to_string();
        }
    }
    let saves = crate::persist::list();
    let note = if saves.is_empty() {
        i18n::t("Save folder: ", "Dossier des sauvegardes : ").to_string()
            + &crate::persist::saves_dir().display().to_string()
    } else {
        crate::tr!(
            "{} save(s) in {}",
            "{} sauvegarde(s) dans {}",
            saves.len(),
            crate::persist::saves_dir().display()
        )
    };
    centered(
        &truncate(&note, w - 40.0, 10.0),
        x + w * 0.5,
        y + h - 44.0,
        10.0,
        MUTED,
    );
    if app.ui.load_open {
        draw_load(app);
    }
    if let Some(slot) = app.setup.token_slot {
        // Ignore the click that opened the picker: it belongs to the setup row.
        draw_token_picker(app, slot, picker_open);
    }
}

fn draw_token_picker(app: &mut App, slot: usize, interactive: bool) {
    let r = modal_rect(720.0, 430.0);
    let content = modal_shell(r, i18n::t("CHOOSE YOUR TOKEN", "CHOISISSEZ VOTRE JETON"));
    let cols = 4.0;
    let gap = 12.0;
    let card_w = (content.w - gap * (cols - 1.0)) / cols;
    let card_h = (content.h - 62.0 - gap) * 0.5;
    let mut picked = None;
    for k in 0..tokens::COUNT {
        let col = k % cols as usize;
        let row = k / cols as usize;
        let cr = Rect::new(
            content.x + col as f32 * (card_w + gap),
            content.y + row as f32 * (card_h + gap),
            card_w,
            card_h,
        );
        let selected = app.setup.slots[slot].token == k;
        let (mx, my) = mouse_position();
        let hover = inr(cr, mx, my);
        draw_rectangle(
            cr.x,
            cr.y,
            cr.w,
            cr.h,
            if selected {
                SELECT
            } else if hover {
                BTN_HOVER
            } else {
                ROW_BG
            },
        );
        draw_rectangle_lines(
            cr.x,
            cr.y,
            cr.w,
            cr.h,
            if selected { 2.0 } else { 1.0 },
            if selected { ACCENT } else { PANEL_EDGE },
        );
        tokens::draw(k, TOKENS[slot % TOKENS.len()], cr.x + cr.w * 0.5, cr.y + card_h * 0.36, 30.0);
        centered_bold(
            &truncate(tokens::name(k), cr.w - 12.0, 13.0),
            cr.x + cr.w * 0.5,
            cr.y + card_h * 0.76,
            13.0,
            TEXT,
        );
        if interactive && hover && is_mouse_button_pressed(MouseButton::Left) {
            picked = Some(k);
        }
    }
    let footer = r.y + r.h - 58.0;
    if interactive
        && button(Rect::new(r.x + 24.0, footer, 190.0, 40.0), i18n::t("CLOSE", "FERMER"), true, false)
    {
        app.setup.token_slot = None;
    }
    centered(
        i18n::t("Click a token to use it", "Cliquez sur un jeton pour l'utiliser"),
        r.x + r.w * 0.5 + 100.0,
        footer + 20.0,
        12.0,
        MUTED,
    );
    if let Some(k) = picked {
        app.setup.slots[slot].token = k;
        app.setup.token_slot = None;
    }
}

// -------------------------------------------------------------- play screen

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Bottom,
    Left,
    Top,
    Right,
    Corner,
}

struct Geo {
    x: f32,
    y: f32,
    s: f32,
    t: f32,
    c: f32,
}

impl Geo {
    fn new(x: f32, y: f32, s: f32) -> Self {
        let t = s / 11.6;
        Geo { x, y, s, t, c: 1.3 * t }
    }

    fn rect(&self, i: usize) -> Rect {
        let (x, y, s, t, c) = (self.x, self.y, self.s, self.t, self.c);
        match i {
            0 => Rect::new(x + s - c, y + s - c, c, c),
            1..=9 => Rect::new(x + s - c - i as f32 * t, y + s - t, t, t),
            10 => Rect::new(x, y + s - c, c, c),
            11..=19 => {
                let j = (i - 10) as f32;
                Rect::new(x, y + s - c - j * t, t, t)
            }
            20 => Rect::new(x, y, c, c),
            21..=29 => {
                let j = (i - 20) as f32 - 1.0;
                Rect::new(x + c + j * t, y, t, t)
            }
            30 => Rect::new(x + s - c, y, c, c),
            _ => {
                let j = (i - 30) as f32 - 1.0;
                Rect::new(x + s - t, y + c + j * t, t, t)
            }
        }
    }

    fn center(&self) -> Rect {
        Rect::new(self.x + self.c, self.y + self.c, self.s - 2.0 * self.c, self.s - 2.0 * self.c)
    }
}

/// Three columns on the felt table: the players on the left, the board in the
/// middle, the turn and its actions on the right.
struct Layout {
    geo: Geo,
    left: Rect,
    right: Rect,
}

fn layout() -> Layout {
    let sw = screen_width();
    let sh = screen_height();
    let m = 20.0;
    let side_want = (sw * 0.175).clamp(210.0, 300.0);
    let board_s = (sh - 2.0 * m)
        .min(sw - 2.0 * side_want - 4.0 * m)
        .max(320.0);
    let side = ((sw - board_s - 4.0 * m) * 0.5).max(170.0);
    let x0 = m;
    let y0 = (sh - board_s) * 0.5;
    let board_x = x0 + side + m;
    Layout {
        geo: Geo::new(board_x, y0, board_s),
        left: Rect::new(x0, y0, side, board_s),
        right: Rect::new(board_x + board_s + m, y0, side, board_s),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Modal {
    None,
    /// Dice are still tumbling: let the roll finish before any dialog opens.
    Waiting,
    Assets,
    Trade,
    Auction,
    Debt,
    Over,
    Rules,
    Vote,
    Quit,
    Pause,
}

fn buying(g: &Game) -> bool {
    g.phase == Phase::AwaitBuy && !g.players[g.turn].ai && g.dice_anim <= 0.0
}

fn active_modal(g: &Game, ui: &Ui) -> Modal {
    if g.phase == Phase::Over {
        return Modal::Over;
    }
    // The book and the vote can be opened from the pause menu, so they sit
    // above it; the quit question too, or leaving would go unseen.
    if ui.rules_open {
        return Modal::Rules;
    }
    if ui.vote_open {
        return Modal::Vote;
    }
    if ui.quit_confirm {
        return Modal::Quit;
    }
    // The pause menu opens even mid-roll: the table is frozen, dice and all.
    if ui.pause_open {
        return Modal::Pause;
    }
    if g.dice_anim > 0.0 {
        return Modal::Waiting;
    }
    if ui.asset_open {
        return Modal::Assets;
    }
    if g.trade.is_some() {
        return Modal::Trade;
    }
    if g.phase == Phase::Auction {
        return Modal::Auction;
    }
    if g.phase == Phase::Debt && !g.players[g.turn].ai {
        return Modal::Debt;
    }
    Modal::None
}

fn draw_play(app: &mut App) {
    let App { screen, game, ui, .. } = app;
    let Some(g) = game.as_mut() else {
        return;
    };
    let lay = layout();
    let (mx, my) = mouse_position();
    let modal = active_modal(g, ui);
    let input = modal == Modal::None;

    felt_background();
    // A tile under the cursor shows its own deed; the card of the space on
    // offer would only cover it.
    let over_tile = input && (0..40).any(|i| inr(lay.geo.rect(i), mx, my));
    draw_board(g, &lay, mx, my, input);
    draw_left_column(g, ui, &lay);
    draw_right_column(g, ui, &lay, input);
    if buying(g) && !over_tile {
        draw_deed_card(g, &lay);
    }

    let mut again = false;
    match modal {
        Modal::None | Modal::Waiting => {}
        Modal::Assets => draw_assets(g, ui, &lay),
        Modal::Trade => draw_trade(g, ui),
        Modal::Auction => draw_auction(g),
        Modal::Debt => draw_debt(g, ui),
        Modal::Over => again = draw_over(g),
        Modal::Rules => draw_rules(ui),
        Modal::Vote => draw_vote(g, ui),
        Modal::Pause => draw_pause(g, ui),
        Modal::Quit => {
            if draw_quit(ui) {
                *game = None;
                *ui = Ui::new();
                *screen = Screen::Menu;
                return;
            }
        }
    }
    if again {
        *game = None;
        *screen = Screen::Setup;
    }
}

// -------------------------------------------------------------------- board

fn side_of(i: usize) -> Side {
    match i {
        0 | 10 | 20 | 30 => Side::Corner,
        1..=9 => Side::Bottom,
        11..=19 => Side::Left,
        21..=29 => Side::Top,
        _ => Side::Right,
    }
}

fn band_rect(r: &Rect, side: Side) -> Rect {
    match side {
        Side::Bottom => Rect::new(r.x, r.y, r.w, r.h * 0.24),
        Side::Top => Rect::new(r.x, r.y + r.h - r.h * 0.24, r.w, r.h * 0.24),
        Side::Left => Rect::new(r.x + r.w - r.w * 0.22, r.y, r.w * 0.22, r.h),
        Side::Right => Rect::new(r.x, r.y, r.w * 0.22, r.h),
        Side::Corner => *r,
    }
}

fn text_rect(r: &Rect, side: Side) -> Rect {
    match side {
        Side::Bottom => Rect::new(r.x + 2.0, r.y + r.h * 0.24 + 1.0, r.w - 4.0, r.h - r.h * 0.24 - 13.0),
        Side::Top => Rect::new(r.x + 2.0, r.y + 13.0, r.w - 4.0, r.h - r.h * 0.24 - 14.0),
        Side::Left => Rect::new(r.x + 2.0, r.y + 2.0, r.w * 0.78 - 3.0, r.h - 15.0),
        Side::Right => Rect::new(r.x + r.w * 0.22 + 1.0, r.y + 2.0, r.w * 0.78 - 3.0, r.h - 15.0),
        Side::Corner => *r,
    }
}

fn draw_board(g: &Game, lay: &Layout, mx: f32, my: f32, input: bool) {
    let geo = &lay.geo;
    // The board is set in a thick wooden frame on the felt.
    round_rect(
        geo.x - 16.0,
        geo.y - 14.0,
        geo.s + 32.0,
        geo.s + 32.0,
        10.0,
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.22 },
    );
    round_rect(geo.x - 14.0, geo.y - 14.0, geo.s + 28.0, geo.s + 28.0, 8.0, WOOD_EDGE);
    round_rect(geo.x - 11.0, geo.y - 11.0, geo.s + 22.0, geo.s + 22.0, 6.0, WOOD);
    draw_rectangle(geo.x, geo.y, geo.s, geo.s, BOARD_BG);

    let mut hovered = None;
    for i in 0..40 {
        let r = geo.rect(i);
        let hover = input && inr(r, mx, my);
        if hover {
            hovered = Some(i);
        }
        draw_tile(g, geo, i, hover);
    }
    draw_center(g, geo);
    draw_tokens(g, geo);
    if let (Some(tile), true) = (hovered, input) {
        draw_tooltip(g, geo, tile, mx, my);
    }
}

fn draw_buildings(band: Rect, side: Side, count: u8) {
    if count == 0 {
        return;
    }
    let vertical = matches!(side, Side::Left | Side::Right);
    if count >= MAX_HOUSES {
        if vertical {
            draw_rectangle(band.x + band.w * 0.15, band.y + band.h * 0.18, band.w * 0.70, band.h * 0.64, HOTEL_COL);
        } else {
            draw_rectangle(band.x + band.w * 0.27, band.y + band.h * 0.16, band.w * 0.46, band.h * 0.68, HOTEL_COL);
        }
        return;
    }
    let n = count as f32;
    for k in 0..count {
        let t = (k as f32 + 0.5) / n;
        if vertical {
            let y = band.y + band.h * (0.10 + 0.80 * t) - band.h * 0.055;
            draw_rectangle(band.x + band.w * 0.20, y, band.w * 0.60, band.h * 0.11, HOUSE_COL);
        } else {
            let x = band.x + band.w * (0.14 + 0.72 * t) - band.w * 0.045;
            draw_rectangle(x, band.y + band.h * 0.26, band.w * 0.09, band.h * 0.48, HOUSE_COL);
        }
    }
}

/// Where the pictogram of a side tile sits: in the band strip, clear of the name.
fn icon_center(r: &Rect, side: Side) -> (f32, f32) {
    match side {
        Side::Bottom => (r.x + r.w * 0.5, r.y + r.h * 0.125),
        Side::Top => (r.x + r.w * 0.5, r.y + r.h * 0.875),
        Side::Left => (r.x + r.w * 0.87, r.y + r.h * 0.5),
        Side::Right => (r.x + r.w * 0.13, r.y + r.h * 0.5),
        Side::Corner => (r.x + r.w * 0.5, r.y + r.h * 0.5),
    }
}

fn icon_size(r: &Rect) -> f32 {
    // Kept inside the colour band so the name below it stays clear.
    (r.w * 0.26).min(20.0)
}

const PLATE: Color = Color { r: 0.114, g: 0.157, b: 0.145, a: 1.0 };
const CELL: Color = Color { r: 0.192, g: 0.243, b: 0.224, a: 1.0 };
const LEAF: Color = Color { r: 0.180, g: 0.451, b: 0.259, a: 1.0 };
const STEEL: Color = Color { r: 0.337, g: 0.502, b: 0.620, a: 1.0 };
const WATER: Color = Color { r: 0.196, g: 0.451, b: 0.741, a: 1.0 };
const BOLT: Color = Color { r: 0.949, g: 0.769, b: 0.145, a: 1.0 };
const CHEST_BODY: Color = Color { r: 0.157, g: 0.310, b: 0.573, a: 1.0 };
const CHEST_LID: Color = Color { r: 0.243, g: 0.416, b: 0.686, a: 1.0 };

/// A little locomotive on its rails: the four stations.
fn draw_rail_icon(r: &Rect, side: Side) {
    let (cx, cy) = icon_center(r, side);
    let s = icon_size(r);
    // Track first, so the engine sits on it.
    let ry = cy + s * 0.36;
    draw_line(cx - s * 0.48, ry, cx + s * 0.48, ry, 1.5, INK_SOFT);
    for k in 0..5 {
        let x = cx - s * 0.38 + k as f32 * s * 0.19;
        draw_line(x, ry - s * 0.04, x, ry + s * 0.09, 1.2, INK_SOFT);
    }
    // The engine, in one dark silhouette: boiler, cab, chimney.
    draw_rectangle(cx - s * 0.38, cy - s * 0.04, s * 0.56, s * 0.26, INK);
    draw_rectangle(cx + s * 0.08, cy - s * 0.28, s * 0.30, s * 0.50, INK);
    draw_rectangle(cx - s * 0.34, cy - s * 0.30, s * 0.14, s * 0.26, INK);
    draw_circle(cx - s * 0.06, cy - s * 0.06, s * 0.10, INK);
    // Its window, then the wheels and a puff of smoke on the cream tile.
    draw_rectangle(cx + s * 0.15, cy - s * 0.18, s * 0.16, s * 0.16, TILE);
    for k in 0..3 {
        let wx = cx - s * 0.26 + k as f32 * s * 0.24;
        draw_circle(wx, cy + s * 0.28, s * 0.088, GOLD);
    }
    draw_circle(cx - s * 0.30, cy - s * 0.42, s * 0.055, Color { a: 0.45, ..INK });
}

/// Water Works: a tap with a drop falling out of it.
fn draw_water_icon(r: &Rect, side: Side) {
    let (cx, cy) = icon_center(r, side);
    let s = icon_size(r);
    // The pipe runs in from the left, then bends down into a spout.
    draw_rectangle(cx - s * 0.50, cy - s * 0.34, s * 0.40, s * 0.22, STEEL);
    draw_rectangle(cx - s * 0.28, cy - s * 0.34, s * 0.26, s * 0.58, STEEL);
    draw_rectangle(cx - s * 0.32, cy + s * 0.14, s * 0.34, s * 0.10, STEEL);
    // The valve on top of the pipe.
    draw_rectangle(cx - s * 0.40, cy - s * 0.48, s * 0.26, s * 0.10, GOLD);
    draw_rectangle(cx - s * 0.28, cy - s * 0.48, s * 0.08, s * 0.18, GOLD);
    // The drop.
    draw_circle(cx - s * 0.15, cy + s * 0.34, s * 0.115, WATER);
    draw_triangle(
        vec2(cx - s * 0.265, cy + s * 0.30),
        vec2(cx - s * 0.035, cy + s * 0.30),
        vec2(cx - s * 0.15, cy + s * 0.10),
        WATER,
    );
}

/// Electric Company: a lightning bolt.
fn draw_power_icon(r: &Rect, side: Side) {
    let (cx, cy) = icon_center(r, side);
    let s = icon_size(r);
    draw_triangle(
        vec2(cx + s * 0.02, cy - s * 0.48),
        vec2(cx + s * 0.40, cy - s * 0.48),
        vec2(cx - s * 0.16, cy + s * 0.10),
        BOLT,
    );
    draw_triangle(
        vec2(cx - s * 0.10, cy + s * 0.02),
        vec2(cx + s * 0.24, cy + s * 0.02),
        vec2(cx - s * 0.02, cy + s * 0.48),
        BOLT,
    );
}

/// Chance: the little card with its question mark.
fn draw_chance_icon(r: &Rect, side: Side) {
    let (cx, cy) = icon_center(r, side);
    let s = icon_size(r) * 0.98;
    let w = s * 0.76;
    round_rect(cx - w * 0.5 + 1.3, cy - s * 0.5 + 1.3, w, s, 1.5, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.13 });
    round_rect(cx - w * 0.5, cy - s * 0.5, w, s, 1.5, TILE);
    round_outline(cx - w * 0.5, cy - s * 0.5, w, s, 1.5, 1.4, WINE);
    centered_bold("?", cx, cy, s * 0.68, WINE);
}

/// Community Chest: a banded treasure chest.
fn draw_chest_icon(r: &Rect, side: Side) {
    let (cx, cy) = icon_center(r, side);
    let s = icon_size(r) * 0.96;
    let w = s * 1.02;
    let top = cy - s * 0.44;
    draw_rectangle(cx - w * 0.46, top + s * 0.34, w * 0.92, s * 0.52, CHEST_BODY);
    round_rect(cx - w * 0.46, top, w * 0.92, s * 0.36, 2.0, CHEST_LID);
    draw_line(cx - w * 0.46, top + s * 0.34, cx + w * 0.46, top + s * 0.34, 1.3, PLATE);
    for k in 0..2 {
        let bx = cx - w * 0.23 + k as f32 * w * 0.46;
        draw_rectangle(bx - w * 0.045, top, w * 0.09, s * 0.86, GOLD);
    }
    draw_rectangle(cx - w * 0.10, top + s * 0.26, w * 0.20, s * 0.20, GOLD);
    draw_circle(cx, top + s * 0.34, w * 0.032, PLATE);
}

fn draw_corner(space: &Space, r: Rect) {
    let cx = r.x + r.w * 0.5;
    match space.kind {
        Kind::Go => {
            centered_bold(i18n::t("GO", "DÉPART"), cx, r.y + r.h * 0.22, r.w * 0.22, INK);
            centered(i18n::t("COLLECT", "ENCAISSEZ"), cx, r.y + r.h * 0.40, r.w * 0.090, INK_SOFT);
            centered_bold("200$", cx, r.y + r.h * 0.49, r.w * 0.120, INK_SOFT);
            // A little red car, with its windows, its hubcaps and a trail of dust.
            let cy = r.y + r.h * 0.74;
            let bx = r.x + r.w * 0.24;
            let bw = r.w * 0.56;
            for k in 0..3 {
                let tx = bx - r.w * 0.04 - k as f32 * r.w * 0.05;
                draw_line(tx, cy + r.h * 0.012, tx - r.w * 0.03, cy + r.h * 0.012, 1.2, INK_LINE);
            }
            round_rect(bx, cy - r.h * 0.030, bw, r.h * 0.062, r.h * 0.018, WINE);
            round_rect(bx + bw * 0.26, cy - r.h * 0.082, bw * 0.46, r.h * 0.058, r.h * 0.014, WINE);
            draw_rectangle(bx + bw * 0.32, cy - r.h * 0.076, bw * 0.15, r.h * 0.036, CREAM);
            draw_rectangle(bx + bw * 0.50, cy - r.h * 0.076, bw * 0.15, r.h * 0.036, CREAM);
            draw_circle(bx + bw * 0.26, cy + r.h * 0.034, r.h * 0.030, INK);
            draw_circle(bx + bw * 0.74, cy + r.h * 0.034, r.h * 0.030, INK);
            draw_circle(bx + bw * 0.26, cy + r.h * 0.034, r.h * 0.012, CREAM_DIM);
            draw_circle(bx + bw * 0.74, cy + r.h * 0.034, r.h * 0.012, CREAM_DIM);
            draw_circle(bx + bw - 2.0, cy - r.h * 0.002, r.h * 0.011, GOLD);
        }
        Kind::Jail => {
            centered_bold(i18n::t("JAIL", "PRISON"), cx, r.y + r.h * 0.20, r.w * 0.20, INK);
            // A barred window onto a dark cell, set in a heavy frame.
            let bw = r.w * 0.56;
            let bh = r.h * 0.21;
            let bx = cx - bw * 0.5;
            let by = r.y + r.h * 0.29;
            round_rect(bx - 2.5, by - 2.5, bw + 5.0, bh + 5.0, 2.0, INK);
            draw_rectangle(bx, by, bw, bh, CELL);
            for k in 1..5 {
                let lx = bx + bw * k as f32 / 5.0;
                draw_line(lx, by, lx, by + bh, 1.8, CREAM_DIM);
            }
            draw_line(bx, by + bh * 0.5, bx + bw, by + bh * 0.5, 1.4, CREAM_DIM);
            centered(i18n::t("JUST", "SIMPLE"), cx, r.y + r.h * 0.66, r.w * 0.085, INK_SOFT);
            centered(i18n::t("VISITING", "VISITE"), cx, r.y + r.h * 0.76, r.w * 0.085, INK_SOFT);
        }
        Kind::FreeParking => {
            centered_bold(i18n::t("FREE", "PARC"), cx, r.y + r.h * 0.20, r.w * 0.155, INK);
            centered_bold(i18n::t("PARKING", "GRATUIT"), cx, r.y + r.h * 0.32, r.w * 0.125, INK);
            // A sunny little park: a tree, a bench and the sun above them.
            let ground = r.y + r.h * 0.72;
            let sun = (r.x + r.w * 0.76, r.y + r.h * 0.56);
            draw_circle(sun.0, sun.1, r.w * 0.042, GOLD);
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                let (dx, dy) = (a.cos(), a.sin());
                draw_line(
                    sun.0 + dx * r.w * 0.062,
                    sun.1 + dy * r.w * 0.062,
                    sun.0 + dx * r.w * 0.085,
                    sun.1 + dy * r.w * 0.085,
                    1.3,
                    GOLD_DIM,
                );
            }
            draw_rectangle(cx - r.w * 0.020, ground - r.h * 0.055, r.w * 0.040, r.h * 0.115, WOOD);
            draw_circle(cx, ground - r.h * 0.105, r.w * 0.088, LEAF);
            draw_circle(cx - r.w * 0.058, ground - r.h * 0.062, r.w * 0.062, LEAF);
            draw_circle(cx + r.w * 0.058, ground - r.h * 0.062, r.w * 0.062, LEAF);
            let bx = r.x + r.w * 0.14;
            let bw = r.w * 0.26;
            draw_rectangle(bx, ground + r.h * 0.020, bw, r.h * 0.022, WOOD);
            draw_line(bx + 2.0, ground + r.h * 0.042, bx + 2.0, ground + r.h * 0.078, 1.4, WOOD);
            draw_line(bx + bw - 2.0, ground + r.h * 0.042, bx + bw - 2.0, ground + r.h * 0.078, 1.4, WOOD);
            draw_line(r.x + r.w * 0.10, ground + r.h * 0.100, r.x + r.w * 0.90, ground + r.h * 0.100, 1.2, INK_LINE);
        }
        _ => {
            centered_bold(i18n::t("GO TO", "ALLEZ EN"), cx, r.y + r.h * 0.22, r.w * 0.125, INK);
            centered_bold(i18n::t("JAIL", "PRISON"), cx, r.y + r.h * 0.35, r.w * 0.165, INK);
            // An arrow driving straight into a barred door.
            let bh = r.h * 0.22;
            let by = r.y + r.h * 0.52;
            let bw = r.w * 0.28;
            let bx = r.x + r.w * 0.62;
            round_rect(bx - 2.0, by - 2.0, bw + 4.0, bh + 4.0, 1.5, INK);
            draw_rectangle(bx, by, bw, bh, CELL);
            for k in 1..4 {
                let lx = bx + bw * k as f32 / 4.0;
                draw_line(lx, by, lx, by + bh, 1.6, CREAM_DIM);
            }
            let ay = by + bh * 0.5;
            draw_line(r.x + r.w * 0.12, ay, bx - 1.0, ay, 2.4, INK);
            draw_triangle(vec2(bx + 1.5, ay), vec2(bx - 6.5, ay - 5.5), vec2(bx - 6.5, ay + 5.5), INK);
        }
    }
}

fn owner_dot(r: &Rect, side: Side) -> (f32, f32) {
    match side {
        Side::Bottom => (r.x + r.w - 9.0, r.y + r.h - 9.0),
        Side::Top => (r.x + r.w - 9.0, r.y + 9.0),
        Side::Left => (r.x + 9.0, r.y + r.h - 9.0),
        Side::Right => (r.x + 9.0, r.y + 9.0),
        Side::Corner => (r.x + 9.0, r.y + 9.0),
    }
}

fn draw_tile(g: &Game, geo: &Geo, i: usize, hover: bool) {
    let r = geo.rect(i);
    let space = &g.spaces[i];
    let side = side_of(i);
    draw_rectangle(r.x, r.y, r.w, r.h, TILE);
    if hover {
        draw_rectangle(r.x, r.y, r.w, r.h, HOVER);
    }

    if space.corner() {
        draw_corner(space, r);
    } else {
        if space.has_band() {
            let band = band_rect(&r, side);
            draw_rectangle(band.x, band.y, band.w, band.h, space.group.color());
            draw_buildings(band, side, g.props[i].houses);
        } else if space.kind == Kind::Railroad {
            draw_rail_icon(&r, side);
        } else if space.kind == Kind::Utility {
            if space.name.starts_with("Water") {
                draw_water_icon(&r, side);
            } else {
                draw_power_icon(&r, side);
            }
        } else if space.kind == Kind::Chance {
            draw_chance_icon(&r, side);
        } else if space.kind == Kind::Chest {
            draw_chest_icon(&r, side);
        }

        let tr = text_rect(&r, side);
        let (lines, size) = fit_wrap(space.name, tr.w, 3, r.w * 0.145);
        let lh = size * 1.10;
        let total = lines.len() as f32 * lh;
        let start = tr.y + (tr.h - total) * 0.5;
        for (k, line) in lines.iter().enumerate() {
            centered(line, tr.x + tr.w * 0.5, start + lh * (k as f32 + 0.5), size, INK);
        }

        let price = match space.kind {
            Kind::Street | Kind::Railroad | Kind::Utility => Some(space.price),
            Kind::Tax => Some(space.tax),
            _ => None,
        };
        if let Some(value) = price {
            let py = if side == Side::Top {
                r.y + r.h - r.h * 0.24 - 6.0
            } else {
                r.y + r.h - 6.0
            };
            centered_bold(&format!("{}$", value), r.x + r.w * 0.5, py - 3.0, r.w * 0.115, INK_SOFT);
        }
    }

    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, INK_LINE);

    if let Some(owner) = g.props[i].owner {
        let color = g.players[owner].color;
        draw_rectangle(r.x, r.y, r.w, r.h, Color { r: color.r, g: color.g, b: color.b, a: 0.10 });
        let (dx, dy) = owner_dot(&r, side);
        draw_circle(dx, dy, 4.8, color);
        draw_circle_lines(dx, dy, 4.8, 1.2, TILE);
        if g.props[i].mortgaged {
            draw_rectangle(r.x, r.y, r.w, r.h, Color { r: 0.10, g: 0.10, b: 0.10, a: 0.40 });
            centered_bold("M", dx, dy, 8.0, WHITE);
        }
    }
}

fn draw_tokens(g: &Game, geo: &Geo) {
    const OFFSETS: [(f32, f32); 8] = [
        (-0.26, -0.26),
        (0.26, -0.26),
        (-0.26, 0.24),
        (0.26, 0.24),
        (0.0, -0.34),
        (0.0, 0.34),
        (-0.46, 0.0),
        (0.46, 0.0),
    ];
    for (i, player) in g.players.iter().enumerate() {
        if player.bankrupt {
            continue;
        }
        let r = geo.rect(player.pos);
        let (ox, oy) = OFFSETS[i % OFFSETS.len()];
        let cx = r.x + r.w * 0.5 + ox * r.w;
        let mut cy = r.y + r.h * 0.5 + oy * r.h;
        if let Some(anim) = &g.anim {
            if anim.player == i {
                cy -= (anim.t * std::f32::consts::PI).sin() * r.h * 0.35;
            }
        }
        // Round chip with the player's token drawn in white on it.
        let radius = (r.w * 0.165).clamp(8.0, 14.0);
        draw_circle(cx + 1.0, cy + 2.0, radius, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.16 });
        draw_circle(cx, cy, radius, player.color);
        draw_circle_lines(cx, cy, radius, 1.2, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.55 });
        tokens::draw(player.token, WHITE, cx, cy, radius * 0.74);
    }
}

/// Small caps with extra letter spacing, as used for the board's subtitle.
fn spaced_center(text: &str, cx: f32, cy: f32, size: f32, spacing: f32, color: Color) {
    let mut total = 0.0;
    for ch in text.chars() {
        total += text_w(&ch.to_string(), size) + spacing;
    }
    total -= spacing;
    let mut x = cx - total * 0.5;
    for ch in text.chars() {
        let glyph = ch.to_string();
        txt(&glyph, x, cy + size * 0.34, size, color);
        x += text_w(&glyph, size) + spacing;
    }
}

fn draw_center(g: &Game, geo: &Geo) {
    let c = geo.center();
    draw_rectangle(c.x, c.y, c.w, c.h, CENTER_BG);
    let cx = c.x + c.w * 0.5;

    let title = "CRAZYOPOLY";
    let mut title_size = c.w * 0.128;
    while serif_w(title, title_size) > c.w * 0.90 && title_size > 8.0 {
        title_size -= 1.0;
    }
    serif_center(title, cx, c.y + c.h * 0.150, title_size, WINE);

    let dy = c.y + c.h * 0.455;
    let ds = (c.w * 0.105).min(c.h * 0.105);
    crate::dice::draw(g, cx, dy, ds);
    serif_center(
        &format!("{} + {} = {}", g.dice.0, g.dice.1, g.dice.0 as u16 + g.dice.1 as u16),
        cx,
        dy + ds * 0.90,
        ds * 0.42,
        INK_SOFT,
    );

    let player = &g.players[g.turn];
    let label = crate::tr!("{}'s turn", "Tour de {}", player.name);
    serif_center(&label, cx, c.y + c.h * 0.705, c.w * 0.040, WINE);
    centered(&phase_hint(g), cx, c.y + c.h * 0.752, c.w * 0.023, INK_SOFT);

    if let Some((text, _)) = &g.banner {
        let bcard_w = c.w * 0.76;
        let bcard_h = c.h * 0.22;
        let bx = c.x + (c.w - bcard_w) * 0.5;
        let by2 = c.y + c.h - bcard_h - c.h * 0.02;
        draw_rectangle(bx + 3.0, by2 + 5.0, bcard_w, bcard_h, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.16 });
        draw_rectangle(bx, by2, bcard_w, bcard_h, TILE);
        draw_rectangle_lines(bx, by2, bcard_w, bcard_h, 1.5, GOLD);
        spaced_center(
            i18n::t("CARD DRAWN", "CARTE TIRÉE"),
            bx + bcard_w * 0.5,
            by2 + bcard_h * 0.16,
            c.w * 0.020,
            c.w * 0.003,
            GOLD_DIM,
        );
        // Naming the player makes the longest cards run to a fourth line, so
        // the text drops a size rather than losing its end.
        let mut size = c.w * 0.027;
        let mut lines = wrap(text, bcard_w - 24.0, size);
        if lines.len() > 3 {
            size = c.w * 0.023;
            lines = wrap(text, bcard_w - 24.0, size);
        }
        let lh = size * 1.28;
        let start = by2 + bcard_h * 0.34;
        for (k, line) in lines.iter().take(4).enumerate() {
            centered(line, bx + bcard_w * 0.5, start + k as f32 * lh, size, INK);
        }
    }
}

// The dice themselves live in `dice.rs`.

fn phase_hint(g: &Game) -> String {
    match g.phase {
        Phase::Roll => {
            if g.players[g.turn].in_jail {
                i18n::t(
                    "In jail: roll doubles, pay the fine or use a card",
                    "En prison : double, amende ou carte de sortie",
                )
                .to_string()
            } else if g.rolled_double {
                i18n::t("Doubles! Roll again", "Double ! Relancez les dés").to_string()
            } else {
                i18n::t("Roll the dice to move", "Lancez les dés pour avancer").to_string()
            }
        }
        Phase::Moving => i18n::t("Moving...", "Déplacement...").to_string(),
        Phase::AwaitBuy => i18n::t("Unowned property", "Propriété libre").to_string(),
        Phase::Auction => match &g.auction {
            Some(a) => crate::tr!("Auction - {} to bid", "Enchères - à {} de miser", g.players[a.turn].name),
            None => i18n::t("Auction", "Enchères").to_string(),
        },
        Phase::Debt => crate::tr!(
            "Owes {} - raise cash or go bankrupt",
            "Doit {} - vendez ou faites faillite",
            money(g.debt_amount())
        ),
        Phase::Manage => i18n::t(
            "Build, trade, mortgage, then end your turn",
            "Construisez, échangez, hypothéquez puis terminez",
        )
        .to_string(),
        Phase::Over => i18n::t("Game over", "Partie terminée").to_string(),
    }
}

fn draw_tooltip(g: &Game, geo: &Geo, tile: usize, mx: f32, my: f32) {
    let space = &g.spaces[tile];
    let w = (geo.s * 0.28).clamp(180.0, 250.0);
    let h = w * 1.28;
    let mut x = mx + 22.0;
    if x + w > geo.x + geo.s {
        x = mx - w - 20.0;
    }
    x = x.clamp(6.0, (screen_width() - w - 6.0).max(6.0));
    let mut y = my + 24.0;
    if y + h > geo.y + geo.s {
        y = my - h - 20.0;
    }
    y = y.clamp(6.0, (screen_height() - h - 34.0).max(6.0));

    if space.purchasable() {
        deed_card(g, tile, x, y, w, h);
    } else {
        // Corners, taxes and the card spaces still get a card-shaped note.
        let ch = h * 0.62;
        draw_rectangle(x + 4.0, y + 6.0, w, ch, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.18 });
        draw_rectangle(x, y, w, ch, TILE);
        draw_rectangle_lines(x, y, w, ch, 1.5, INK_LINE);
        centered_bold(
            &space.name.to_uppercase(),
            x + w * 0.5,
            y + 34.0,
            w * 0.070,
            INK,
        );
        let note = match space.kind {
            Kind::Tax => crate::tr!("Pay {}", "Payez {}", money(space.tax as i64)),
            Kind::Chance => i18n::t("Draw a Chance card", "Tirez une carte Chance").to_string(),
            Kind::Chest => {
                i18n::t("Draw a Community Chest card", "Tirez une carte Caisse de communauté")
                    .to_string()
            }
            Kind::GoToJail => i18n::t("Go to jail", "Allez en prison").to_string(),
            Kind::Jail => i18n::t("Just visiting", "Simple visite").to_string(),
            Kind::FreeParking => i18n::t("Free parking", "Parc gratuit").to_string(),
            Kind::Go => crate::tr!("Collect {}", "Recevez {}", money(200)).to_string(),
            _ => i18n::t("Special space", "Case spéciale").to_string(),
        };
        for (k, line) in wrap(&note, w - 20.0, w * 0.050).iter().take(3).enumerate() {
            centered(line, x + w * 0.5, y + 58.0 + k as f32 * w * 0.075, w * 0.050, INK_SOFT);
        }
    }
}

// ------------------------------------------------------------------- columns

/// Serif text broken to fit `max_w` at a fixed size.
fn serif_lines(text: &str, max_w: f32, size: f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };
        if serif_w(&candidate, size) <= max_w || current.is_empty() {
            current = candidate;
        } else {
            out.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn spaced_left(text: &str, x: f32, baseline: f32, size: f32, spacing: f32, color: Color) {
    let mut cx = x;
    for ch in text.chars() {
        let glyph = ch.to_string();
        txt(&glyph, cx, baseline, size, color);
        cx += text_w(&glyph, size) + spacing;
    }
}

/// The line the right-hand card shows: what the game is waiting for.
fn turn_statement(g: &Game) -> (String, String) {
    let player = &g.players[g.turn];
    match g.phase {
        Phase::Roll => {
            if player.in_jail {
                (
                    i18n::t("You are in jail", "Vous êtes en prison").to_string(),
                    i18n::t(
                        "Roll doubles, pay the fine or use a card.",
                        "Faites un double, payez l'amende ou utilisez une carte.",
                    )
                    .to_string(),
                )
            } else if g.rolled_double {
                (
                    i18n::t("Doubles! Roll again", "Double ! Relancez les dés").to_string(),
                    i18n::t("You move again.", "Vous rejouez.").to_string(),
                )
            } else if player.ai {
                (
                    crate::tr!("{} is playing", "{} joue", player.name),
                    i18n::t("The dice are about to roll.", "Les dés vont être lancés.").to_string(),
                )
            } else {
                (
                    i18n::t("Roll the dice to move", "Lancez les dés pour avancer").to_string(),
                    i18n::t("Good luck.", "Bonne chance.").to_string(),
                )
            }
        }
        Phase::Moving => (
            i18n::t("Moving...", "Déplacement...").to_string(),
            String::new(),
        ),
        Phase::AwaitBuy => {
            let space = &g.spaces[player.pos];
            (
                crate::tr!("{} is for sale", "{} est libre", space.name),
                crate::tr!(
                    "Buy it for {} or pass.",
                    "Achetez-la pour {} ou passez.",
                    money(space.price as i64)
                ),
            )
        }
        Phase::Auction => (
            i18n::t("Auction in progress", "Enchères en cours").to_string(),
            i18n::t(
                "The highest bid wins the lot.",
                "La meilleure offre remporte le bien.",
            )
            .to_string(),
        ),
        Phase::Debt => (
            i18n::t("You owe money", "Vous devez de l'argent").to_string(),
            crate::tr!(
                "Sell buildings or mortgage lots to pay {}.",
                "Vendez des constructions ou hypothéquez pour payer {}.",
                money(g.debt_amount())
            ),
        ),
        Phase::Manage => (
            i18n::t("Your move", "À vous de jouer").to_string(),
            i18n::t(
                "Build, trade or mortgage, then end your turn.",
                "Construisez, échangez ou hypothéquez, puis terminez votre tour.",
            )
            .to_string(),
        ),
        Phase::Over => (
            i18n::t("Game over", "Partie terminée").to_string(),
            String::new(),
        ),
    }
}

fn log_color(g: &Game, line: &str) -> Color {
    if line.starts_with("--") {
        return GOLD_DIM;
    }
    for player in g.players.iter() {
        if !player.name.is_empty() && line.contains(&player.name) {
            return player.color;
        }
    }
    DIM_GREEN
}

fn draw_left_column(g: &mut Game, ui: &mut Ui, lay: &Layout) {
    let p = lay.left;
    let n = g.players.len();
    let top = p.y;
    let reserved = 118.0;
    let avail = (p.y + p.h - reserved - top).max(60.0);
    let gap = (p.w * 0.035).clamp(6.0, 12.0);
    let card_h = ((avail - gap * (n as f32 - 1.0)) / n as f32).clamp(50.0, 104.0);
    for i in 0..n {
        let r = Rect::new(p.x, top + i as f32 * (card_h + gap), p.w, card_h);
        draw_player_card(g, i, r);
    }

    let bar = p.y + p.h - 30.0;
    if sound_button(Rect::new(p.x, bar, 34.0, 28.0), Icon::Pause, true) {
        ui.pause_open = true;
    }
    if button(Rect::new(p.x + 42.0, bar, 38.0, 28.0), i18n::code(), true, false) {
        i18n::toggle();
    }
    if sound_button(
        Rect::new(p.x + 88.0, bar, 28.0, 28.0),
        Icon::Sfx,
        crate::audio::sfx_on(),
    ) {
        crate::audio::toggle_sfx();
    }
    if sound_button(
        Rect::new(p.x + 120.0, bar, 28.0, 28.0),
        Icon::Music,
        crate::audio::music_on(),
    ) {
        crate::audio::toggle_music();
    }
    if toggle_button(Rect::new(p.x + 154.0, bar, 54.0, 28.0), ">>>", g.fast) {
        g.fast = !g.fast;
    }
}

fn draw_player_card(g: &Game, index: usize, r: Rect) {
    let player = &g.players[index];
    let current = index == g.turn && g.phase != Phase::Over;
    let fill = if player.bankrupt {
        Color { r: 0.051, g: 0.141, b: 0.106, a: 1.0 }
    } else if current {
        Color { r: 0.090, g: 0.227, b: 0.173, a: 1.0 }
    } else {
        CARD
    };
    round_card(r.x, r.y, r.w, r.h, 12.0, fill, CARD_EDGE);
    if current {
        round_outline(r.x, r.y, r.w, r.h, 12.0, 1.4, GOLD);
    }

    let radius = (r.h * 0.15).clamp(8.0, 15.0);
    let cx = r.x + 13.0 + radius;
    let cy = r.y + 13.0 + radius;
    draw_circle(
        cx,
        cy,
        radius,
        if player.bankrupt { MUTED } else { player.color },
    );
    tokens::draw(player.token, WHITE, cx, cy, radius * 0.74);

    let text_x = cx + radius + 13.0;
    let name_size = (r.h * 0.20).clamp(12.0, 18.0);
    let cash_size = (r.h * 0.20).clamp(12.0, 19.0);
    let cash = money(player.cash);
    let cash_w = serif_w(&cash, cash_size);
    serif_right(
        &cash,
        r.x + r.w - 14.0,
        r.y + name_size + 10.0,
        cash_size,
        if current { GOLD_SOFT } else { CREAM },
    );
    txt_bold(
        &truncate(
            &player.name,
            r.x + r.w - 14.0 - cash_w - 12.0 - text_x,
            name_size,
        ),
        text_x,
        r.y + name_size + 8.0,
        name_size,
        if player.bankrupt { MUTED } else { CREAM },
    );

    let status = if player.bankrupt {
        i18n::t("Bankrupt", "En faillite").to_string()
    } else if current {
        i18n::t("Their turn", "Son tour").to_string()
    } else if player.in_jail {
        i18n::t("In jail", "En prison").to_string()
    } else {
        i18n::t("Waiting", "En attente").to_string()
    };
    txt(
        &status,
        text_x,
        r.y + name_size + 24.0,
        11.0,
        if current { GOLD_SOFT } else { MUTED },
    );

    if r.h < 64.0 {
        return;
    }
    let owned = g.owned_tiles(index);
    let chips_y = r.y + r.h - 14.0;
    let mut x = text_x;
    for tile in owned.iter().take(6) {
        draw_rectangle(x, chips_y - 4.5, 9.0, 9.0, g.spaces[*tile].group.color());
        x += 11.0;
    }
    if !owned.is_empty() {
        x += 3.0;
    }
    let houses: u32 = owned.iter().map(|t| g.props[*t].houses as u32).sum();
    let mut parts = vec![crate::tr!("{} lots", "{} biens", owned.len())];
    if houses > 0 {
        parts.push(crate::tr!("{} houses", "{} maisons", houses));
    }
    let complete = [
        Group::Brown,
        Group::LightBlue,
        Group::Pink,
        Group::Orange,
        Group::Red,
        Group::Yellow,
        Group::Green,
        Group::DarkBlue,
    ]
    .iter()
    .filter(|group| g.has_monopoly(index, **group))
    .count();
    if complete > 0 {
        parts.push(i18n::t("complete set", "série complète").to_string());
    }
    txt(
        &truncate(&parts.join(" · "), r.x + r.w - 14.0 - x, 10.5),
        x,
        chips_y + 3.5,
        10.5,
        MUTED,
    );
}

fn draw_right_column(g: &mut Game, ui: &mut Ui, lay: &Layout, input: bool) {
    let p = lay.right;
    let human = !g.players[g.turn].ai;
    let name = g.players[g.turn].name.clone();
    let cash = g.players[g.turn].cash;
    let color = g.players[g.turn].color;
    let in_jail = g.players[g.turn].in_jail;
    let cards = g.players[g.turn].jail_cards;

    // ---- statement card
    let card_h = (p.h * 0.20).clamp(112.0, 170.0);
    round_card(p.x, p.y, p.w, card_h, 14.0, CARD, CARD_EDGE);
    draw_circle(p.x + 19.0, p.y + 22.0, 5.0, color);
    txt(
        &format!("{} · {}", truncate(&name, p.w - 90.0, 12.5), money(cash)),
        p.x + 31.0,
        p.y + 26.0,
        12.5,
        CREAM_DIM,
    );
    let (title, sub) = turn_statement(g);
    let title_size = (p.w * 0.085).clamp(15.0, 25.0);
    let mut ty = p.y + 40.0 + title_size;
    for line in serif_lines(&title, p.w - 32.0, title_size).iter().take(2) {
        serif_left(line, p.x + 16.0, ty, title_size, GOLD_SOFT);
        ty += title_size * 1.15;
    }
    if !sub.is_empty() {
        let sub_size = (p.w * 0.043).clamp(10.5, 12.5);
        for line in wrap(&sub, p.w - 32.0, sub_size).iter().take(2) {
            txt(line, p.x + 16.0, ty + sub_size, sub_size, CREAM_DIM);
            ty += sub_size * 1.45;
        }
    }

    // ---- actions
    let bh = (p.h * 0.055).clamp(34.0, 50.0);
    let gap = (p.h * 0.012).clamp(7.0, 11.0);
    let half = (p.w - gap) * 0.5;
    let mut y = p.y + card_h + 14.0;
    let price = if buying(g) {
        Some(g.spaces[g.players[g.turn].pos].price as i64)
    } else {
        None
    };

    if let Some(price) = price {
        if button(
            Rect::new(p.x, y, p.w, bh + 4.0),
            &crate::tr!("BUY FOR {}", "ACHETER · {}", money(price)),
            input && cash >= price,
            true,
        ) {
            g.buy();
        }
        y += bh + 4.0 + gap;
        if button(
            Rect::new(p.x, y, p.w, bh),
            i18n::t("PASS", "PASSER"),
            input,
            false,
        ) {
            g.start_auction_current();
        }
        y += bh + gap;
    } else {
        let roll_label = if in_jail && g.phase == Phase::Roll {
            i18n::t("ROLL FOR DOUBLES", "TENTER UN DOUBLE")
        } else if g.rolled_double {
            i18n::t("ROLL AGAIN", "RELANCER")
        } else {
            i18n::t("ROLL THE DICE", "LANCER LES DÉS")
        };
        if button(
            Rect::new(p.x, y, p.w, bh + 4.0),
            roll_label,
            input && human && g.phase == Phase::Roll,
            true,
        ) {
            g.roll();
        }
        y += bh + 4.0 + gap;

        if g.phase == Phase::Debt && human {
            if button(
                Rect::new(p.x, y, p.w, bh),
                &crate::tr!("PAY {}", "PAYER {}", money(g.debt_amount())),
                input && g.can_afford_debt(),
                false,
            ) {
                g.pay_debt();
            }
            y += bh + gap;
        } else if in_jail && g.phase == Phase::Roll {
            let label = if cards > 0 {
                i18n::t("USE A JAIL CARD", "UTILISER UNE CARTE")
            } else {
                i18n::t("PAY THE FINE", "PAYER L'AMENDE")
            };
            if button(Rect::new(p.x, y, p.w, bh), label, input && human, false) {
                if cards > 0 {
                    g.use_jail_card();
                } else {
                    g.pay_jail_fine();
                }
            }
            y += bh + gap;
        }
    }

    if button(
        Rect::new(p.x, y, half, bh),
        i18n::t("ASSETS", "MES BIENS"),
        input && human && g.can_manage(),
        false,
    ) {
        ui.asset_open = true;
        ui.asset_page = 0;
    }
    if button(
        Rect::new(p.x + half + gap, y, half, bh),
        i18n::t("TRADE", "ÉCHANGE"),
        input && human && g.can_manage() && g.active_count() > 1,
        false,
    ) {
        g.open_trade();
    }
    y += bh + gap;

    if button(
        Rect::new(p.x, y, p.w, bh),
        i18n::t("END TURN", "FIN DU TOUR"),
        input && human && g.phase == Phase::Manage,
        false,
    ) {
        g.end_turn();
    }
    y += bh + gap + 2.0;

    if draw_log(g, p, y, input) {
        ui.quit_confirm = true;
    }
}

fn draw_log(g: &Game, p: Rect, top: f32, input: bool) -> bool {
    let h = p.y + p.h - top;
    if h < 56.0 {
        return false;
    }
    round_card(p.x, top, p.w, h, 14.0, CARD, CARD_EDGE);
    spaced_left(
        i18n::t("GAME LOG", "JOURNAL DE PARTIE"),
        p.x + 16.0,
        top + 24.0,
        9.5,
        1.7,
        GOLD_DIM,
    );
    let mut back = false;
    if small_button(Rect::new(p.x + p.w - 64.0, top + 9.0, 50.0, 22.0), i18n::t("MENU", "MENU"), input) {
        back = true;
    }
    let size = (p.w * 0.042).clamp(10.0, 12.0);
    let line_h = size * 1.35;
    let max_lines = (((h - 40.0) / line_h).floor() as usize).max(1);
    let mut lines: Vec<(String, Color, bool)> = Vec::new();
    for msg in g.log.iter().rev() {
        let color = log_color(g, msg);
        let wrapped = wrap(msg, p.w - 44.0, size);
        let mut entry: Vec<(String, Color, bool)> = wrapped
            .into_iter()
            .enumerate()
            .map(|(k, text)| (text, color, k == 0))
            .collect();
        if lines.len() + entry.len() > max_lines {
            break;
        }
        entry.append(&mut lines);
        lines = entry;
    }
    let mut cy = top + 42.0 + size;
    for (text, color, bullet) in lines {
        if bullet {
            draw_circle(p.x + 20.0, cy - size * 0.32, 3.0, color);
        }
        txt(&text, p.x + 30.0, cy, size, color);
        cy += line_h;
    }
    back
}

// ------------------------------------------------------------------- modals

/// Title deed of the property being offered, laid over the board.
fn draw_deed_card(g: &Game, lay: &Layout) {
    let geo = &lay.geo;
    let tile = g.players[g.turn].pos;
    if !g.spaces[tile].purchasable() {
        return;
    }
    deed_card(
        g,
        tile,
        geo.x + geo.s * 0.105,
        geo.y + geo.s * 0.195,
        (geo.s * 0.31).max(150.0),
        (geo.s * 0.40).max(180.0),
    );
}

/// The title deed of one lot, drawn as a cream card.
fn deed_card(g: &Game, tile: usize, x: f32, y: f32, w: f32, h: f32) {
    let space = &g.spaces[tile];
    draw_rectangle(x + 5.0, y + 7.0, w, h, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.22 });
    draw_rectangle(x, y, w, h, TILE);
    draw_rectangle_lines(x, y, w, h, 1.5, INK_LINE);

    let head_h = h * 0.30;
    draw_rectangle(x, y, w, head_h, space.group.color());
    draw_rectangle_lines(x, y, w, head_h, 1.2, INK_LINE);
    // Dark bands (railroads, utilities) need white lettering.
    let head_ink = if space.group.street() { INK } else { WHITE };
    spaced_center(
        i18n::t("TITLE DEED", "TITRE DE PROPRIÉTÉ"),
        x + w * 0.5,
        y + head_h * 0.26,
        w * 0.058,
        w * 0.006,
        head_ink,
    );
    let mut name_size = w * 0.115;
    while serif_w(space.name, name_size) > w - 20.0 && name_size > 8.0 {
        name_size -= 0.5;
    }
    serif_center(space.name, x + w * 0.5, y + head_h * 0.66, name_size, head_ink);

    let mut rows: Vec<(String, String)> = Vec::new();
    if space.group.street() {
        rows.push((
            i18n::t("Rent", "Loyer").to_string(),
            format!("{}$", space.rent[0]),
        ));
        let labels = [
            i18n::t("With 1 house", "Avec 1 maison"),
            i18n::t("With 2 houses", "Avec 2 maisons"),
            i18n::t("With 3 houses", "Avec 3 maisons"),
            i18n::t("With 4 houses", "Avec 4 maisons"),
        ];
        for (k, label) in labels.iter().enumerate() {
            rows.push((label.to_string(), format!("{}$", space.rent[k + 1])));
        }
        rows.push((
            i18n::t("With a hotel", "Avec un hôtel").to_string(),
            format!("{}$", space.rent[5]),
        ));
    } else if space.kind == Kind::Railroad {
        rows.push((
            i18n::t("Price", "Prix").to_string(),
            format!("{}$", space.price),
        ));
        let labels = [
            i18n::t("1 railroad", "1 gare"),
            i18n::t("2 railroads", "2 gares"),
            i18n::t("3 railroads", "3 gares"),
            i18n::t("4 railroads", "4 gares"),
        ];
        for (k, label) in labels.iter().enumerate() {
            rows.push((label.to_string(), format!("{}$", 25u32 << k as u32)));
        }
    } else {
        rows.push((
            i18n::t("Price", "Prix").to_string(),
            format!("{}$", space.price),
        ));
        rows.push((
            i18n::t("Rent, 1 company", "Loyer, 1 réseau").to_string(),
            i18n::t("4 x dice", "4 x dés").to_string(),
        ));
        rows.push((
            i18n::t("Rent, 2 companies", "Loyer, 2 réseaux").to_string(),
            i18n::t("10 x dice", "10 x dés").to_string(),
        ));
    }

    let line = (h - head_h - 68.0) / (rows.len() as f32 + 0.6);
    let mut ry = y + head_h + line * 0.9;
    for (label, value) in rows {
        txt(&label, x + 14.0, ry, w * 0.062, INK);
        text_r_bold(&value, x + w - 14.0, ry, w * 0.062, INK_SOFT);
        draw_line(
            x + 12.0,
            ry + line * 0.28,
            x + w - 12.0,
            ry + line * 0.28,
            1.0,
            INK_LINE,
        );
        ry += line;
    }

    let foot = if space.group.street() {
        crate::tr!(
            "Price {} · house {} each",
            "Prix {} · maison {} chacune",
            money(space.price as i64),
            money(space.house_cost as i64)
        )
    } else {
        crate::tr!("Price {}", "Prix {}", money(space.price as i64))
    };
    centered(&foot, x + w * 0.5, y + h - 34.0, w * 0.052, INK_SOFT);

    // Owner plate, in the same cream-and-gold language as the rest of the card.
    let band_h = 24.0;
    draw_rectangle(
        x,
        y + h - band_h,
        w,
        band_h,
        Color { r: 0.929, g: 0.906, b: 0.839, a: 1.0 },
    );
    draw_line(x, y + h - band_h, x + w, y + h - band_h, 1.2, GOLD_DIM);
    let mortgaged = g.props[tile].mortgaged;
    let (label, dot) = match g.props[tile].owner {
        Some(p) => (
            if mortgaged {
                crate::tr!("{} - mortgaged", "{} - hypothéqué", g.players[p].name)
            } else {
                g.players[p].name.clone()
            },
            g.players[p].color,
        ),
        None => (
            i18n::t("Unowned", "Libre").to_string(),
            Color { r: 0.541, g: 0.514, b: 0.443, a: 1.0 },
        ),
    };
    draw_circle(x + 14.0, y + h - band_h * 0.5, 3.6, dot);
    let ink = if mortgaged { WARN } else { INK_SOFT };
    centered(
        &truncate(&label, w - 36.0, 11.5),
        x + w * 0.5 + 6.0,
        y + h - band_h * 0.5 + 4.0,
        11.5,
        ink,
    );
}

fn draw_auction(g: &mut Game) {
    let Some(auction) = g.auction.as_ref() else {
        return;
    };
    let tile = auction.tile;
    let bid = auction.bid;
    let high = auction.high;
    let turn = auction.turn;
    let active = auction.active.clone();
    let name = g.spaces[tile].name;
    let price = g.spaces[tile].price;
    let bidder_ai = g.players[turn].ai;
    let bidder_name = g.players[turn].name.clone();
    let bidder_cash = g.players[turn].cash;
    let rows: Vec<(String, bool, bool, Color)> = g
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.clone(), active[i], high == Some(i), p.color))
        .collect();

    let r = modal_rect(600.0, 550.0);
    let content = modal_shell(r, i18n::t("AUCTION", "ENCHÈRES"));
    centered_bold(name, r.x + r.w * 0.5, content.y + 20.0, 21.0, TEXT);
    centered(
        &format!("{} {}", i18n::t("List price", "Prix affiché"), money(price as i64)),
        r.x + r.w * 0.5,
        content.y + 46.0,
        12.0,
        MUTED,
    );

    draw_rectangle(r.x + 40.0, content.y + 66.0, r.w - 80.0, 82.0, ROW_BG);
    centered(i18n::t("CURRENT BID", "ENCHÈRE ACTUELLE"), r.x + r.w * 0.5, content.y + 88.0, 11.5, MUTED);
    centered_bold(&money(bid), r.x + r.w * 0.5, content.y + 122.0, 34.0, if bid > 0 { ACCENT } else { TEXT_DIM });

    let mut y = content.y + 168.0;
    for (i, (pname, is_active, is_high, color)) in rows.iter().enumerate() {
        let status = if *is_high {
            i18n::t("HIGH BIDDER", "MEILLEURE OFFRE")
        } else if !*is_active {
            i18n::t("out", "sorti")
        } else if i == turn {
            i18n::t("to bid", "à miser")
        } else {
            i18n::t("in", "en lice")
        };
        draw_circle(r.x + 44.0, y + 8.0, 5.5, *color);
        txt(&truncate(pname, 240.0, 13.0), r.x + 58.0, y + 13.0, 13.0, if *is_active { TEXT } else { MUTED });
        text_r_bold(status, r.x + r.w - 44.0, y + 13.0, 11.0, if *is_high { WARN } else { MUTED });
        y += 24.0;
    }

    let footer = r.y + r.h - 104.0;
    if bidder_ai {
        centered(
            &crate::tr!("{} is thinking...", "{} réfléchit...", bidder_name),
            r.x + r.w * 0.5,
            footer + 24.0,
            14.0,
            TEXT_DIM,
        );
    } else {
        txt(
            &crate::tr!(
                "{} to bid - you have {}",
                "À {} de miser - vous avez {}",
                bidder_name,
                money(bidder_cash)
            ),
            r.x + 30.0,
            footer - 8.0,
            12.0,
            TEXT_DIM,
        );
        let bw = (r.w - 76.0) / 4.0;
        let labels = ["+10", "+50", "+100", i18n::t("PASS", "PASSER")];
        let increments = [10i64, 50, 100, 0];
        for (k, label) in labels.iter().enumerate() {
            let rect = Rect::new(r.x + 30.0 + k as f32 * (bw + 6.0), footer + 10.0, bw, 42.0);
            if increments[k] == 0 {
                if button(rect, label, true, false) {
                    g.auction_pass();
                }
            } else {
                let inc = increments[k];
                if button(rect, label, bid + inc <= bidder_cash, false) {
                    g.auction_bid(inc);
                }
            }
        }
        txt(
            i18n::t(
                "Bids are limited by the bidder's cash.",
                "Les mises sont limitées à l'argent disponible.",
            ),
            r.x + 30.0,
            footer + 72.0,
            10.5,
            MUTED,
        );
    }
}

fn draw_debt(g: &mut Game, ui: &mut Ui) {
    let amount = g.debt_amount();
    let cash = g.players[g.turn].cash;
    let creditor = g.debt.as_ref().and_then(|d| d.creditor);
    let creditor_name = creditor.map(|c| g.players[c].name.clone());
    let name = g.players[g.turn].name.clone();

    let r = modal_rect(540.0, 360.0);
    let content = modal_shell(r, i18n::t("DEBT DUE", "DETTE À PAYER"));
    let target = match &creditor_name {
        Some(c) => c.clone(),
        None => i18n::t("the bank", "la banque").to_string(),
    };
    centered_bold(
        &crate::tr!("{} owes {} to {}", "{} doit {} à {}", name, money(amount), target),
        r.x + r.w * 0.5,
        content.y + 26.0,
        18.0,
        TEXT,
    );
    centered(
        &format!("{} {}", i18n::t("Cash on hand:", "Argent disponible :"), money(cash)),
        r.x + r.w * 0.5,
        content.y + 56.0,
        13.0,
        TEXT_DIM,
    );
    let lines = wrap(
        i18n::t(
            "Sell buildings and mortgage lots to raise the money, then pay. If you cannot cover the debt you must declare bankruptcy.",
            "Vendez des constructions et hypothéquez des biens pour réunir la somme, puis payez. Si vous ne pouvez pas couvrir la dette, vous devez déclarer faillite.",
        ),
        content.w - 20.0,
        12.5,
    );
    for (k, line) in lines.iter().take(3).enumerate() {
        centered(line, r.x + r.w * 0.5, content.y + 88.0 + k as f32 * 18.0, 12.5, MUTED);
    }

    let by = r.y + r.h - 112.0;
    if button(Rect::new(r.x + 26.0, by, 226.0, 40.0), i18n::t("MANAGE ASSETS", "GÉRER MES BIENS"), true, false) {
        ui.asset_open = true;
        ui.asset_page = 0;
    }
    if button(
        Rect::new(r.x + 266.0, by, 248.0, 40.0),
        &crate::tr!("PAY {}", "PAYER {}", money(amount)),
        cash >= amount,
        true,
    ) {
        g.pay_debt();
    }
    let label = if ui.confirm_bankrupt {
        i18n::t("CONFIRM BANKRUPTCY", "CONFIRMER LA FAILLITE")
    } else {
        i18n::t("DECLARE BANKRUPTCY", "DÉCLARER FAILLITE")
    };
    if button(Rect::new(r.x + 26.0, by + 50.0, 488.0, 38.0), label, true, false) {
        if ui.confirm_bankrupt {
            g.declare_bankrupt();
            ui.confirm_bankrupt = false;
        } else {
            ui.confirm_bankrupt = true;
        }
    }
}

fn draw_assets(g: &mut Game, ui: &mut Ui, _lay: &Layout) {
    let p = g.turn;
    let name = g.players[p].name.clone();
    let cash = g.players[p].cash;
    let tiles = g.owned_tiles(p);
    let per_page = 16usize;
    let pages = ((tiles.len() + per_page - 1) / per_page).max(1);
    if ui.asset_page >= pages {
        ui.asset_page = pages - 1;
    }
    let start = ui.asset_page * per_page;
    let slice: Vec<usize> = tiles.iter().skip(start).take(per_page).copied().collect();

    if let Some(tile) = ui.asset_card {
        if tiles.contains(&tile) {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color { r: 0.0, g: 0.0, b: 0.0, a: 0.55 },
            );
            let w = 300.0;
            let h = 384.0;
            let x = (screen_width() - w) * 0.5;
            let y = (screen_height() - h) * 0.5 - 18.0;
            deed_card(g, tile, x, y, w, h);
            if button(
                Rect::new(x + w * 0.5 - 80.0, y + h + 12.0, 160.0, 38.0),
                i18n::t("CLOSE", "FERMER"),
                true,
                false,
            ) {
                ui.asset_card = None;
            }
            return;
        }
        ui.asset_card = None;
    }

    // Eight lots per column, so the window grows with the longest column.
    let rows_used = slice.len().clamp(1, 8) as f32;
    let r = modal_rect(820.0, (256.0 + rows_used * 48.0).max(320.0));
    let title = crate::tr!("BUILD & MORTGAGE - {}", "CONSTRUIRE & HYPOTHÉQUER - {}", name);
    let content = modal_shell(r, &title);
    txt(
        &crate::tr!(
            "Cash {}   |   Net worth {}",
            "Argent {}   |   Patrimoine {}",
            money(cash),
            money(g.net_worth(p))
        ),
        r.x + 22.0,
        content.y + 12.0,
        13.0,
        TEXT_DIM,
    );
    txt(
        &crate::tr!(
            "Bank: {} houses / {} hotels",
            "Banque : {} maisons / {} hôtels",
            g.houses_left,
            g.hotels_left
        ),
        r.x + 22.0,
        content.y + 32.0,
        12.0,
        MUTED,
    );

    let list_top = content.y + 48.0;
    let col_w = (content.w - 14.0) * 0.5;
    if slice.is_empty() {
        centered(
            i18n::t("You do not own anything yet.", "Vous ne possédez encore rien."),
            r.x + r.w * 0.5,
            list_top + 46.0,
            15.0,
            MUTED,
        );
    }

    for (k, tile) in slice.iter().enumerate() {
        let col = k / 8;
        let row = k % 8;
        if col > 1 {
            break;
        }
        let rr = Rect::new(
            content.x + col as f32 * (col_w + 14.0),
            list_top + row as f32 * 48.0,
            col_w,
            44.0,
        );
        let (mx, my) = mouse_position();
        if mx < rr.x + rr.w - 108.0 && inr(rr, mx, my) && is_mouse_button_pressed(MouseButton::Left)
        {
            ui.asset_card = Some(*tile);
        }
        let space = &g.spaces[*tile];
        let gcolor = space.group.color();
        let gtitle = group_title(space.group);
        let is_street = space.group.street();
        let tile_name = space.name;
        let houses = g.props[*tile].houses;
        let mortgaged = g.props[*tile].mortgaged;

        draw_rectangle(rr.x, rr.y, rr.w, rr.h, ROW_BG);
        draw_rectangle(rr.x, rr.y, 6.0, rr.h, gcolor);
        txt_bold(&truncate(tile_name, rr.w - 112.0, 12.0), rr.x + 14.0, rr.y + 18.0, 12.0, TEXT);
        let status = if mortgaged {
            i18n::t("MORTGAGED", "HYPOTHÉQUÉ").to_string()
        } else if houses == MAX_HOUSES {
            i18n::t("HOTEL", "HÔTEL").to_string()
        } else if houses > 0 {
            crate::tr!("{} houses", "{} maisons", houses)
        } else {
            gtitle.to_string()
        };
        txt(&status, rr.x + 14.0, rr.y + 34.0, 10.5, if mortgaged { WARN } else { MUTED });

        let bs = 24.0;
        let by = rr.y + (rr.h - bs) * 0.5;
        let mut bx = rr.x + rr.w - bs - 4.0;
        if is_street {
            let label = if mortgaged { "U" } else { "M" };
            let cost = if mortgaged { g.unmortgage_cost(*tile) } else { g.mortgage_value(*tile) };
            let allowed = if mortgaged { g.players[p].cash >= cost } else { houses == 0 };
            if small_button(Rect::new(bx, by, bs, bs), label, allowed) {
                if mortgaged {
                    g.unmortgage(*tile);
                } else {
                    g.mortgage(*tile);
                }
            }
            bx -= bs + 4.0;
            if small_button(Rect::new(bx, by, bs, bs), "-", g.can_sell_house(*tile)) {
                g.sell_house(*tile);
            }
            bx -= bs + 4.0;
            if small_button(Rect::new(bx, by, bs, bs), "+", g.can_build(*tile)) {
                g.build_house(*tile);
            }
        } else {
            let label = if mortgaged { "U" } else { "M" };
            let allowed = if mortgaged {
                g.players[p].cash >= g.unmortgage_cost(*tile)
            } else {
                true
            };
            if small_button(Rect::new(bx, by, bs, bs), label, allowed) {
                if mortgaged {
                    g.unmortgage(*tile);
                } else {
                    g.mortgage(*tile);
                }
            }
        }
    }

    let footer = r.y + r.h - 66.0;
    if pages > 1 {
        if button(Rect::new(r.x + 22.0, footer, 44.0, 38.0), "<", ui.asset_page > 0, false) {
            ui.asset_page -= 1;
        }
        centered(&format!("{} / {}", ui.asset_page + 1, pages), r.x + 118.0, footer + 19.0, 12.0, MUTED);
        if button(Rect::new(r.x + 150.0, footer, 44.0, 38.0), ">", ui.asset_page + 1 < pages, false) {
            ui.asset_page += 1;
        }
    }
    centered(
        i18n::t(
            "+ build  - sell  M mortgage  U unmortgage  -  click a lot for its deed",
            "+ construire  - vendre  M hypothéquer  U lever  -  cliquez un bien pour sa carte",
        ),
        r.x + r.w * 0.5,
        footer + 8.0,
        11.5,
        MUTED,
    );
    centered(
        i18n::t(
            "Sell all buildings on a lot before mortgaging it.",
            "Vendez les constructions avant d'hypothéquer un bien.",
        ),
        r.x + r.w * 0.5,
        footer + 27.0,
        11.0,
        MUTED,
    );
    if button(Rect::new(r.x + r.w - 154.0, footer, 132.0, 38.0), i18n::t("CLOSE", "FERMER"), true, false) {
        ui.asset_open = false;
    }
}

fn draw_trade(g: &mut Game, ui: &mut Ui) {
    let Some(trade) = g.trade.as_ref() else {
        return;
    };
    let proposer_ai = g.players[trade.from].ai;
    if proposer_ai {
        draw_incoming_offer(g, ui);
    } else {
        draw_trade_editor(g);
    }
}

fn draw_incoming_offer(g: &mut Game, ui: &mut Ui) {
    let Some(trade) = g.trade.as_ref() else {
        return;
    };
    let from = trade.from;
    let to = trade.to;
    let from_name = g.players[from].name.clone();
    let give = trade.give.clone();
    let get = trade.get.clone();
    let give_cash = trade.give_cash;
    let get_cash = trade.get_cash;
    let state = trade.state;
    let is_proposed = state == TradeState::Proposed;

    let r = modal_rect(660.0, 430.0);
    let content = modal_shell(r, i18n::t("TRADE OFFER", "OFFRE D'ÉCHANGE"));
    centered_bold(
        &crate::tr!("{} wants to trade", "{} vous propose un échange", from_name),
        r.x + r.w * 0.5,
        content.y + 18.0,
        19.0,
        TEXT,
    );

    draw_rectangle(content.x, content.y + 44.0, content.w, 108.0, ROW_BG);
    txt_bold(i18n::t("They offer", "Il vous donne"), content.x + 16.0, content.y + 66.0, 12.5, MUTED);
    let offer = format!(
        "{}{}",
        if give_cash > 0 { format!("{} + ", money(give_cash)) } else { String::new() },
        list_names(g, &give)
    );
    txt(&truncate(&offer, content.w - 32.0, 14.0), content.x + 16.0, content.y + 88.0, 14.0, TEXT);
    txt_bold(i18n::t("They want", "Il demande"), content.x + 16.0, content.y + 116.0, 12.5, MUTED);
    let ask = format!(
        "{}{}",
        if get_cash > 0 { format!("{} + ", money(get_cash)) } else { String::new() },
        list_names(g, &get)
    );
    txt(&truncate(&ask, content.w - 32.0, 14.0), content.x + 16.0, content.y + 138.0, 14.0, TEXT);

    let footer = r.y + r.h - 72.0;
    if is_proposed {
        if button(Rect::new(r.x + 40.0, footer, 270.0, 46.0), i18n::t("ACCEPT", "ACCEPTER"), true, true) {
            g.accept_trade();
        }
        if button(Rect::new(r.x + 330.0, footer, 290.0, 46.0), i18n::t("DECLINE", "REFUSER"), true, false) {
            g.decline_trade();
        }
        let _ = to;
        let _ = ui;
    }
}

fn draw_trade_editor(g: &mut Game) {
    let Some(trade) = g.trade.as_ref() else {
        return;
    };
    let from = trade.from;
    let to = trade.to;
    let state = trade.state;
    let give = trade.give.clone();
    let get = trade.get.clone();
    let give_cash = trade.give_cash;
    let get_cash = trade.get_cash;
    let notice = trade.notice.clone();
    let my_name = g.players[from].name.clone();
    let partner_name = g.players[to].name.clone();
    let partner_ai = g.players[to].ai;
    let my_cash = g.players[from].cash;
    let partner_cash = g.players[to].cash;
    let mine = g.owned_tiles(from);
    let theirs = g.owned_tiles(to);
    let draft = state == TradeState::Draft;

    let r = modal_rect(900.0, 660.0);
    let content = modal_shell(r, i18n::t("TRADE", "ÉCHANGE"));
    let col_w = (content.w - 18.0) * 0.5;
    let left_x = content.x;
    let right_x = content.x + col_w + 18.0;

    txt_bold(
        &crate::tr!("YOU - {}", "VOUS - {}", truncate(&my_name, col_w - 90.0, 14.0)),
        left_x,
        content.y + 14.0,
        14.0,
        TEXT,
    );
    text_r_bold(&money(my_cash), left_x + col_w, content.y + 14.0, 13.0, ACCENT_SOFT);
    txt_bold(
        &crate::tr!(
            "PARTNER - {}",
            "PARTENAIRE - {}",
            truncate(&partner_name, col_w - 150.0, 14.0)
        ),
        right_x,
        content.y + 14.0,
        14.0,
        TEXT,
    );
    text_r_bold(&money(partner_cash), right_x + col_w, content.y + 14.0, 13.0, ACCENT_SOFT);

    if draft && g.active_count() > 2 {
        if button(
            Rect::new(r.x + r.w - 186.0, r.y + 9.0, 162.0, 28.0),
            i18n::t("CHANGE PARTNER", "CHANGER"),
            true,
            false,
        ) {
            g.cycle_trade_partner();
        }
    }

    let list_top = content.y + 30.0;
    let row_h = 24.0;
    let max_rows = 10usize;

    for (k, tile) in mine.iter().take(max_rows).enumerate() {
        let rr = Rect::new(left_x, list_top + k as f32 * row_h, col_w, row_h - 2.0);
        let selected = give.contains(tile);
        let space = &g.spaces[*tile];
        let has_houses = g.props[*tile].houses > 0;
        draw_rectangle(rr.x, rr.y, rr.w, rr.h, if selected { SELECT } else { ROW_BG });
        draw_rectangle(rr.x, rr.y, 6.0, rr.h, space.group.color());
        txt(&truncate(space.name, col_w - 120.0, 12.0), rr.x + 14.0, rr.y + 16.0, 12.0, if has_houses { MUTED } else { TEXT });
        text_r(&format!("{} $", space.price), rr.x + col_w - 10.0, rr.y + 16.0, 11.0, MUTED);
        if draft && inr(rr, mouse_position().0, mouse_position().1) && is_mouse_button_pressed(MouseButton::Left) {
            g.trade_toggle(*tile);
        }
    }
    if mine.len() > max_rows {
        txt(&format!("+{}", mine.len() - max_rows), left_x, list_top + max_rows as f32 * row_h + 12.0, 11.0, MUTED);
    }
    if mine.is_empty() {
        txt(i18n::t("No properties", "Aucun bien"), left_x, list_top + 16.0, 12.0, MUTED);
    }

    for (k, tile) in theirs.iter().take(max_rows).enumerate() {
        let rr = Rect::new(right_x, list_top + k as f32 * row_h, col_w, row_h - 2.0);
        let selected = get.contains(tile);
        let space = &g.spaces[*tile];
        let has_houses = g.props[*tile].houses > 0;
        draw_rectangle(rr.x, rr.y, rr.w, rr.h, if selected { SELECT } else { ROW_BG });
        draw_rectangle(rr.x, rr.y, 6.0, rr.h, space.group.color());
        txt(&truncate(space.name, col_w - 120.0, 12.0), rr.x + 14.0, rr.y + 16.0, 12.0, if has_houses { MUTED } else { TEXT });
        text_r(&format!("{} $", space.price), rr.x + col_w - 10.0, rr.y + 16.0, 11.0, MUTED);
        if draft && inr(rr, mouse_position().0, mouse_position().1) && is_mouse_button_pressed(MouseButton::Left) {
            g.trade_toggle(*tile);
        }
    }
    if theirs.is_empty() {
        txt(i18n::t("No properties", "Aucun bien"), right_x, list_top + 16.0, 12.0, MUTED);
    }

    let cash_y = list_top + max_rows as f32 * row_h + 26.0;
    txt_bold(i18n::t("You pay", "Vous payez"), left_x, cash_y, 12.5, MUTED);
    txt_bold(&money(give_cash), left_x + 96.0, cash_y, 15.0, ACCENT_SOFT);
    if small_button(Rect::new(left_x + 180.0, cash_y - 16.0, 32.0, 24.0), "-", draft) {
        g.trade_cash(true, -50);
    }
    if small_button(Rect::new(left_x + 218.0, cash_y - 16.0, 32.0, 24.0), "+", draft) {
        g.trade_cash(true, 50);
    }
    txt_bold(i18n::t("You receive", "Vous recevez"), left_x, cash_y + 32.0, 12.5, MUTED);
    txt_bold(&money(get_cash), left_x + 112.0, cash_y + 32.0, 15.0, ACCENT_SOFT);
    if small_button(Rect::new(left_x + 196.0, cash_y + 16.0, 32.0, 24.0), "-", draft) {
        g.trade_cash(false, -50);
    }
    if small_button(Rect::new(left_x + 234.0, cash_y + 16.0, 32.0, 24.0), "+", draft) {
        g.trade_cash(false, 50);
    }

    let summary = crate::tr!(
        "Offer: you give {} and {}; you get {} and {}.",
        "Offre : vous donnez {} et {} ; vous recevez {} et {}.",
        list_names(g, &give),
        money(give_cash),
        list_names(g, &get),
        money(get_cash)
    );
    let lines = wrap(&summary, content.w, 12.0);
    for (k, line) in lines.iter().take(3).enumerate() {
        txt(line, content.x, cash_y + 68.0 + k as f32 * 16.0, 12.0, TEXT_DIM);
    }
    if !notice.is_empty() {
        txt(&truncate(&notice, content.w, 12.0), content.x, r.y + r.h - 96.0, 12.0, WARN);
    }

    let footer = r.y + r.h - 64.0;
    if draft {
        if button(Rect::new(r.x + 24.0, footer, 250.0, 44.0), i18n::t("PROPOSE TRADE", "PROPOSER"), true, true) {
            g.propose_trade();
        }
        if button(Rect::new(r.x + 286.0, footer, 170.0, 44.0), i18n::t("CANCEL", "ANNULER"), true, false) {
            g.close_trade();
        }
    } else if partner_ai {
        centered(
            &crate::tr!("{} is considering the offer...", "{} examine l'offre...", partner_name),
            r.x + r.w * 0.5 - 90.0,
            footer + 22.0,
            13.5,
            TEXT_DIM,
        );
        if button(Rect::new(r.x + r.w - 200.0, footer, 176.0, 44.0), i18n::t("WITHDRAW", "RETIRER"), true, false) {
            g.withdraw_trade();
        }
    } else {
        txt_bold(
            &crate::tr!("{}, do you accept?", "{}, acceptez-vous ?", partner_name),
            r.x + 24.0,
            footer + 26.0,
            14.0,
            TEXT,
        );
        if button(Rect::new(r.x + r.w - 360.0, footer, 160.0, 44.0), i18n::t("ACCEPT", "ACCEPTER"), true, true) {
            g.accept_trade();
        }
        if button(Rect::new(r.x + r.w - 186.0, footer, 162.0, 44.0), i18n::t("DECLINE", "REFUSER"), true, false) {
            g.decline_trade();
        }
    }
}

fn draw_over(g: &mut Game) -> bool {
    let r = modal_rect(660.0, 640.0);
    let content = modal_shell(r, i18n::t("GAME OVER", "PARTIE TERMINÉE"));
    let mut order: Vec<usize> = (0..g.players.len()).collect();
    order.sort_by_key(|i| std::cmp::Reverse(g.net_worth(*i)));

    if let Some(w) = g.winner {
        centered(i18n::t("WINNER", "VAINQUEUR"), r.x + r.w * 0.5, content.y + 16.0, 12.0, MUTED);
        centered_bold(&g.players[w].name, r.x + r.w * 0.5, content.y + 48.0, 36.0, g.players[w].color);
        centered(
            &format!("{} {}", i18n::t("Net worth", "Patrimoine"), money(g.net_worth(w))),
            r.x + r.w * 0.5,
            content.y + 78.0,
            14.0,
            TEXT_DIM,
        );
    }

    let mut y = content.y + 112.0;
    txt_bold(i18n::t("STANDINGS", "CLASSEMENT"), r.x + 30.0, y, 11.0, MUTED);
    y += 12.0;
    for (rank, i) in order.iter().enumerate() {
        let row = Rect::new(r.x + 28.0, y, r.w - 56.0, 38.0);
        draw_rectangle(row.x, row.y, row.w, row.h, ROW_BG);
        draw_circle(row.x + 19.0, row.y + 19.0, 9.0, g.players[*i].color);
        centered_bold(&(rank + 1).to_string(), row.x + 19.0, row.y + 19.0, 10.0, WHITE);
        txt_bold(
            &truncate(&g.players[*i].name, row.w - 260.0, 13.5),
            row.x + 38.0,
            row.y + 24.0,
            13.5,
            if g.players[*i].bankrupt { MUTED } else { TEXT },
        );
        text_r(
            &format!(
                "{}{}",
                if g.players[*i].bankrupt {
                    i18n::t("bankrupt - ", "faillite - ")
                } else {
                    ""
                },
                money(g.net_worth(*i))
            ),
            row.x + row.w - 12.0,
            row.y + 24.0,
            12.0,
            if g.players[*i].bankrupt { MUTED } else { TEXT_DIM },
        );
        y += 42.0;
    }

    if button(
        Rect::new(r.x + r.w * 0.5 - 120.0, r.y + r.h - 66.0, 240.0, 46.0),
        i18n::t("PLAY AGAIN", "REJOUER"),
        true,
        true,
    ) {
        return true;
    }
    false
}



// ------------------------------------------------------------ menu & dialogs

// ---------------------------------------------------------------- rule book

// The rulebook is printed on cream paper, in wine and gold like the table.
const PAPER: Color = Color { r: 0.949, g: 0.918, b: 0.839, a: 1.0 };
const PAPER_EDGE: Color = Color { r: 0.780, g: 0.714, b: 0.553, a: 1.0 };
const BOOK_INK: Color = Color { r: 0.169, g: 0.157, b: 0.129, a: 1.0 };
const BOOK_SOFT: Color = Color { r: 0.376, g: 0.345, b: 0.278, a: 1.0 };
const BOOK_WINE: Color = Color { r: 0.478, g: 0.180, b: 0.212, a: 1.0 };
const BOOK_GOLD: Color = Color { r: 0.690, g: 0.549, b: 0.310, a: 1.0 };

/// A paragraph or a bullet: the only two kinds of text a rule page holds.
enum Blk {
    P(&'static str, &'static str),
    B(&'static str, &'static str),
}

/// One rule and the clauses that belong to it. A section is always placed
/// whole, so a rule is never cut by a column break or the foot of a page.
struct Section {
    head: (&'static str, &'static str),
    blocks: &'static [Blk],
}

struct BookPage {
    title: (&'static str, &'static str),
    sections: &'static [Section],
}

static BOOK: [BookPage; 4] = [
    BookPage {
        title: ("The table", "La table"),
        sections: &[
            Section {
                head: ("The goal", "Le but"),
                blocks: &[Blk::P(
                    "Crazyopoly is a property trading game for two to eight players sharing one screen. Players buy streets, build on them, trade them and collect rent. The game ends when a single solvent player remains; if the table decides otherwise, the ranking is settled on net worth.",
                    "Crazyopoly est un jeu de négociation immobilière pour deux à huit joueurs réunis sur un seul écran. Les joueurs achètent des rues, les construisent, les échangent et encaissent les loyers. La partie s'arrête quand un seul joueur reste solvable ; si la table en décide autrement, le classement se fait sur la fortune nette.",
                )],
            },
            Section {
                head: ("The setup", "La mise en place"),
                blocks: &[
                    Blk::B(
                        "Two to eight seats. Each seat is held by a human or by a bot, and takes a name and a token.",
                        "De deux à huit sièges. Chaque siège est tenu par un humain ou par un bot, et reçoit un nom et un jeton.",
                    ),
                    Blk::B(
                        "Every player starts on GO with the same amount of cash.",
                        "Tous les joueurs partent de la case DÉPART avec la même somme d'argent.",
                    ),
                    Blk::B(
                        "The game begins once the seats are ready; a saved game is resumed from the main menu.",
                        "La partie commence dès que les sièges sont prêts ; une partie sauvegardée se reprend depuis le menu principal.",
                    ),
                    Blk::B(
                        "A table made only of bots plays by itself.",
                        "Une table entièrement composée de bots joue toute seule.",
                    ),
                ],
            },
            Section {
                head: ("Order of play", "L'ordre de jeu"),
                blocks: &[Blk::P(
                    "Players take their turns in seat order, clockwise. The player whose turn it is rolls both dice, moves the token by the total, then resolves the space reached.",
                    "Les joueurs jouent dans l'ordre des sièges, dans le sens des aiguilles d'une montre. Le joueur dont c'est le tour lance les deux dés, avance son jeton du total obtenu, puis résout la case atteinte.",
                )],
            },
            Section {
                head: ("Doubles", "Les doubles"),
                blocks: &[Blk::P(
                    "A double grants a second roll to the same player. Three doubles in a row send that player to Jail without moving.",
                    "Un double offre un second lancer au même joueur. Trois doubles consécutifs envoient ce joueur en prison sans qu'il avance.",
                )],
            },
            Section {
                head: ("Ending a turn", "La fin du tour"),
                blocks: &[Blk::P(
                    "Once the space is resolved, the player may build, mortgage or trade, then ends the turn and play passes to the next seat. Nothing is decided in the player's place.",
                    "Une fois la case résolue, le joueur peut construire, hypothéquer ou négocier, puis termine son tour et la main passe au siège suivant. Aucune décision n'est prise à la place du joueur.",
                )],
            },
        ],
    },
    BookPage {
        title: ("The board", "Le plateau"),
        sections: &[
            Section {
                head: ("The spaces", "Les cases"),
                blocks: &[Blk::P(
                    "Forty spaces: twenty-two streets in eight colour groups, four railroads, two utilities, four corner spaces, two tax spaces, the Jail, Free Parking, GO and the two card decks.",
                    "Quarante cases : vingt-deux rues réparties en huit groupes de couleur, quatre gares, deux compagnies, quatre cases d'angle, deux cases d'impôt, la prison, le parc gratuit, la case DÉPART et les deux paquets de cartes.",
                )],
            },
            Section {
                head: ("Colour groups", "Les groupes de couleur"),
                blocks: &[Blk::P(
                    "Each group gathers two or three streets of the same colour, and holding all of them doubles the rent. Street names stay in English, as on the original board; the rest of the interface follows the language chosen.",
                    "Chaque groupe réunit deux ou trois rues de même couleur, et le fait de les détenir toutes double le loyer. Les noms de rues restent en anglais, comme sur le plateau d'origine ; le reste de l'interface suit la langue choisie.",
                )],
            },
            Section {
                head: ("GO", "Le départ"),
                blocks: &[Blk::P(
                    "The salary is paid on every pass over GO and on every landing there.",
                    "Le salaire est versé à chaque passage sur la case DÉPART et à chaque arrêt sur cette case.",
                )],
            },
            Section {
                head: ("Buying", "L'achat"),
                blocks: &[Blk::P(
                    "On a street, a railroad or a utility that is still free, the player may buy the lot at the price printed on the space.",
                    "Sur une rue, une gare ou une compagnie encore libre, le joueur peut acheter le lot au prix affiché sur la case.",
                )],
            },
            Section {
                head: ("Auctions", "Les enchères"),
                blocks: &[Blk::P(
                    "A player who declines to buy sends the lot to auction. Every player bids in turn, up to the cash on hand; the highest bid wins the lot, which returns to the bank if nobody bids.",
                    "Le joueur qui renonce à l'achat envoie le lot aux enchères. Chacun mise à son tour, dans la limite de son argent liquide ; la plus haute offre l'emporte, et le lot retourne à la banque si personne ne mise.",
                )],
            },
            Section {
                head: ("Rent", "Les loyers"),
                blocks: &[
                    Blk::B(
                        "Street: the base rent, doubled once the owner holds the whole colour group.",
                        "Rue : le loyer de base, doublé dès que le propriétaire détient tout le groupe de couleur.",
                    ),
                    Blk::B(
                        "Railroad: twenty-five, fifty, one hundred or two hundred, depending on how many are held.",
                        "Gare : vingt-cinq, cinquante, cent ou deux cents, selon le nombre de gares détenues.",
                    ),
                    Blk::B(
                        "Utility: four times the dice with one utility, ten times with both.",
                        "Compagnie : quatre fois les dés avec une seule compagnie, dix fois avec les deux.",
                    ),
                    Blk::B(
                        "A mortgaged lot pays no rent.",
                        "Un lot hypothéqué ne rapporte aucun loyer.",
                    ),
                ],
            },
        ],
    },
    BookPage {
        title: ("Building", "Construire"),
        sections: &[
            Section {
                head: ("Houses and hotels", "Maisons et hôtels"),
                blocks: &[Blk::P(
                    "Building requires the whole colour group and is done evenly: no street of the group may hold two houses more than another. The hotel is the fifth building; it replaces the four houses and costs one further house price.",
                    "La construction exige la totalité du groupe de couleur et se fait à effectif égal : aucune rue du groupe ne peut compter deux maisons de plus qu'une autre. L'hôtel est la cinquième construction ; il remplace les quatre maisons et coûte un prix de maison supplémentaire.",
                )],
            },
            Section {
                head: ("Selling and mortgaging", "Vendre et hypothéquer"),
                blocks: &[Blk::P(
                    "Buildings are sold back for half their price, still evenly. Mortgaging a lot yields its mortgage value; redeeming it costs that value plus ten per cent interest. A mortgaged lot cannot be built on.",
                    "Les constructions se revendent à la moitié de leur prix, toujours à effectif égal. Hypothéquer un lot en rapporte la valeur hypothécaire ; le dégager coûte cette valeur majorée de dix pour cent d'intérêt. Un lot hypothéqué ne peut pas être construit.",
                )],
            },
            Section {
                head: ("The bank", "La banque"),
                blocks: &[Blk::P(
                    "The bank never runs short of cash or of buildings: houses, hotels and notes are supplied for as long as the game asks for them.",
                    "La banque ne manque ni d'argent ni de bâtiments : maisons, hôtels et billets sont fournis tant que la partie en demande.",
                )],
            },
            Section {
                head: ("Trades", "Les échanges"),
                blocks: &[Blk::P(
                    "Two players may exchange lots and cash during their turn. A bot weighs the offer and answers it; two bots may settle between themselves without help.",
                    "Deux joueurs peuvent échanger des lots et de l'argent pendant leur tour. Un bot évalue l'offre et y répond ; deux bots peuvent conclure entre eux sans intervention.",
                )],
            },
            Section {
                head: ("Jail", "La prison"),
                blocks: &[Blk::P(
                    "A player enters Jail through a space, through a card, or through three doubles. Three ways out: roll a double, pay the fine, or present a Get Out of Jail Free card. After three turns without a double, the fine is due.",
                    "Le joueur entre en prison par une case, par une carte ou par trois doubles. Trois issues s'offrent à lui : tenter un double, payer l'amende, ou présenter une carte Sortie de prison. Après trois tours sans double, l'amende est due.",
                )],
            },
        ],
    },
    BookPage {
        title: ("Cards and the end", "Cartes et fin de partie"),
        sections: &[
            Section {
                head: ("Chance and Community Chest", "Chance et Caisse de communauté"),
                blocks: &[Blk::P(
                    "The two decks mix moves, payments, repairs and Get Out of Jail Free cards. A card drawn applies at once, then returns under its deck.",
                    "Les deux paquets mêlent déplacements, perceptions, réparations et cartes de sortie de prison. Une carte tirée s'applique immédiatement, puis retourne sous son paquet.",
                )],
            },
            Section {
                head: ("Repairs", "Les réparations"),
                blocks: &[Blk::P(
                    "Repair cards are paid per house and per hotel, over every property the player holds.",
                    "Les cartes de réparation se paient par maison et par hôtel, sur l'ensemble des biens du joueur.",
                )],
            },
            Section {
                head: ("Debts and bankruptcy", "Dettes et faillite"),
                blocks: &[Blk::P(
                    "A player who cannot meet a debt sells buildings, mortgages lots, or declares bankruptcy. The estate then passes to the creditor, or to the bank.",
                    "Le joueur qui ne peut pas honorer une dette vend ses constructions, hypothèque ses lots, ou se déclare en faillite. Ses biens passent alors à son créancier, ou à la banque.",
                )],
            },
            Section {
                head: ("The end of the game", "La fin de partie"),
                blocks: &[Blk::P(
                    "The game ends when one solvent player remains. The table may also vote to stop it at any moment; the ranking is then settled on net worth, cash and property together, buildings counted and mortgages deducted.",
                    "La partie s'achève quand un seul joueur reste solvable. La table peut aussi voter son arrêt à tout moment ; le classement se fait alors sur la fortune nette, argent et biens compris, constructions comptées et hypothèques déduites.",
                )],
            },
            Section {
                head: ("The pause menu", "Le menu pause"),
                blocks: &[Blk::P(
                    "Escape freezes the table and opens the pause menu, from which one resumes, saves, rereads these rules or proposes to stop the game. A save is a single file.",
                    "La touche Échap fige la table et ouvre le menu pause, d'où l'on reprend, sauvegarde, relit ces règles ou propose d'arrêter la partie. Une sauvegarde tient dans un seul fichier.",
                )],
            },
        ],
    },
];

fn draw_menu(app: &mut App) {
    felt_background();
    let sw = screen_width();
    let sh = screen_height();
    let cx = sw * 0.5;
    // The rulebook and the save picker both take every click while open.
    let locked = app.ui.rules_open || app.ui.load_open;
    let t = get_time() as f32;

    // Gilt frame and a quiet lattice of lines, like the edge of a board.
    draw_rectangle_lines(
        14.0,
        14.0,
        sw - 28.0,
        sh - 28.0,
        2.0,
        Color { r: 0.843, g: 0.706, b: 0.396, a: 0.16 },
    );
    draw_rectangle_lines(
        20.0,
        20.0,
        sw - 40.0,
        sh - 40.0,
        1.0,
        Color { r: 0.843, g: 0.706, b: 0.396, a: 0.07 },
    );

    // Gold motes drifting up the table, like dust in a lamp's light.
    for i in 0..36 {
        let fi = i as f32;
        let speed = 9.0 + (i % 5) as f32 * 4.5;
        let y = sh + 20.0 - ((t * speed + fi * 71.0) % (sh + 40.0));
        let x = (fi * 137.0 + (t * 0.6 + fi).sin() * 26.0).rem_euclid(sw);
        let a = 0.045 + 0.075 * ((t * 1.7 + fi * 0.9).sin() * 0.5 + 0.5);
        draw_circle(
            x,
            y,
            1.1 + (i % 3) as f32 * 0.6,
            Color { r: 0.945, g: 0.863, b: 0.647, a },
        );
    }

    let mut size = (sw * 0.085).clamp(46.0, 104.0);
    let title = "Crazyopoly";
    while serif_w(title, size) > sw * 0.70 && size > 20.0 {
        size -= 1.0;
    }
    let pulse = (t * 0.9).sin() * 0.5 + 0.5;
    serif_center(
        title,
        cx,
        sh * 0.176,
        size * 1.04,
        Color { r: 0.945, g: 0.863, b: 0.647, a: 0.04 + 0.05 * pulse },
    );
    serif_center(title, cx, sh * 0.176, size, GOLD_SOFT);
    spaced_center(
        i18n::t("THE PROPERTY TRADING GAME", "LE JEU DE NÉGOCE IMMOBILIER"),
        cx,
        sh * 0.262,
        (sw * 0.0085).clamp(10.0, 15.0),
        3.2,
        CREAM_DIM,
    );
    draw_line(
        cx - sw * 0.085,
        sh * 0.292,
        cx + sw * 0.085,
        sh * 0.292,
        1.4,
        GOLD_DIM,
    );

    let w = (sw * 0.34).clamp(300.0, 440.0);
    let bh = 54.0;
    let gap = 12.0;
    let card_h = 3.0 * bh + 2.0 * gap + 36.0;
    let x = cx - w * 0.5;
    let y = sh * 0.372;
    round_card(x, y, w, card_h, 18.0, PANEL_BG, CARD_EDGE);

    let mut by = y + 18.0;
    if !locked
        && button(
            Rect::new(x + 18.0, by, w - 36.0, bh),
            i18n::t("NEW GAME", "NOUVELLE PARTIE"),
            true,
            true,
        )
    {
        app.screen = Screen::Setup;
    }
    by += bh + gap;
    let has_save = crate::persist::has_saves();
    if !locked
        && button(
            Rect::new(x + 18.0, by, w - 36.0, bh),
            i18n::t("CONTINUE", "REPRENDRE"),
            has_save,
            false,
        )
    {
        if let Some(game) = open_saves(&mut app.ui) {
            app.game = Some(game);
            app.ui = Ui::new();
            app.screen = Screen::Play;
        }
    }
    by += bh + gap;
    if !locked
        && button(
            Rect::new(x + 18.0, by, w - 36.0, bh),
            i18n::t("RULES OF THE GAME", "RÈGLES DU JEU"),
            true,
            false,
        )
    {
        app.ui.rules_open = true;
        app.ui.rules_page = 0;
    }

    // A highlight sweeps across the new-game button now and then.
    let bx = x + 18.0;
    let bw = w - 36.0;
    let sweep = (t * 0.32).fract();
    let sx = bx - bw * 0.35 + sweep * (bw * 1.7);
    let mut px = bx;
    while px < bx + bw {
        let d = (px - sx).abs() / (bw * 0.10);
        let a = (1.0 - d).max(0.0) * 0.09;
        if a > 0.002 {
            draw_rectangle(px, y + 18.0, 2.0, bh, Color { r: 1.0, g: 1.0, b: 1.0, a });
        }
        px += 2.0;
    }

    // The eight tokens, bobbing like pieces waiting on the table.
    let step = (sw * 0.05).clamp(46.0, 66.0);
    let start = cx - step * 3.5;
    for k in 0..TOKENS.len() {
        let phase = t * 1.7 + k as f32 * 0.85;
        let tx = start + step * k as f32;
        let ty = y + card_h + 54.0 + phase.sin() * 5.0;
        draw_circle(
            tx,
            ty + 9.0,
            step * 0.30,
            Color { r: 0.0, g: 0.0, b: 0.0, a: 0.20 },
        );
        let halo = 0.09 + 0.08 * (phase.cos() * 0.5 + 0.5);
        draw_circle(
            tx,
            ty,
            step * 0.37,
            Color { r: TOKENS[k].r, g: TOKENS[k].g, b: TOKENS[k].b, a: halo },
        );
        tokens::draw(k, TOKENS[k], tx, ty, step * 0.30);
    }

    // Language and sound, in the top right corner.
    let bar_x = sw - 190.0;
    if !locked && button(Rect::new(bar_x, 20.0, 56.0, 30.0), i18n::code(), true, false) {
        i18n::toggle();
        app.setup.relabel_defaults();
    }
    if !locked
        && sound_button(
            Rect::new(bar_x + 64.0, 20.0, 34.0, 30.0),
            Icon::Sfx,
            crate::audio::sfx_on(),
        )
    {
        crate::audio::toggle_sfx();
    }
    if !locked
        && sound_button(
            Rect::new(bar_x + 104.0, 20.0, 34.0, 30.0),
            Icon::Music,
            crate::audio::music_on(),
        )
    {
        crate::audio::toggle_music();
    }

    centered(
        &format!(
            "Crazyopoly {} - {}",
            env!("CARGO_PKG_VERSION"),
            i18n::t("a Monopoly-like game in Rust", "un jeu inspiré de Monopoly, en Rust")
        ),
        cx,
        sh - 24.0,
        11.0,
        DIM_GREEN,
    );

    if app.ui.rules_open {
        draw_rules(&mut app.ui);
    }
    if app.ui.load_open {
        draw_load(app);
    }
}

fn draw_rules(ui: &mut Ui) {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.02, g: 0.06, b: 0.045, a: 0.62 });

    if is_key_pressed(KeyCode::Left) {
        ui.rules_page = ui.rules_page.saturating_sub(1);
    }
    if is_key_pressed(KeyCode::Right) {
        ui.rules_page = (ui.rules_page + 1).min(BOOK.len() - 1);
    }
    ui.rules_page = ui.rules_page.min(BOOK.len() - 1);

    let w = (sw * 0.90).min(1180.0);
    let h = (sh * 0.88).min(786.0);
    let r = Rect::new((sw - w) * 0.5, (sh - h) * 0.5, w, h);
    // The pages under the open one.
    for k in (1..=2).rev() {
        let o = k as f32 * 5.0;
        round_card(
            r.x + o,
            r.y + o,
            r.w,
            r.h,
            14.0,
            Color { r: 0.898, g: 0.867, b: 0.784, a: 1.0 },
            PAPER_EDGE,
        );
    }
    draw_rectangle(r.x + 6.0, r.y + 8.0, r.w, r.h, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.30 });
    round_card(r.x, r.y, r.w, r.h, 14.0, PAPER, PAPER_EDGE);
    draw_rectangle_lines(
        r.x + 11.0,
        r.y + 11.0,
        r.w - 22.0,
        r.h - 22.0,
        1.0,
        Color { r: BOOK_GOLD.r, g: BOOK_GOLD.g, b: BOOK_GOLD.b, a: 0.5 },
    );

    let page = &BOOK[ui.rules_page];
    let pad = 52.0;
    let cx = r.x + r.w * 0.5;
    spaced_center("CRAZYOPOLY", cx, r.y + 34.0, 9.5, 3.0, BOOK_GOLD);
    serif_center(i18n::t(page.title.0, page.title.1), cx, r.y + 76.0, 31.0, BOOK_WINE);
    draw_line(cx - 92.0, r.y + 90.0, cx + 92.0, r.y + 90.0, 1.2, BOOK_GOLD);

    let gutter = 46.0;
    let col_w = (r.w - pad * 2.0 - gutter) * 0.5;
    let line_h = 25.0;
    let top = r.y + 122.0;
    let bottom = r.y + h - 106.0;

    // Measure every section whole, then place it: a rule never straddles the
    // two columns nor the foot of the page.
    struct Item {
        kind: u8,
        lines: Vec<String>,
        h: f32,
    }
    let mut sections: Vec<(Vec<Item>, f32)> = Vec::new();
    for section in page.sections.iter() {
        let mut items: Vec<Item> = Vec::new();
        items.push(Item {
            kind: 0,
            lines: vec![i18n::t(section.head.0, section.head.1).to_uppercase()],
            h: 32.0,
        });
        for blk in section.blocks.iter() {
            let (kind, text) = match blk {
                Blk::P(en, fr) => (1u8, i18n::t(en, fr).to_string()),
                Blk::B(en, fr) => (2u8, i18n::t(en, fr).to_string()),
            };
            let width = if kind == 2 { col_w - 17.0 } else { col_w };
            let lines = wrap(&text, width, 16.5);
            let h = lines.len() as f32 * line_h + if kind == 2 { 10.0 } else { 14.0 };
            items.push(Item { kind, lines, h });
        }
        let height: f32 = items.iter().map(|i| i.h).sum::<f32>() + 16.0;
        sections.push((items, height));
    }

    // Split where the two columns end closest together, so a page looks set
    // rather than poured to one side; a section still never straddles the two.
    let col_h = bottom - top;
    let heights: Vec<f32> = sections.iter().map(|(_, h)| *h).collect();
    let total: f32 = heights.iter().sum();
    let mut split = sections.len();
    if total > col_h {
        let mut best = f32::MAX;
        let mut left = 0.0;
        for k in 1..sections.len() {
            left += heights[k - 1];
            let right = total - left;
            if left <= col_h && right <= col_h {
                let skew = (left - right).abs();
                if skew < best {
                    best = skew;
                    split = k;
                }
            }
        }
        if best == f32::MAX {
            // One section is taller than a column: fall back to plain flow.
            split = sections.len();
            let mut acc = 0.0;
            for (k, h) in heights.iter().enumerate() {
                if acc + h > col_h {
                    split = k;
                    break;
                }
                acc += h;
            }
        }
    }
    let mut placed: Vec<(usize, f32, Vec<Item>)> = Vec::new();
    let mut y = top;
    for (k, (items, _)) in sections.into_iter().enumerate() {
        let column = if k < split { 0 } else { 1 };
        if column == 1 && k == split {
            y = top;
        }
        placed.push((column, y, items));
        y += heights[k];
    }
    let two_columns = placed.iter().any(|(c, _, _)| *c == 1);

    for (col, start, items) in placed.iter() {
        let x = r.x + pad + *col as f32 * (col_w + gutter);
        let mut y = *start;
        for item in items.iter() {
            match item.kind {
                0 => {
                    draw_rectangle(x, y + 5.0, 18.0, 1.6, BOOK_GOLD);
                    spaced_left(&item.lines[0], x + 26.0, y + 14.0, 11.5, 1.6, BOOK_WINE);
                }
                2 => {
                    for (i, line) in item.lines.iter().enumerate() {
                        if i == 0 {
                            draw_circle(x + 4.0, y + 7.0, 2.2, BOOK_GOLD);
                        }
                        txt(line, x + 17.0, y + 15.0 + i as f32 * line_h, 16.5, BOOK_INK);
                    }
                }
                _ => {
                    for (i, line) in item.lines.iter().enumerate() {
                        txt(line, x, y + 15.0 + i as f32 * line_h, 16.5, BOOK_INK);
                    }
                }
            }
            y += item.h;
        }
    }

    if two_columns {
        let gx = r.x + pad + col_w + gutter * 0.5;
        draw_line(
            gx,
            top - 16.0,
            gx,
            bottom + 6.0,
            1.0,
            Color { r: BOOK_GOLD.r, g: BOOK_GOLD.g, b: BOOK_GOLD.b, a: 0.35 },
        );
    }

    // Footer: page turning, page dots, close.
    let fy = r.y + h - 64.0;
    draw_line(
        r.x + pad,
        fy - 18.0,
        r.x + w - pad,
        fy - 18.0,
        1.0,
        Color { r: BOOK_GOLD.r, g: BOOK_GOLD.g, b: BOOK_GOLD.b, a: 0.4 },
    );
    if paper_button(
        Rect::new(r.x + pad, fy, 96.0, 36.0),
        i18n::t("< PREV", "< PRÉC."),
        ui.rules_page > 0,
    ) {
        ui.rules_page -= 1;
    }
    if paper_button(
        Rect::new(r.x + pad + 104.0, fy, 96.0, 36.0),
        i18n::t("NEXT >", "SUIV. >"),
        ui.rules_page + 1 < BOOK.len(),
    ) {
        ui.rules_page += 1;
    }
    let n = BOOK.len() as f32;
    for (k, _) in BOOK.iter().enumerate() {
        let x = cx + (k as f32 - (n - 1.0) * 0.5) * 20.0;
        let on = k == ui.rules_page;
        draw_circle(
            x,
            fy + 11.0,
            if on { 4.6 } else { 3.0 },
            if on {
                BOOK_WINE
            } else {
                Color { r: BOOK_GOLD.r, g: BOOK_GOLD.g, b: BOOK_GOLD.b, a: 0.45 }
            },
        );
    }
    centered(
        &format!(
            "{} {} / {}",
            i18n::t("PAGE", "PAGE"),
            ui.rules_page + 1,
            BOOK.len()
        ),
        cx,
        fy + 34.0,
        10.0,
        BOOK_SOFT,
    );
    if paper_button(
        Rect::new(r.x + w - pad - 130.0, fy, 130.0, 36.0),
        i18n::t("CLOSE", "FERMER"),
        true,
    ) {
        ui.rules_open = false;
    }
}

/// A page-turn control printed on the cream paper: wine plate, paper type, and
/// a pale plate when the turn is not available.
fn paper_button(r: Rect, label: &str, enabled: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = enabled && inr(r, mx, my);
    if !enabled {
        round_outline(
            r.x,
            r.y,
            r.w,
            r.h,
            8.0,
            1.0,
            Color { r: 0.62, g: 0.56, b: 0.44, a: 0.32 },
        );
        centered_bold(
            label,
            r.x + r.w * 0.5,
            r.y + r.h * 0.5,
            11.5,
            Color { r: 0.55, g: 0.49, b: 0.38, a: 0.6 },
        );
        return false;
    }
    round_rect(
        r.x,
        r.y,
        r.w,
        r.h,
        8.0,
        if hover {
            Color { r: 0.47, g: 0.185, b: 0.215, a: 1.0 }
        } else {
            BOOK_WINE
        },
    );
    centered_bold(label, r.x + r.w * 0.5, r.y + r.h * 0.5, 11.5, PAPER);
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_pause(g: &mut Game, ui: &mut Ui) {
    let r = modal_rect(560.0, 470.0);
    let body = modal_shell(r, i18n::t("PAUSED", "PAUSE"));
    let gap = 12.0;
    let h = 48.0;
    let mut y = body.y + 10.0;
    if button(
        Rect::new(body.x, y, body.w, h),
        i18n::t("RESUME", "REPRENDRE"),
        true,
        true,
    ) {
        ui.pause_open = false;
    }
    y += h + gap;
    let can_save = g.anim.is_none() && g.trade.is_none() && g.debt.is_none() && g.phase != Phase::Over;
    if button(
        Rect::new(body.x, y, body.w, h),
        i18n::t("SAVE", "SAUVEGARDER"),
        can_save,
        false,
    ) {
        match crate::persist::save(g) {
            Ok(path) => g.log(crate::tr!(
                "Game saved to {}",
                "Partie sauvegardée dans {}",
                path.display()
            )),
            Err(err) => g.log(crate::tr!(
                "Could not save: {}",
                "Impossible de sauvegarder : {}",
                err
            )),
        }
    }
    y += h + gap;
    if button(
        Rect::new(body.x, y, body.w, h),
        i18n::t("RULES OF THE GAME", "RÈGLES DU JEU"),
        true,
        false,
    ) {
        ui.rules_open = true;
        ui.rules_page = 0;
    }
    y += h + gap;
    // Anyone at the table can call the vote, even while a bot is playing.
    let any_human = g.players.iter().any(|player| !player.ai);
    if button(
        Rect::new(body.x, y, body.w, h),
        i18n::t("END THE GAME", "ARRÊTER LA PARTIE"),
        any_human && g.phase != Phase::Over,
        false,
    ) {
        ui.vote_open = true;
        ui.vote_agree = g.players.iter().map(|player| player.ai).collect();
    }
    y += h + gap;
    if button(
        Rect::new(body.x, y, body.w, h),
        i18n::t("MAIN MENU", "MENU PRINCIPAL"),
        true,
        false,
    ) {
        ui.quit_confirm = true;
    }
    y += h + gap + 4.0;
    centered(
        i18n::t(
            "Escape closes this menu",
            "Échap referme ce menu",
        ),
        body.x + body.w * 0.5,
        y + 10.0,
        11.5,
        MUTED,
    );
}

/// Loads the only save straight away, or opens the picker when there is a
/// choice to make.
fn open_saves(ui: &mut Ui) -> Option<Game> {
    let slots = crate::persist::list();
    if slots.is_empty() {
        return None;
    }
    if slots.len() == 1 {
        return crate::persist::load_at(&slots[0].path).ok();
    }
    ui.load_list = slots;
    ui.load_note.clear();
    ui.load_armed = false;
    ui.load_open = true;
    None
}

fn ago_label(secs: u64) -> String {
    if secs < 60 {
        i18n::t("just now", "à l'instant").to_string()
    } else if secs < 3600 {
        crate::tr!("{} min ago", "il y a {} min", secs / 60)
    } else if secs < 86_400 {
        crate::tr!("{} h ago", "il y a {} h", secs / 3600)
    } else {
        crate::tr!("{} d ago", "il y a {} j", secs / 86_400)
    }
}

fn draw_load(app: &mut App) {
    let rows = app.ui.load_list.len().min(8);
    let r = modal_rect(700.0, 176.0 + rows as f32 * 60.0);
    let content = modal_shell(r, i18n::t("LOAD A GAME", "CHARGER UNE PARTIE"));
    // The click that opened the picker belongs to the button behind it.
    let armed = app.ui.load_armed;
    app.ui.load_armed = true;
    let (mx, my) = mouse_position();
    let mut chosen = None;
    let mut y = content.y + 2.0;
    for slot in app.ui.load_list.iter().take(rows) {
        let row = Rect::new(content.x, y, content.w, 52.0);
        let hover = inr(row, mx, my);
        round_card(
            row.x,
            row.y,
            row.w,
            row.h,
            10.0,
            if hover {
                Color { r: 0.145, g: 0.310, b: 0.231, a: 1.0 }
            } else {
                CARD
            },
            if hover { GOLD } else { CARD_EDGE },
        );
        // When it was written, then how long ago that was.
        let stamp = slot.stamp();
        txt(&stamp, row.x + 16.0, row.y + 21.0, 13.0, GOLD_SOFT);
        let age = ago_label(slot.age_secs());
        let age_x = row.x + 26.0 + text_w(&stamp, 13.0);
        txt(&age, age_x, row.y + 21.5, 11.5, CREAM_DIM);
        let seats = crate::tr!(
            "{} players",
            "{} joueurs",
            slot.players
        );
        let seats = crate::tr!("{}", "{}", seats);
        let seats_w = text_w(&seats, 12.0);
        txt(&seats, row.x + row.w - 16.0 - seats_w, row.y + 21.0, 12.0, CREAM_DIM);
        let who = crate::tr!(
            "{} - {}'s turn",
            "{} - tour de {}",
            truncate(&slot.names.join(", "), row.w - 190.0, 12.5),
            slot.current_name()
        );
        txt(&who, row.x + 16.0, row.y + 40.0, 12.5, CREAM);
        if armed && hover && is_mouse_button_pressed(MouseButton::Left) {
            chosen = Some(slot.path.clone());
        }
        y += 60.0;
    }
    if let Some(path) = chosen {
        match crate::persist::load_at(&path) {
            Ok(game) => {
                app.game = Some(game);
                app.ui = Ui::new();
                app.screen = Screen::Play;
                return;
            }
            Err(err) => app.ui.load_note = err,
        }
    }
    if !app.ui.load_note.is_empty() {
        centered(
            &truncate(&app.ui.load_note, content.w, 11.5),
            content.x + content.w * 0.5,
            r.y + r.h - 34.0,
            11.5,
            Color { r: 0.902, g: 0.529, b: 0.443, a: 1.0 },
        );
    }
    if button(
        Rect::new(content.x + content.w - 150.0, r.y + r.h - 60.0, 128.0, 36.0),
        i18n::t("CANCEL", "ANNULER"),
        armed,
        false,
    ) {
        app.ui.load_open = false;
    }
}

fn draw_vote(g: &mut Game, ui: &mut Ui) {
    let r = modal_rect(700.0, 250.0 + 46.0 * g.players.len() as f32);
    let content = modal_shell(r, i18n::t("END THE GAME?", "TERMINER LA PARTIE ?"));
    let intro = wrap(
        i18n::t(
            "The whole table has to agree, and bots always do. The winner is whoever holds the most cash, deeds and buildings.",
            "Toute la table doit être d'accord, et les bots le sont toujours. Le vainqueur est celui qui totalise le plus d'argent, de biens et de constructions.",
        ),
        content.w,
        13.0,
    );
    for (i, line) in intro.iter().enumerate() {
        txt(line, content.x, content.y + 10.0 + i as f32 * 18.0, 13.0, TEXT_DIM);
    }

    let mut y = content.y + 18.0 + intro.len() as f32 * 18.0 + 12.0;
    let mut agreed_all = true;
    for k in 0..g.players.len() {
        let name = g.players[k].name.clone();
        let ai = g.players[k].ai;
        let color = g.players[k].color;
        let worth = g.net_worth(k);
        draw_circle(content.x + 7.0, y - 5.0, 5.0, color);
        txt_bold(&name, content.x + 20.0, y, 14.0, CREAM);
        txt(
            &crate::tr!("worth {}", "patrimoine {}", money(worth)),
            content.x + 20.0,
            y + 16.0,
            11.0,
            MUTED,
        );
        let agreed = ai || ui.vote_agree.get(k).copied().unwrap_or(false);
        if !agreed {
            agreed_all = false;
        }
        if ai {
            txt(
                i18n::t("bot - agrees", "bot - d'accord"),
                content.x + content.w - 132.0,
                y,
                12.5,
                GOLD_SOFT,
            );
        } else {
            let rect = Rect::new(content.x + content.w - 150.0, y - 19.0, 150.0, 32.0);
            let label = if agreed {
                i18n::t("AGREED", "D'ACCORD")
            } else {
                i18n::t("I AGREE", "JE SUIS D'ACCORD")
            };
            if toggle_button(rect, label, agreed) {
                if ui.vote_agree.len() <= k {
                    ui.vote_agree.resize(k + 1, false);
                }
                ui.vote_agree[k] = !agreed;
            }
        }
        y += 44.0;
    }

    let by = r.y + r.h - 64.0;
    if button(
        Rect::new(r.x + 30.0, by, 280.0, 42.0),
        i18n::t("END THE GAME", "TERMINER"),
        agreed_all,
        true,
    ) {
        g.finish_early();
        ui.vote_open = false;
    }
    if button(
        Rect::new(r.x + r.w - 310.0, by, 280.0, 42.0),
        i18n::t("KEEP PLAYING", "CONTINUER"),
        true,
        false,
    ) {
        ui.vote_open = false;
    }
}

fn draw_quit(ui: &mut Ui) -> bool {
    let r = modal_rect(600.0, 280.0);
    let content = modal_shell(r, i18n::t("BACK TO THE MENU", "RETOUR AU MENU"));
    for (i, line) in wrap(
        i18n::t(
            "The game in progress will be lost unless you saved it first.",
            "La partie en cours sera perdue si vous ne l'avez pas sauvegardée.",
        ),
        content.w,
        13.5,
    )
    .iter()
    .enumerate()
    {
        txt(
            line,
            content.x,
            content.y + 14.0 + i as f32 * 20.0,
            13.5,
            CREAM_DIM,
        );
    }
    let y = r.y + r.h - 64.0;
    if button(
        Rect::new(r.x + 32.0, y, 240.0, 40.0),
        i18n::t("LEAVE", "QUITTER"),
        true,
        true,
    ) {
        return true;
    }
    if button(
        Rect::new(r.x + r.w - 272.0, y, 240.0, 40.0),
        i18n::t("KEEP PLAYING", "CONTINUER"),
        true,
        false,
    ) {
        ui.quit_confirm = false;
    }
    false
}
