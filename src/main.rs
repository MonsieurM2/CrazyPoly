// Crazyopoly - a complete, self-contained recreation of the classic Monopoly
// board game, written in Rust with the macroquad 2D framework.
//
// Base repository: https://github.com/MonsieurM2/CrazyPoly
//
// The project is split into small focused modules:
//   board.rs  - the 40 official spaces, colour groups, prices and rents
//   cards.rs  - the Chance and Community Chest decks and their effects
//   game.rs   - the rules engine: turns, movement, rent, auctions, debt,
//               buildings, trading and bankruptcy
//   ai.rs     - bot players (valuation, buying, bidding, cash raising, trades)
//   dice.rs   - replaying the recorded dice throw, seen from straight above
//   dice_physics.rs - the dice throw, simulated with the rapier3d engine
//   ui.rs     - top-down board rendering, side panel and modal dialogs
//
// Run it with:  cargo run --release
//
// Controls: click the on-screen buttons. Names on the setup screen are typed
// with the keyboard; Escape is not used so that accidental presses never
// destroy a game in progress.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod audio;
mod board;
mod cards;
mod dice;
mod dice_physics;
mod game;
mod i18n;
mod persist;
mod tokens;
mod ui;

use macroquad::prelude::*;

use game::Game;
use ui::{Setup, Ui};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Setup,
    Play,
}

pub struct App {
    pub screen: Screen,
    pub setup: Setup,
    pub ui: Ui,
    pub game: Option<Game>,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Crazyopoly".to_owned(),
        window_width: 1440,
        window_height: 920,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    ui::load_fonts();
    audio::load().await;
    macroquad::rand::srand(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5EED_5EED),
    );

    let mut app = App {
        screen: Screen::Menu,
        setup: Setup::default(),
        ui: Ui::new(),
        game: None,
    };

    loop {
        // Clamp dt so dragging the window never teleports tokens.
        let dt = get_frame_time().min(0.05);
        if let Some(game) = app.game.as_mut() {
            game.update(dt);
        }
        ui::frame(&mut app);
        next_frame().await;
    }
}
