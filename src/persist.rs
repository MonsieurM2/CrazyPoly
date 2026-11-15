//! Save game persistence.
//!
//! A game can be saved whenever it is at a safe point (the current player is
//! waiting to roll, or has finished their move). The whole state needed to
//! resume - players, deeds, buildings, bank supply, whose turn it is and both
//! card decks in their current order - is written as JSON to a single slot in
//! the user's application data directory.

use std::fs;
use std::path::PathBuf;

use macroquad::prelude::Color;
use serde::{Deserialize, Serialize};

use crate::cards::{self, Card, Deck};
use crate::game::{Game, Phase, Player, Prop, PLAYER_COLORS};

const SAVE_VERSION: u32 = 1;
const SAVE_FILE: &str = "savegame.json";

#[derive(Serialize, Deserialize)]
struct SavedPlayer {
    name: String,
    color: usize,
    cash: i64,
    pos: usize,
    in_jail: bool,
    jail_turns: u8,
    jail_cards: u8,
    bankrupt: bool,
    ai: bool,
    #[serde(default)]
    token: usize,
    laps: u32,
}

#[derive(Serialize, Deserialize)]
struct SavedProp {
    owner: Option<usize>,
    houses: u8,
    mortgaged: bool,
}

#[derive(Serialize, Deserialize)]
struct SavedGame {
    version: u32,
    players: Vec<SavedPlayer>,
    props: Vec<SavedProp>,
    turn: usize,
    /// True when the saved player still has to press "end turn".
    managing: bool,
    dice: (u8, u8),
    rolled_double: bool,
    double_count: u8,
    houses_left: u32,
    hotels_left: u32,
    chance: Vec<usize>,
    chance_idx: usize,
    chest: Vec<usize>,
    chest_idx: usize,
    log: Vec<String>,
}

fn color_index(color: Color) -> usize {
    PLAYER_COLORS
        .iter()
        .position(|c| {
            (c.r - color.r).abs() < 0.002
                && (c.g - color.g).abs() < 0.002
                && (c.b - color.b).abs() < 0.002
        })
        .unwrap_or(0)
}

fn card_index(chance: bool, card: &Card) -> usize {
    let list = if chance {
        cards::chance_cards()
    } else {
        cards::chest_cards()
    };
    list.iter().position(|c| c.en == card.en).unwrap_or(0)
}

fn deck_from_indices(
    label: &'static str,
    chance: bool,
    order: &[usize],
    idx: usize,
) -> Deck {
    let canonical = if chance {
        cards::chance_cards()
    } else {
        cards::chest_cards()
    };
    let mut cards: Vec<Card> = Vec::new();
    for i in order {
        if let Some(card) = canonical.get(*i) {
            cards.push(*card);
        }
    }
    let cards = if cards.is_empty() { canonical } else { cards };
    Deck::from_cards(label, cards, idx)
}

impl SavedGame {
    fn from_game(g: &Game) -> Result<SavedGame, String> {
        if !matches!(g.phase, Phase::Roll | Phase::Manage) {
            return Err(crate::i18n::t(
                "A game can only be saved at the start or end of a turn.",
                "Une partie ne peut être sauvegardée qu'en début ou en fin de tour.",
            )
            .to_string());
        }
        let players = g
            .players
            .iter()
            .map(|p| SavedPlayer {
                name: p.name.clone(),
                color: color_index(p.color),
                cash: p.cash,
                pos: p.pos,
                in_jail: p.in_jail,
                jail_turns: p.jail_turns,
                jail_cards: p.jail_cards,
                bankrupt: p.bankrupt,
                ai: p.ai,
                token: p.token,
                laps: p.laps,
            })
            .collect();
        let props = g
            .props
            .iter()
            .map(|p| SavedProp {
                owner: p.owner,
                houses: p.houses,
                mortgaged: p.mortgaged,
            })
            .collect();
        let chance = g
            .chance
            .cards
            .iter()
            .map(|c| card_index(true, c))
            .collect();
        let chest = g.chest.cards.iter().map(|c| card_index(false, c)).collect();
        let log: Vec<String> = g.log.iter().rev().take(40).rev().cloned().collect();

        Ok(SavedGame {
            version: SAVE_VERSION,
            players,
            props,
            turn: g.turn,
            managing: g.phase == Phase::Manage,
            dice: g.dice,
            rolled_double: g.rolled_double,
            double_count: g.double_count,
            houses_left: g.houses_left,
            hotels_left: g.hotels_left,
            chance,
            chance_idx: g.chance.idx,
            chest,
            chest_idx: g.chest.idx,
            log,
        })
    }

    fn into_game(self) -> Result<Game, String> {
        if self.version != SAVE_VERSION {
            return Err(crate::tr!(
                "Unsupported save version {} (this build reads version {}).",
                "Version de sauvegarde {} non prise en charge (cette version lit la version {}).",
                self.version,
                SAVE_VERSION
            ));
        }
        if self.players.len() < 2 || self.players.len() > 8 {
            return Err(crate::i18n::t(
                "Corrupt save: invalid player count.",
                "Sauvegarde corrompue : nombre de joueurs invalide.",
            )
            .to_string());
        }
        if self.props.len() != 40 {
            return Err(crate::i18n::t(
                "Corrupt save: expected 40 spaces.",
                "Sauvegarde corrompue : 40 cases attendues.",
            )
            .to_string());
        }
        if self.turn >= self.players.len() {
            return Err(crate::i18n::t(
                "Corrupt save: invalid current player.",
                "Sauvegarde corrompue : joueur courant invalide.",
            )
            .to_string());
        }

        let mut players = Vec::with_capacity(self.players.len());
        for (i, sp) in self.players.iter().enumerate() {
            if sp.pos >= 40 {
                return Err(crate::tr!(
                    "Corrupt save: player {} has an invalid position.",
                    "Sauvegarde corrompue : la position du joueur {} est invalide.",
                    i
                ));
            }
            let color = PLAYER_COLORS[sp.color % PLAYER_COLORS.len()];
            let mut player = Player::new(sp.name.clone(), color, sp.ai);
            player.cash = sp.cash;
            player.pos = sp.pos;
            player.in_jail = sp.in_jail;
            player.jail_turns = sp.jail_turns;
            player.jail_cards = sp.jail_cards;
            player.bankrupt = sp.bankrupt;
            player.token = sp.token % crate::tokens::COUNT;
            player.laps = sp.laps;
            players.push(player);
        }

        let mut props = Vec::with_capacity(40);
        for sp in &self.props {
            if sp.houses > 5 {
                return Err(crate::i18n::t(
                    "Corrupt save: invalid building count.",
                    "Sauvegarde corrompue : nombre de constructions invalide.",
                )
                .to_string());
            }
            if let Some(owner) = sp.owner {
                if owner >= players.len() {
                    return Err(crate::i18n::t(
                        "Corrupt save: unknown property owner.",
                        "Sauvegarde corrompue : propriétaire inconnu.",
                    )
                    .to_string());
                }
            }
            props.push(Prop {
                owner: sp.owner,
                houses: sp.houses,
                mortgaged: sp.mortgaged,
            });
        }

        let mut game = Game::new(players);
        game.props = props;
        game.turn = self.turn;
        game.phase = if self.managing {
            Phase::Manage
        } else {
            Phase::Roll
        };
        game.dice = self.dice;
        game.rolled_double = self.rolled_double;
        game.double_count = self.double_count;
        game.houses_left = self.houses_left;
        game.hotels_left = self.hotels_left;
        game.chance = deck_from_indices("CHANCE", true, &self.chance, self.chance_idx);
        game.chest = deck_from_indices("COMMUNITY CHEST", false, &self.chest, self.chest_idx);
        game.log = self.log;
        if game.log.len() > 400 {
            let cut = game.log.len() - 400;
            game.log.drain(..cut);
        }
        let name = game.current_name();
        game.log(crate::tr!(
            "Loaded saved game - {}'s turn.",
            "Partie sauvegardée chargée - tour de {}.",
            name
        ));
        Ok(game)
    }
}

/// Folder holding the save slot. Uses the platform's application data folder
/// so the game works no matter where the executable lives.
pub fn save_dir() -> PathBuf {
    if cfg!(windows) {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.is_empty() {
                return PathBuf::from(appdata).join("Crazyopoly");
            }
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("Crazyopoly");
    }
    PathBuf::from(".")
}

pub fn save_path() -> PathBuf {
    // CRAZYOPOLY_SAVE lets you keep the save file somewhere else (portable
    // installs, tests). Otherwise the platform data folder is used.
    if let Ok(custom) = std::env::var("CRAZYOPOLY_SAVE") {
        if !custom.trim().is_empty() {
            return PathBuf::from(custom);
        }
    }
    save_dir().join(SAVE_FILE)
}

pub fn has_save() -> bool {
    save_path().is_file()
}

pub fn save(game: &Game) -> Result<PathBuf, String> {
    let data = SavedGame::from_game(game)?;
    let path = save_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    }
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("{}: {}", path.display(), e))?;
    Ok(path)
}

pub fn load() -> Result<Game, String> {
    let path = save_path();
    let text =
        fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let data: SavedGame =
        serde_json::from_str(&text).map_err(|e| {
            format!("{}: {}", crate::i18n::t("Corrupt save", "Sauvegarde corrompue"), e)
        })?;
    data.into_game()
}

