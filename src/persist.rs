//! Save game persistence.
//!
//! A game can be saved whenever it is at a safe point (the current player is
//! waiting to roll, or has finished their move). The whole state needed to
//! resume - players, deeds, buildings, bank supply, whose turn it is and both
//! card decks in their current order - is written as JSON to a single slot in
//! the user's application data directory.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use macroquad::prelude::Color;
use serde::{Deserialize, Serialize};

use crate::cards::{self, Card, Deck};
use crate::game::{Game, Phase, Player, Prop, PLAYER_COLORS};
use crate::i18n;

const SAVE_VERSION: u32 = 1;
const SAVE_FILE: &str = "savegame.json";
const SAVE_PREFIX: &str = "save-";
const MAX_SAVES: usize = 8;

/// The few fields the picker reads, so listing a save does not depend on the
/// rest of the format.
#[derive(Deserialize)]
struct SummaryPlayer {
    name: String,
}

#[derive(Deserialize)]
struct SaveSummary {
    players: Vec<SummaryPlayer>,
    turn: usize,
}

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
    #[serde(default)]
    strategy: crate::ai::Strategy,
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
                strategy: p.strategy,
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
            player.strategy = sp.strategy;
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

/// Folder holding this player's data. Uses the platform's application data
/// folder so the game works no matter where the executable lives.
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

/// Folder holding the save slots. `CRAZYOPOLY_SAVE` points it somewhere else
/// (portable installs, tests); a path ending in `.json` names the folder after
/// that file, so an old single-save path keeps working.
pub fn saves_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("CRAZYOPOLY_SAVE") {
        if !custom.trim().is_empty() {
            let path = PathBuf::from(custom);
            let is_json = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("json"))
                .unwrap_or(false);
            return if is_json {
                path.with_extension("")
            } else {
                path
            };
        }
    }
    save_dir().join("saves")
}

/// The one-file layout of earlier versions: folded into the slot folder the
/// first time the game looks at its saves.
fn migrate_legacy() {
    let legacy = save_dir().join(SAVE_FILE);
    if !legacy.is_file() {
        return;
    }
    let dir = saves_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let _ = fs::rename(&legacy, dir.join(format!("{SAVE_PREFIX}{stamp}.json")));
}

fn stamp_of(path: &PathBuf) -> u64 {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix(SAVE_PREFIX))
        .and_then(|n| n.strip_suffix(".json"))
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or(0)
}

/// One save on disk, with what the picker needs to describe it.
pub struct SaveInfo {
    pub path: PathBuf,
    /// Milliseconds since the epoch, taken from the slot name.
    pub saved_at: u64,
    pub players: usize,
    pub turn: usize,
    pub names: Vec<String>,
}

impl SaveInfo {
    pub fn current_name(&self) -> String {
        self.names.get(self.turn).cloned().unwrap_or_default()
    }

    /// Local date and time of the save, ready to print in the picker.
    pub fn stamp(&self) -> String {
        stamp(self.saved_at)
    }

    /// Whole seconds, for a "three hours ago" style label.
    pub fn age_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        now.saturating_sub(self.saved_at) / 1000
    }
}

/// Local "2026-11-15 · 13:24" stamp of a slot, written when it was saved.
pub fn stamp(saved_at: u64) -> String {
    use chrono::{Local, TimeZone};
    let secs = (saved_at / 1000) as i64;
    let Some(dt) = Local.timestamp_opt(secs, 0).single() else {
        return String::new();
    };
    let pattern = i18n::t("%b %d, %Y · %H:%M", "%d/%m/%Y · %H:%M");
    dt.format(pattern).to_string()
}

/// Every save on disk, newest first.
pub fn list() -> Vec<SaveInfo> {
    migrate_legacy();
    let dir = saves_dir();
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_json = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if !is_json {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(summary) = serde_json::from_str::<SaveSummary>(&text) else {
            continue;
        };
        if summary.players.is_empty() || summary.players.len() > 8 {
            continue;
        }
        out.push(SaveInfo {
            saved_at: stamp_of(&path),
            players: summary.players.len(),
            turn: summary.turn.min(summary.players.len() - 1),
            names: summary.players.into_iter().map(|p| p.name).collect(),
            path,
        });
    }
    out.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
    out
}

pub fn has_saves() -> bool {
    !list().is_empty()
}

/// Writes the game to a fresh slot and keeps the newest `MAX_SAVES`.
pub fn save(game: &Game) -> Result<PathBuf, String> {
    let data = SavedGame::from_game(game)?;
    migrate_legacy();
    let dir = saves_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    let mut stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let mut path = dir.join(format!("{SAVE_PREFIX}{stamp}.json"));
    // Two saves in the same millisecond are unlikely, not impossible.
    while path.exists() {
        stamp += 1;
        path = dir.join(format!("{SAVE_PREFIX}{stamp}.json"));
    }
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("{}: {}", path.display(), e))?;

    let mut slots = list();
    if slots.len() > MAX_SAVES {
        for old in slots.split_off(MAX_SAVES) {
            let _ = fs::remove_file(old.path);
        }
    }
    Ok(path)
}

pub fn load_at(path: &std::path::Path) -> Result<Game, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let data: SavedGame = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{}: {}",
            crate::i18n::t("Corrupt save", "Sauvegarde corrompue"),
            e
        )
    })?;
    data.into_game()
}

