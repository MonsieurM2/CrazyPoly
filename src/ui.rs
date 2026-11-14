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

const BG: Color = Color { r: 0.065, g: 0.07, b: 0.085, a: 1.0 };
const PANEL_BG: Color = Color { r: 0.115, g: 0.125, b: 0.15, a: 1.0 };
const MODAL_BG: Color = Color { r: 0.145, g: 0.155, b: 0.185, a: 1.0 };
const MODAL_HEADER: Color = Color { r: 0.19, g: 0.20, b: 0.24, a: 1.0 };
const PANEL_EDGE: Color = Color { r: 0.30, g: 0.32, b: 0.37, a: 1.0 };
const TEXT: Color = Color { r: 0.97, g: 0.96, b: 0.93, a: 1.0 };
const TEXT_DIM: Color = Color { r: 0.78, g: 0.79, b: 0.82, a: 1.0 };
const MUTED: Color = Color { r: 0.62, g: 0.64, b: 0.69, a: 1.0 };
const ACCENT: Color = Color { r: 0.88, g: 0.22, b: 0.23, a: 1.0 };
const ACCENT_SOFT: Color = Color { r: 0.97, g: 0.68, b: 0.44, a: 1.0 };
const WARN: Color = Color { r: 0.98, g: 0.76, b: 0.30, a: 1.0 };
const BTN: Color = Color { r: 0.23, g: 0.25, b: 0.30, a: 1.0 };
const BTN_HOVER: Color = Color { r: 0.32, g: 0.35, b: 0.41, a: 1.0 };
const BTN_OFF: Color = Color { r: 0.145, g: 0.155, b: 0.18, a: 1.0 };
const BOARD_BG: Color = Color { r: 0.90, g: 0.885, b: 0.84, a: 1.0 };
const CENTER_BG: Color = Color { r: 0.94, g: 0.925, b: 0.88, a: 1.0 };
const TILE: Color = Color { r: 0.975, g: 0.965, b: 0.935, a: 1.0 };
const INK: Color = Color { r: 0.09, g: 0.09, b: 0.09, a: 1.0 };
const INK_SOFT: Color = Color { r: 0.30, g: 0.29, b: 0.27, a: 1.0 };
const INK_LINE: Color = Color { r: 0.20, g: 0.20, b: 0.19, a: 1.0 };
const ROW_BG: Color = Color { r: 0.18, g: 0.19, b: 0.225, a: 1.0 };
const HOUSE_COL: Color = Color { r: 0.07, g: 0.58, b: 0.26, a: 1.0 };
const HOTEL_COL: Color = Color { r: 0.80, g: 0.12, b: 0.12, a: 1.0 };
const HOVER: Color = Color { r: 1.0, g: 1.0, b: 0.60, a: 0.33 };
const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
const SELECT: Color = Color { r: 0.20, g: 0.40, b: 0.26, a: 1.0 };

// -------------------------------------------------------------------- fonts

thread_local! {
    static FONTS: RefCell<(Option<Font>, Option<Font>)> = RefCell::new((None, None));
}

/// Load the embedded Lato faces once the window exists.
pub fn load_fonts() {
    let regular = load_ttf_font_from_bytes(&include_bytes!("../assets/Lato-Regular.ttf")[..]).ok();
    let bold = load_ttf_font_from_bytes(&include_bytes!("../assets/Lato-Bold.ttf")[..]).ok();
    FONTS.with(|cell| *cell.borrow_mut() = (regular, bold));
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

fn button(r: Rect, label: &str, enabled: bool, primary: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = enabled && inr(r, mx, my);
    let bg = if !enabled {
        BTN_OFF
    } else if primary {
        if hover {
            Color { r: 0.97, g: 0.34, b: 0.32, a: 1.0 }
        } else {
            ACCENT
        }
    } else if hover {
        BTN_HOVER
    } else {
        BTN
    };
    draw_rectangle(r.x, r.y, r.w, r.h, bg);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, PANEL_EDGE);
    let fg = if enabled { TEXT } else { Color { r: 0.45, g: 0.47, b: 0.51, a: 1.0 } };
    let size = (r.h * 0.40).clamp(10.0, 17.0);
    let label = truncate(label, r.w - 10.0, size);
    let dims = measure(&label, size, true);
    draw_raw(
        &label,
        r.x + (r.w - dims.width) * 0.5,
        r.y + r.h * 0.5 + dims.height * 0.34,
        size,
        fg,
        true,
    );
    enabled && hover && is_mouse_button_pressed(MouseButton::Left)
}

fn small_button(r: Rect, label: &str, enabled: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = enabled && inr(r, mx, my);
    draw_rectangle(
        r.x,
        r.y,
        r.w,
        r.h,
        if !enabled {
            BTN_OFF
        } else if hover {
            BTN_HOVER
        } else {
            BTN
        },
    );
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, PANEL_EDGE);
    let size = (r.h * 0.55).clamp(9.0, 15.0);
    centered(label, r.x + r.w * 0.5, r.y + r.h * 0.5, size, if enabled { TEXT } else { MUTED });
    enabled && hover && is_mouse_button_pressed(MouseButton::Left)
}

fn tag(right: f32, cy: f32, label: &str, fg: Color) -> f32 {
    let size = 10.0;
    let w = measure(label, size, true).width + 14.0;
    let h = 17.0;
    draw_rectangle(
        right - w,
        cy - h * 0.5,
        w,
        h,
        Color { r: fg.r * 0.26, g: fg.g * 0.26, b: fg.b * 0.26, a: 1.0 },
    );
    centered_bold(label, right - w * 0.5, cy, size, fg);
    right - w - 6.0
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
        Color { r: 0.02, g: 0.02, b: 0.03, a: 0.68 },
    );
    draw_rectangle(r.x, r.y, r.w, r.h, MODAL_BG);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, PANEL_EDGE);
    draw_rectangle(r.x, r.y, r.w, 46.0, MODAL_HEADER);
    centered_bold(title, r.x + r.w * 0.5, r.y + 23.0, 19.0, TEXT);
    Rect::new(r.x + 20.0, r.y + 58.0, r.w - 40.0, r.h - 76.0)
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
    pub confirm_bankrupt: bool,
}

impl Ui {
    pub fn new() -> Self {
        Ui {
            asset_open: false,
            asset_page: 0,
            confirm_bankrupt: false,
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
    match app.screen {
        Screen::Setup => draw_setup(app),
        Screen::Play => draw_play(app),
    }
}

// ------------------------------------------------------------ setup screen

fn draw_setup(app: &mut App) {
    clear_background(BG);
    let sw = screen_width();
    let sh = screen_height();
    let w = 830.0_f32.min(sw - 32.0);
    let h = (350.0 + app.setup.count as f32 * 54.0).min(sh - 32.0);
    let x = (sw - w) * 0.5;
    let y = (sh - h) * 0.5;
    // While the token picker is open it owns every click.
    let picker_open = app.setup.token_slot.is_some();

    draw_rectangle(x, y, w, h, PANEL_BG);
    draw_rectangle_lines(x, y, w, h, 1.5, PANEL_EDGE);
    draw_rectangle(x, y, w, 106.0, MODAL_HEADER);
    centered_bold("CRAZYPOLY", x + w * 0.5, y + 42.0, 42.0, ACCENT);
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
    if !picker_open && button(Rect::new(x + w - 70.0, y + 14.0, 54.0, 30.0), i18n::code(), true, false) {
        i18n::toggle();
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
        draw_rectangle(row.x, row.y, row.w, row.h, if editing { Color { r: 0.21, g: 0.23, b: 0.27, a: 1.0 } } else { ROW_BG });
        draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, PANEL_EDGE);
        let token_rect = Rect::new(row.x + 8.0, row.y + 2.0, 44.0, row.h - 4.0);
        let (tmx, tmy) = mouse_position();
        if inr(token_rect, tmx, tmy) {
            draw_rectangle(token_rect.x, token_rect.y, token_rect.w, token_rect.h, BTN_HOVER);
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
        draw_rectangle(name_rect.x, name_rect.y, name_rect.w, name_rect.h, BTN_OFF);
        if editing {
            draw_rectangle_lines(name_rect.x, name_rect.y, name_rect.w, name_rect.h, 1.5, ACCENT);
        }
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
        draw_rectangle(
            human_rect.x,
            human_rect.y,
            human_rect.w,
            human_rect.h,
            if human_on {
                ACCENT
            } else if inr(human_rect, mx, my) {
                BTN_HOVER
            } else {
                BTN
            },
        );
        draw_rectangle(
            bot_rect.x,
            bot_rect.y,
            bot_rect.w,
            bot_rect.h,
            if !human_on {
                Color { r: 0.24, g: 0.44, b: 0.68, a: 1.0 }
            } else if inr(bot_rect, mx, my) {
                BTN_HOVER
            } else {
                BTN
            },
        );
        centered_bold(i18n::t("HUMAN", "HUMAIN"), human_rect.x + seg * 0.5, human_rect.y + human_rect.h * 0.5, 12.0, TEXT);
        centered_bold("BOT", bot_rect.x + seg * 0.5, bot_rect.y + bot_rect.h * 0.5, 12.0, TEXT);
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
            i18n::t("START GAME", "NOUVELLE PARTIE"),
            true,
            true,
        )
    {
        app.setup.error.clear();
        app.game = Some(Game::new(app.setup.build_players()));
        app.ui = Ui::new();
        app.screen = Screen::Play;
    }
    let has_save = crate::persist::has_save();
    if !picker_open
        && button(
            Rect::new(x + w * 0.5 + 6.0, start_y, btn_w, 50.0),
            i18n::t("CONTINUE", "REPRENDRE"),
            has_save,
            false,
        )
    {
        match crate::persist::load() {
            Ok(game) => {
                app.setup.error.clear();
                app.game = Some(game);
                app.ui = Ui::new();
                app.screen = Screen::Play;
            }
            Err(err) => app.setup.error = format!("{}: {}", i18n::t("Load failed", "Échec du chargement"), err),
        }
    }
    centered(
        &format!(
            "{} {}",
            i18n::t("Save slot:", "Sauvegarde :"),
            crate::persist::save_path().display()
        ),
        x + w * 0.5,
        y + h - 44.0,
        10.0,
        MUTED,
    );
    centered(
        i18n::t(
            "Click a name to type, a token to change it. Bots play themselves: set every seat to BOT to watch a full game.",
            "Cliquez sur un nom pour le modifier, sur un jeton pour le changer. Les bots jouent seuls : mettez tous les sièges en BOT pour regarder une partie complète.",
        ),
        x + w * 0.5,
        y + h - 24.0,
        11.5,
        TEXT_DIM,
    );

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

struct Layout {
    geo: Geo,
    panel: Rect,
}

fn layout() -> Layout {
    let sw = screen_width();
    let sh = screen_height();
    let m = 16.0;
    let panel_min = 370.0;
    let panel_max = 660.0;
    let board_s = (sh - 2.0 * m).min(sw - panel_min - 3.0 * m).max(360.0);
    let avail = sw - board_s - 3.0 * m;
    let panel_w = avail.clamp(panel_min, panel_max);
    let total = board_s + m + panel_w;
    let x0 = ((sw - total) * 0.5).max(m);
    let y0 = (sh - board_s) * 0.5;
    Layout {
        geo: Geo::new(x0, y0, board_s),
        panel: Rect::new(x0 + board_s + m, y0, panel_w, board_s),
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
    Buy,
    Over,
}

fn active_modal(g: &Game, ui: &Ui) -> Modal {
    if g.phase == Phase::Over {
        return Modal::Over;
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
    if g.phase == Phase::AwaitBuy && !g.players[g.turn].ai {
        return Modal::Buy;
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

    clear_background(BG);
    draw_board(g, &lay, mx, my, input);
    draw_panel(g, ui, &lay, input);

    let mut again = false;
    match modal {
        Modal::None | Modal::Waiting => {}
        Modal::Assets => draw_assets(g, ui, &lay),
        Modal::Trade => draw_trade(g, ui),
        Modal::Auction => draw_auction(g),
        Modal::Debt => draw_debt(g, ui),
        Modal::Buy => draw_buy(g),
        Modal::Over => again = draw_over(g),
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
    draw_rectangle(geo.x - 5.0, geo.y - 5.0, geo.s + 10.0, geo.s + 10.0, Color { r: 0.09, g: 0.09, b: 0.10, a: 1.0 });
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
    if let Some(tile) = hovered {
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

fn draw_rail_icon(r: &Rect, side: Side) {
    let (cx, cy) = match side {
        Side::Bottom => (r.x + r.w * 0.5, r.y + r.h * 0.14),
        Side::Top => (r.x + r.w * 0.5, r.y + r.h * 0.86),
        Side::Left => (r.x + r.w * 0.88, r.y + r.h * 0.5),
        Side::Right => (r.x + r.w * 0.12, r.y + r.h * 0.5),
        Side::Corner => (r.x + r.w * 0.5, r.y + r.h * 0.5),
    };
    let s = (r.w * 0.26).min(19.0);
    draw_rectangle(cx - s * 0.5, cy - s * 0.5, s, s, Color { r: 0.18, g: 0.18, b: 0.18, a: 1.0 });
    draw_line(cx - s * 0.26, cy - s * 0.34, cx - s * 0.26, cy + s * 0.34, 1.4, TILE);
    draw_line(cx + s * 0.26, cy - s * 0.34, cx + s * 0.26, cy + s * 0.34, 1.4, TILE);
    for k in 0..3 {
        let ty = cy - s * 0.24 + k as f32 * s * 0.24;
        draw_line(cx - s * 0.36, ty, cx + s * 0.36, ty, 1.3, TILE);
    }
}

fn draw_utility_icon(r: &Rect, side: Side) {
    let (cx, cy) = match side {
        Side::Bottom => (r.x + r.w * 0.5, r.y + r.h * 0.14),
        Side::Top => (r.x + r.w * 0.5, r.y + r.h * 0.86),
        Side::Left => (r.x + r.w * 0.88, r.y + r.h * 0.5),
        Side::Right => (r.x + r.w * 0.12, r.y + r.h * 0.5),
        Side::Corner => (r.x + r.w * 0.5, r.y + r.h * 0.5),
    };
    let s = (r.w * 0.20).min(15.0);
    draw_circle(cx, cy - s * 0.10, s * 0.46, Color { r: 0.97, g: 0.78, b: 0.16, a: 1.0 });
    draw_triangle(
        vec2(cx - s * 0.34, cy + s * 0.25),
        vec2(cx + s * 0.34, cy + s * 0.25),
        vec2(cx, cy + s * 0.58),
        Color { r: 0.97, g: 0.78, b: 0.16, a: 1.0 },
    );
}

fn draw_corner(space: &Space, r: Rect) {
    let cx = r.x + r.w * 0.5;
    match space.kind {
        Kind::Go => {
            centered_bold("GO", cx, r.y + r.h * 0.29, r.w * 0.25, INK);
            centered(i18n::t("COLLECT", "ENCAISSEZ"), cx, r.y + r.h * 0.50, r.w * 0.095, INK_SOFT);
            centered_bold("200 $", cx, r.y + r.h * 0.585, r.w * 0.115, INK_SOFT);
            let y = r.y + r.h * 0.78;
            draw_line(r.x + r.w * 0.70, y, r.x + r.w * 0.32, y, 3.0, ACCENT);
            draw_triangle(vec2(r.x + r.w * 0.30, y), vec2(r.x + r.w * 0.40, y - 7.0), vec2(r.x + r.w * 0.40, y + 7.0), ACCENT);
        }
        Kind::Jail => {
            centered_bold(i18n::t("JAIL", "PRISON"), cx, r.y + r.h * 0.24, r.w * 0.19, ACCENT);
            centered(i18n::t("JUST", "SIMPLE"), cx, r.y + r.h * 0.60, r.w * 0.085, INK_SOFT);
            centered(i18n::t("VISITING", "VISITE"), cx, r.y + r.h * 0.71, r.w * 0.085, INK_SOFT);
            for k in 0..5 {
                let bx = r.x + r.w * 0.22 + k as f32 * r.w * 0.11;
                draw_line(bx, r.y + r.h * 0.34, bx, r.y + r.h * 0.50, 2.0, INK);
            }
            draw_line(r.x + r.w * 0.20, r.y + r.h * 0.34, r.x + r.w * 0.80, r.y + r.h * 0.34, 2.0, INK);
            draw_line(r.x + r.w * 0.20, r.y + r.h * 0.50, r.x + r.w * 0.80, r.y + r.h * 0.50, 2.0, INK);
        }
        Kind::FreeParking => {
            centered_bold(i18n::t("FREE", "PARC"), cx, r.y + r.h * 0.34, r.w * 0.16, INK);
            centered_bold(i18n::t("PARKING", "GRATUIT"), cx, r.y + r.h * 0.47, r.w * 0.13, INK);
            draw_rectangle(r.x + r.w * 0.30, r.y + r.h * 0.62, r.w * 0.40, r.h * 0.11, ACCENT);
            draw_rectangle(r.x + r.w * 0.38, r.y + r.h * 0.56, r.w * 0.24, r.h * 0.07, ACCENT);
            draw_circle(r.x + r.w * 0.38, r.y + r.h * 0.75, r.w * 0.045, INK);
            draw_circle(r.x + r.w * 0.62, r.y + r.h * 0.75, r.w * 0.045, INK);
        }
        _ => {
            centered_bold(i18n::t("GO TO", "ALLEZ EN"), cx, r.y + r.h * 0.32, r.w * 0.135, INK);
            centered_bold(i18n::t("JAIL", "PRISON"), cx, r.y + r.h * 0.50, r.w * 0.19, ACCENT);
            let y = r.y + r.h * 0.70;
            draw_line(r.x + r.w * 0.26, y, r.x + r.w * 0.72, y, 2.5, INK);
            draw_triangle(vec2(r.x + r.w * 0.76, y), vec2(r.x + r.w * 0.66, y - 6.0), vec2(r.x + r.w * 0.66, y + 6.0), INK);
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
            draw_utility_icon(&r, side);
        } else if space.kind == Kind::Chance || space.kind == Kind::Chest {
            let (cx, cy) = match side {
                Side::Bottom => (r.x + r.w * 0.5, r.y + r.h * 0.12),
                Side::Top => (r.x + r.w * 0.5, r.y + r.h * 0.88),
                Side::Left => (r.x + r.w * 0.88, r.y + r.h * 0.5),
                Side::Right => (r.x + r.w * 0.12, r.y + r.h * 0.5),
                Side::Corner => (r.x + r.w * 0.5, r.y + r.h * 0.5),
            };
            let s = (r.w * 0.30).min(21.0);
            let color = if space.kind == Kind::Chance {
                ACCENT
            } else {
                Color { r: 0.19, g: 0.38, b: 0.68, a: 1.0 }
            };
            draw_rectangle(cx - s * 0.5, cy - s * 0.5, s, s, color);
            centered_bold("?", cx, cy, s * 0.9, WHITE);
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
            centered_bold(&format!("{} $", value), r.x + r.w * 0.5, py - 3.0, r.w * 0.12, INK_SOFT);
        }
    }

    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, INK_LINE);

    if let Some(owner) = g.props[i].owner {
        let color = g.players[owner].color;
        draw_rectangle(r.x, r.y, r.w, r.h, Color { r: color.r, g: color.g, b: color.b, a: 0.12 });
        let (dx, dy) = owner_dot(&r, side);
        draw_circle(dx, dy, 5.5, color);
        draw_circle_lines(dx, dy, 5.5, 1.0, INK_LINE);
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
        let radius = (r.w * 0.175).clamp(9.0, 15.0);
        tokens::draw(player.token, player.color, cx, cy, radius);
    }
}

fn draw_deck_box(top: &str, bottom: &str, x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x + 3.0, y + 4.0, w, h, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.18 });
    draw_rectangle(x, y, w, h, TILE);
    draw_rectangle_lines(x, y, w, h, 2.5, color);
    centered_bold("?", x + w * 0.22, y + h * 0.34, h * 0.34, color);
    centered_bold(top, x + w * 0.60, y + h * 0.34, w * 0.10, color);
    if !bottom.is_empty() {
        centered(bottom, x + w * 0.60, y + h * 0.55, w * 0.075, INK_SOFT);
    }
    centered(i18n::t("PLACE CARD HERE", "POSEZ LA CARTE ICI"), x + w * 0.5, y + h * 0.85, w * 0.055, Color { r: 0.52, g: 0.50, b: 0.46, a: 1.0 });
}

fn draw_center(g: &Game, geo: &Geo) {
    let c = geo.center();
    draw_rectangle(c.x, c.y, c.w, c.h, CENTER_BG);
    let cx = c.x + c.w * 0.5;

    centered_bold("MONOPOLY", cx, c.y + c.h * 0.115, c.w * 0.105, ACCENT);
    centered(i18n::t("CRAZYPOLY EDITION", "ÉDITION CRAZYPOLY"), cx, c.y + c.h * 0.178, c.w * 0.030, Color { r: 0.36, g: 0.34, b: 0.31, a: 1.0 });

    let bw = c.w * 0.30;
    let bh = c.h * 0.155;
    let by = c.y + c.h * 0.245;
    draw_deck_box(
        i18n::t("CHANCE", "CHANCE"),
        i18n::t("GO TO JAIL", "ALLEZ EN PRISON"),
        c.x + c.w * 0.13,
        by,
        bw,
        bh,
        Color { r: 0.80, g: 0.20, b: 0.22, a: 1.0 },
    );
    draw_deck_box(
        i18n::t("COMMUNITY", "CAISSE DE"),
        i18n::t("CHEST", "COMMUNAUTÉ"),
        c.x + c.w * 0.57,
        by,
        bw,
        bh,
        Color { r: 0.18, g: 0.36, b: 0.66, a: 1.0 },
    );

    let dy = c.y + c.h * 0.53;
    let ds = (c.w * 0.105).min(c.h * 0.105);
    crate::dice::draw(g, cx, dy, ds);
    centered(
        &format!("{} + {} = {}", g.dice.0, g.dice.1, g.dice.0 as u16 + g.dice.1 as u16),
        cx,
        dy + ds * 0.82,
        ds * 0.32,
        INK_SOFT,
    );

    let player = &g.players[g.turn];
    let label = crate::tr!("{}'s turn", "Tour de {}", player.name);
    let label_w = measure(&label, c.w * 0.036, false).width;
    draw_circle(cx - label_w * 0.5 - 16.0, c.y + c.h * 0.70, 7.0, player.color);
    centered(&label, cx, c.y + c.h * 0.70, c.w * 0.036, INK);
    centered(&phase_hint(g), cx, c.y + c.h * 0.745, c.w * 0.026, Color { r: 0.26, g: 0.25, b: 0.23, a: 1.0 });

    if let Some((text, _)) = &g.banner {
        let bcard_w = c.w * 0.76;
        let bcard_h = c.h * 0.22;
        let bx = c.x + (c.w - bcard_w) * 0.5;
        let by2 = c.y + c.h - bcard_h - c.h * 0.02;
        draw_rectangle(bx + 3.0, by2 + 4.0, bcard_w, bcard_h, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.20 });
        draw_rectangle(bx, by2, bcard_w, bcard_h, WHITE);
        draw_rectangle_lines(bx, by2, bcard_w, bcard_h, 2.0, ACCENT);
        centered_bold(i18n::t("CARD DRAWN", "CARTE TIRÉE"), bx + bcard_w * 0.5, by2 + bcard_h * 0.18, c.w * 0.024, MUTED);
        let lines = wrap(text, bcard_w - 24.0, c.w * 0.027);
        let lh = c.w * 0.034;
        let start = by2 + bcard_h * 0.38;
        for (k, line) in lines.iter().take(3).enumerate() {
            centered(line, bx + bcard_w * 0.5, start + k as f32 * lh, c.w * 0.027, INK);
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
    let mut lines: Vec<(String, f32, Color)> = Vec::new();
    lines.push((space.name.to_uppercase(), 14.0, TEXT));
    match space.kind {
        Kind::Street => {
            lines.push((
                format!("{} {}", i18n::t("Group", "Groupe"), group_title(space.group)),
                11.5,
                space.group.color(),
            ));
            lines.push((format!("{} {}", i18n::t("Price", "Prix"), money(space.price as i64)), 12.0, TEXT_DIM));
            let h = g.props[tile].houses.min(MAX_HOUSES) as usize;
            lines.push((format!("{} {}", i18n::t("Rent", "Loyer"), money(space.rent[h] as i64)), 12.0, TEXT_DIM));
            lines.push((format!("{} {}", i18n::t("Building", "Construction"), money(space.house_cost as i64)), 12.0, TEXT_DIM));
        }
        Kind::Railroad => {
            lines.push((format!("{} {}", i18n::t("Price", "Prix"), money(space.price as i64)), 12.0, TEXT_DIM));
            lines.push((i18n::t("Rent 25 / 50 / 100 / 200", "Loyer 25 / 50 / 100 / 200").to_string(), 12.0, TEXT_DIM));
        }
        Kind::Utility => {
            lines.push((format!("{} {}", i18n::t("Price", "Prix"), money(space.price as i64)), 12.0, TEXT_DIM));
            lines.push((i18n::t("Rent 4x or 10x the dice", "Loyer 4x ou 10x les dés").to_string(), 12.0, TEXT_DIM));
        }
        Kind::Tax => lines.push((format!("{} {}", i18n::t("Pay", "Payer"), money(space.tax as i64)), 12.0, TEXT_DIM)),
        _ => lines.push((i18n::t("Special space", "Case spéciale").to_string(), 12.0, TEXT_DIM)),
    }
    if let Some(owner) = g.props[tile].owner {
        lines.push((
            format!("{} {}", i18n::t("Owned by", "Propriétaire :"), g.players[owner].name),
            12.0,
            g.players[owner].color,
        ));
        if g.props[tile].mortgaged {
            lines.push((i18n::t("MORTGAGED", "HYPOTHÉQUÉ").to_string(), 12.0, WARN));
        }
    } else if space.purchasable() {
        lines.push((i18n::t("Unowned", "Libre").to_string(), 12.0, MUTED));
    }

    let w = 270.0;
    let pad = 10.0;
    let h: f32 = lines.iter().map(|(_, s, _)| s * 1.55).sum::<f32>() + pad * 2.0;
    let mut x = mx + 18.0;
    let mut y = my + 20.0;
    if x + w > geo.x + geo.s {
        x = mx - w - 16.0;
    }
    if y + h > geo.y + geo.s {
        y = my - h - 16.0;
    }
    draw_rectangle(x, y, w, h, Color { r: 0.08, g: 0.085, b: 0.10, a: 0.97 });
    draw_rectangle_lines(x, y, w, h, 1.0, PANEL_EDGE);
    let mut cy = y + pad + 10.0;
    for (text, size, color) in lines {
        txt(&truncate(&text, w - pad * 2.0, size), x + pad, cy, size, color);
        cy += size * 1.55;
    }
}

// -------------------------------------------------------------------- panel

fn draw_panel(g: &mut Game, ui: &mut Ui, lay: &Layout, input: bool) {
    let p = lay.panel;
    draw_rectangle(p.x, p.y, p.w, p.h, PANEL_BG);
    draw_rectangle_lines(p.x, p.y, p.w, p.h, 1.0, PANEL_EDGE);
    draw_rectangle(p.x, p.y, p.w, 66.0, MODAL_HEADER);
    txt_bold("CRAZYPOLY", p.x + 18.0, p.y + 30.0, 21.0, ACCENT);
    txt(
        &crate::tr!(
            "{} players - {} solvent",
            "{} joueurs - {} solvables",
            g.players.len(),
            g.active_count()
        ),
        p.x + 18.0,
        p.y + 51.0,
        11.0,
        TEXT_DIM,
    );
    text_r(
        &crate::tr!(
            "BANK {} H / {} T",
            "BANQUE {} M / {} H",
            g.houses_left,
            g.hotels_left
        ),
        p.x + p.w - 78.0,
        p.y + 51.0,
        10.5,
        MUTED,
    );
    if button(Rect::new(p.x + p.w - 62.0, p.y + 14.0, 46.0, 26.0), i18n::code(), true, false) {
        i18n::toggle();
    }

    let n = g.players.len();
    let rows_top = p.y + 78.0;
    let reserved_log = 250.0;
    let actions_h = 130.0;
    let avail = (p.h - 78.0 - actions_h - reserved_log).max(120.0);
    let row_h = (avail / n as f32).clamp(36.0, 76.0);
    for i in 0..n {
        let r = Rect::new(p.x + 12.0, rows_top + i as f32 * row_h, p.w - 24.0, row_h - 5.0);
        draw_player_row(g, i, r);
    }

    let actions_top = rows_top + n as f32 * row_h + 8.0;
    let cur_name = g.players[g.turn].name.clone();
    let cur_cash = g.players[g.turn].cash;
    let cur_color = g.players[g.turn].color;
    let cur_ai = g.players[g.turn].ai;
    let cur_in_jail = g.players[g.turn].in_jail;
    let cur_cards = g.players[g.turn].jail_cards;
    draw_circle(p.x + 21.0, actions_top + 8.0, 6.0, cur_color);
    let turn_label = if cur_ai {
        crate::tr!("{} (BOT) is playing", "{} (BOT) joue", cur_name)
    } else {
        format!("{} - {}", cur_name, money(cur_cash))
    };
    txt_bold(&truncate(&turn_label, p.w - 44.0, 14.0), p.x + 32.0, actions_top + 13.0, 14.0, TEXT);
    txt(
        &truncate(&phase_hint(g), p.w - 36.0, 11.5),
        p.x + 16.0,
        actions_top + 32.0,
        11.5,
        TEXT_DIM,
    );

    let human = !cur_ai;
    let inner = p.w - 36.0;
    let bw = (inner - 16.0) / 3.0;
    let bh = 40.0;
    let row1 = actions_top + 44.0;
    let row2 = row1 + bh + 8.0;

    let roll_label = if cur_in_jail && g.phase == Phase::Roll {
        i18n::t("ROLL FOR DOUBLES", "TENTER UN DOUBLE")
    } else if g.rolled_double {
        i18n::t("ROLL AGAIN", "RELANCER")
    } else {
        i18n::t("ROLL DICE", "LANCER LES DÉS")
    };
    if button(
        Rect::new(p.x + 18.0, row1, bw, bh),
        roll_label,
        input && human && g.phase == Phase::Roll,
        true,
    ) {
        g.roll();
    }

    let context = Rect::new(p.x + 18.0 + bw + 8.0, row1, bw, bh);
    if cur_in_jail && g.phase == Phase::Roll {
        if cur_cards > 0 {
            if button(context, i18n::t("USE JAIL CARD", "CARTE PRISON"), input, false) {
                g.use_jail_card();
            }
        } else if button(
            context,
            &crate::tr!("PAY {}", "PAYER {}", money(crate::game::JAIL_FINE)),
            input && cur_cash >= crate::game::JAIL_FINE,
            false,
        ) {
            g.pay_jail_fine();
        }
    } else if g.phase == Phase::Debt {
        if button(
            context,
            &crate::tr!("PAY {}", "PAYER {}", money(g.debt_amount())),
            input && g.can_afford_debt(),
            false,
        ) {
            g.pay_debt();
        }
    } else {
        button(context, " ", false, false);
    }

    if button(
        Rect::new(p.x + 18.0 + 2.0 * (bw + 8.0), row1, bw, bh),
        i18n::t("END TURN", "FIN DU TOUR"),
        input && human && g.phase == Phase::Manage,
        false,
    ) {
        g.end_turn();
    }

    if button(
        Rect::new(p.x + 18.0, row2, bw, bh),
        i18n::t("ASSETS", "MES BIENS"),
        input && human && g.can_manage(),
        false,
    ) {
        ui.asset_open = true;
        ui.asset_page = 0;
    }
    if button(
        Rect::new(p.x + 18.0 + bw + 8.0, row2, bw, bh),
        i18n::t("TRADE", "ÉCHANGE"),
        input && human && g.can_manage() && g.active_count() > 1,
        false,
    ) {
        g.open_trade();
    }
    let can_save = input
        && human
        && matches!(g.phase, Phase::Roll | Phase::Manage)
        && g.anim.is_none()
        && g.trade.is_none()
        && g.debt.is_none();
    if button(
        Rect::new(p.x + 18.0 + 2.0 * (bw + 8.0), row2, bw, bh),
        i18n::t("SAVE", "SAUVER"),
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
                "Sauvegarde impossible : {}",
                err
            )),
        }
    }

    let log_top = row2 + bh + 22.0;
    draw_line(p.x + 12.0, log_top - 6.0, p.x + p.w - 12.0, log_top - 6.0, 1.0, PANEL_EDGE);
    txt_bold(i18n::t("GAME LOG", "JOURNAL DE PARTIE"), p.x + 18.0, log_top + 12.0, 10.5, MUTED);
    let size = 11.5;
    let line_h = 14.5;
    let available = (p.y + p.h - 14.0) - (log_top + 22.0);
    let max_lines = (available / line_h).max(1.0) as usize;
    let mut lines: Vec<String> = Vec::new();
    for msg in g.log.iter().rev() {
        let wrapped = wrap(msg, p.w - 36.0, size);
        for line in wrapped.into_iter().rev() {
            lines.push(line);
            if lines.len() >= max_lines {
                break;
            }
        }
        if lines.len() >= max_lines {
            break;
        }
    }
    lines.reverse();
    for (k, line) in lines.iter().enumerate() {
        let color = if line.starts_with("--") { ACCENT_SOFT } else { TEXT_DIM };
        txt(line, p.x + 18.0, log_top + 30.0 + k as f32 * line_h, size, color);
    }
}

fn draw_player_row(g: &Game, index: usize, r: Rect) {
    let player = &g.players[index];
    let is_current = index == g.turn && g.phase != Phase::Over;
    let bg = if player.bankrupt {
        Color { r: 0.105, g: 0.11, b: 0.12, a: 1.0 }
    } else if is_current {
        Color { r: 0.21, g: 0.225, b: 0.27, a: 1.0 }
    } else {
        Color { r: 0.15, g: 0.16, b: 0.185, a: 1.0 }
    };
    draw_rectangle(r.x, r.y, r.w, r.h, bg);
    if is_current {
        draw_rectangle(r.x, r.y, 3.0, r.h, player.color);
    }
    tokens::draw(
        player.token,
        if player.bankrupt { MUTED } else { player.color },
        r.x + 23.0,
        r.y + r.h * 0.5,
        12.0,
    );

    txt_bold(
        &truncate(&player.name, r.w - 170.0, 15.0),
        r.x + 44.0,
        r.y + r.h * 0.42,
        15.0,
        if player.bankrupt { MUTED } else { TEXT },
    );
    let props = g.owned_tiles(index).len();
    let sub = format!(
        "{}   {} {}",
        money(player.cash),
        props,
        i18n::t("lots", "biens")
    );
    txt(&sub, r.x + 44.0, r.y + r.h * 0.74, 11.0, if player.bankrupt { MUTED } else { TEXT_DIM });

    let mut right = r.x + r.w - 8.0;
    if player.bankrupt {
        tag(right, r.y + r.h * 0.5, i18n::t("BANKRUPT", "FAILLITE"), MUTED);
    } else {
        if player.ai {
            right = tag(right, r.y + r.h * 0.5, "BOT", Color { r: 0.58, g: 0.74, b: 0.98, a: 1.0 });
        }
        if player.in_jail {
            right = tag(right, r.y + r.h * 0.5, i18n::t("JAIL", "PRISON"), WARN);
        }
        if player.jail_cards > 0 {
            tag(right, r.y + r.h * 0.5, i18n::t("CARD", "CARTE"), Color { r: 0.48, g: 0.88, b: 0.58, a: 1.0 });
        }
    }
}

// ------------------------------------------------------------------- modals

fn draw_buy(g: &mut Game) {
    let pos = g.players[g.turn].pos;
    let name = g.spaces[pos].name;
    let price = g.spaces[pos].price as i64;
    let rent = g.spaces[pos].rent[0] as i64;
    let house = g.spaces[pos].house_cost as i64;
    let kind = g.spaces[pos].kind;
    let group = g.spaces[pos].group;
    let cash = g.players[g.turn].cash;

    let r = modal_rect(500.0, 330.0);
    let content = modal_shell(r, i18n::t("UNOWNED PROPERTY", "PROPRIÉTÉ LIBRE"));
    draw_rectangle(content.x, content.y, content.w, 9.0, if kind == Kind::Street { group.color() } else { MUTED });
    centered_bold(name, r.x + r.w * 0.5, content.y + 38.0, 23.0, TEXT);
    centered_bold(&format!("{} {}", i18n::t("Price", "Prix"), money(price)), r.x + r.w * 0.5, content.y + 72.0, 18.0, ACCENT);
    if kind == Kind::Street {
        txt(&format!("{} {}", i18n::t("Base rent", "Loyer de base"), money(rent)), r.x + 34.0, content.y + 108.0, 13.0, TEXT_DIM);
        txt(&format!("{} {}", i18n::t("Building", "Construction"), money(house)), r.x + 258.0, content.y + 108.0, 13.0, TEXT_DIM);
        txt(
            &format!("{} {}", i18n::t("Group", "Groupe"), group_title(group)),
            r.x + 34.0,
            content.y + 130.0,
            13.0,
            TEXT_DIM,
        );
    } else if kind == Kind::Railroad {
        txt(i18n::t("Rent 25 / 50 / 100 / 200", "Loyer 25 / 50 / 100 / 200"), r.x + 34.0, content.y + 108.0, 13.0, TEXT_DIM);
        txt(i18n::t("Depends on how many you own", "Selon le nombre que vous possédez"), r.x + 34.0, content.y + 130.0, 12.0, MUTED);
    } else {
        txt(i18n::t("Rent is 4x or 10x the dice", "Loyer de 4x ou 10x les dés"), r.x + 34.0, content.y + 108.0, 13.0, TEXT_DIM);
    }
    txt(
        &format!("{} {}", i18n::t("Your cash:", "Votre argent :"), money(cash)),
        r.x + 34.0,
        content.y + 168.0,
        13.0,
        MUTED,
    );

    let by = r.y + r.h - 64.0;
    if button(
        Rect::new(r.x + 26.0, by, 216.0, 44.0),
        &crate::tr!("BUY {}", "ACHETER {}", money(price)),
        cash >= price,
        true,
    ) {
        g.buy();
    }
    if button(Rect::new(r.x + 258.0, by, 216.0, 44.0), i18n::t("AUCTION IT", "METTRE AUX ENCHÈRES"), true, false) {
        g.start_auction_current();
    }
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

    let r = modal_rect(820.0, 660.0);
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
            list_top + 140.0,
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
            "+ build   - sell   M mortgage   U unmortgage",
            "+ construire   - vendre   M hypothéquer   U lever",
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

