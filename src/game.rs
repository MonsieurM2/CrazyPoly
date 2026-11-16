//! Core game engine: turn order, movement, rent, cards, auctions, building,
//! trading, debt and bankruptcy.
//!
//! The engine is UI-agnostic: `ui` only calls public methods and reads state.
//! A single `Game` value owns all mutable state and exposes a `Phase`
//! state machine so both human input and the AI driver know what is legal.

use macroquad::prelude::*;

use crate::ai;
use crate::ai::Strategy;
use crate::board::{self, Group, Kind, Space};
use crate::cards::{self, Card, Deck, Effect};
use crate::dice_physics::Throw;
use crate::i18n;

pub const GO_SALARY: i64 = 200;
pub const JAIL_POS: usize = 10;
pub const JAIL_FINE: i64 = 50;
pub const START_CASH: i64 = 1500;
pub const START_HOUSES: u32 = 32;
pub const START_HOTELS: u32 = 12;
pub const MAX_HOUSES: u8 = 5;

/// Token colours for up to eight players. The save format stores the index.
pub const PLAYER_COLORS: [Color; 8] = [
    Color { r: 0.87, g: 0.22, b: 0.22, a: 1.0 },
    Color { r: 0.20, g: 0.45, b: 0.84, a: 1.0 },
    Color { r: 0.18, g: 0.66, b: 0.36, a: 1.0 },
    Color { r: 0.96, g: 0.79, b: 0.16, a: 1.0 },
    Color { r: 0.60, g: 0.35, b: 0.78, a: 1.0 },
    Color { r: 0.96, g: 0.55, b: 0.16, a: 1.0 },
    Color { r: 0.20, g: 0.73, b: 0.77, a: 1.0 },
    Color { r: 0.93, g: 0.46, b: 0.73, a: 1.0 },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Waiting for the current player to roll.
    Roll,
    /// A token is walking around the board; all input is blocked.
    Moving,
    /// Landed on an unowned property: buy it or send it to auction.
    AwaitBuy,
    /// An auction is in progress (`auction.turn` is the player to act).
    Auction,
    /// The current player owes more cash than they hold.
    Debt,
    /// Free to build, mortgage, trade, then end the turn.
    Manage,
    /// Only one solvent player remains.
    Over,
}

/// Rent modifier applied by Chance cards.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LandingMod {
    Normal,
    UtilityDice,
    RailroadDouble,
}

/// What to do once a debt has been settled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resume {
    FinishTurn,
    /// $50 jail fine paid before rolling: the player may still roll.
    JailRoll,
    /// Third failed jail roll paid: the player now moves the rolled total.
    JailMove,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TradeState {
    Draft,
    Proposed,
}

#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub color: Color,
    pub cash: i64,
    pub pos: usize,
    pub in_jail: bool,
    pub jail_turns: u8,
    pub jail_cards: u8,
    pub bankrupt: bool,
    pub ai: bool,
    /// Token design index (see `tokens::COUNT`).
    pub token: usize,
    /// Temperament of a bot seat (invisible unless the debug mode is on).
    pub strategy: Strategy,
    /// Number of complete laps, shown in the UI.
    pub laps: u32,
}

impl Player {
    pub fn new(name: impl Into<String>, color: Color, ai: bool) -> Self {
        Player {
            name: name.into(),
            color,
            cash: START_CASH,
            pos: 0,
            in_jail: false,
            jail_turns: 0,
            jail_cards: 0,
            bankrupt: false,
            ai,
            token: 0,
            strategy: Strategy::default(),
            laps: 0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Prop {
    pub owner: Option<usize>,
    pub houses: u8,
    pub mortgaged: bool,
}

pub struct Auction {
    pub tile: usize,
    pub bid: i64,
    pub high: Option<usize>,
    pub active: Vec<bool>,
    pub turn: usize,
}

impl Auction {
    pub fn active_count(&self) -> usize {
        self.active.iter().filter(|a| **a).count()
    }
}

#[derive(Clone)]
pub struct Trade {
    /// Who proposed the deal.
    pub from: usize,
    /// Who has to answer it.
    pub to: usize,
    /// Properties `from` hands to `to`.
    pub give: Vec<usize>,
    /// Properties `to` hands to `from`.
    pub get: Vec<usize>,
    /// Cash `from` pays `to`.
    pub give_cash: i64,
    /// Cash `to` pays `from`.
    pub get_cash: i64,
    pub state: TradeState,
    pub notice: String,
}

pub struct MoveAnim {
    pub player: usize,
    pub path: Vec<usize>,
    pub idx: usize,
    pub t: f32,
}

pub struct Debt {
    pub creditor: Option<usize>,
    pub amount: i64,
    pub resume: Resume,
}

pub struct Game {
    /// The table being played: names, prices, colours and order.
    pub board: board::Board,
    pub spaces: Vec<Space>,
    pub props: Vec<Prop>,
    pub players: Vec<Player>,
    pub turn: usize,
    pub phase: Phase,
    pub dice: (u8, u8),
    /// Seconds left in the dice tumble animation (0 = settled).
    pub dice_anim: f32,
    /// Recorded throw being replayed by `dice.rs`; empty for a loaded game.
    pub dice_throw: Option<Throw>,
    pub rolled_double: bool,
    pub double_count: u8,
    pub log: Vec<String>,
    pub chance: Deck,
    pub chest: Deck,
    pub houses_left: u32,
    pub hotels_left: u32,
    pub anim: Option<MoveAnim>,
    pub auction: Option<Auction>,
    pub trade: Option<Trade>,
    pub debt: Option<Debt>,
    /// (text, seconds remaining) shown in the middle of the board.
    pub banner: Option<(String, f32)>,
    pub pending_mod: LandingMod,
    pub pending_jail_steps: usize,
    pub ai_timer: f32,
    /// Set while a dialog covers the board: the world holds still.
    pub paused: bool,
    pub trade_timer: f32,
    /// Set once a bot has offered a trade during its current turn.
    pub bot_offer_done: bool,
    pub winner: Option<usize>,
    /// Players in the order they went bankrupt: the last one lasted longest.
    pub eliminated: Vec<usize>,
    /// Fast forward: every animation is resolved in the frame it starts.
    pub fast: bool,
}

impl Game {
    pub fn new(players: Vec<Player>, board: board::Board) -> Self {
        let mut game = Game {
            spaces: board.spaces.clone(),
            board,
            props: vec![Prop::default(); 40],
            players,
            turn: 0,
            phase: Phase::Roll,
            dice: (1, 1),
            dice_anim: 0.0,
            dice_throw: None,
            rolled_double: false,
            double_count: 0,
            log: Vec::new(),
            chance: cards::chance_deck(),
            chest: cards::chest_deck(),
            houses_left: START_HOUSES,
            hotels_left: START_HOTELS,
            anim: None,
            auction: None,
            paused: false,
            trade: None,
            debt: None,
            banner: None,
            pending_mod: LandingMod::Normal,
            pending_jail_steps: 0,
            ai_timer: 0.9,
            trade_timer: 0.8,
            bot_offer_done: false,
            winner: None,
            eliminated: Vec::new(),
            fast: false,
        };
        crate::ai::deal_strategies(&mut game);
        let first = game.players[0].name.clone();
        game.log(crate::tr!(
            "Welcome to Crazyopoly! {} goes first.",
            "Bienvenue dans Crazyopoly ! {} commence.",
            first
        ));
        game
    }

    // ---------------------------------------------------------------- helpers

    /// Swaps the table without touching the game state: used when the
    /// interface language changes in the middle of a game. Only a classic
    /// board is ever replaced, and the two share the same 40 spaces.
    pub fn set_board(&mut self, board: board::Board) {
        self.spaces = board.spaces.clone();
        self.board = board;
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
        if self.log.len() > 400 {
            self.log.remove(0);
        }
    }

    pub fn pname(&self, p: usize) -> String {
        self.players[p].name.clone()
    }

    pub fn current(&self) -> usize {
        self.turn
    }

    pub fn current_name(&self) -> String {
        self.players[self.turn].name.clone()
    }

    pub fn is_ai_turn(&self) -> bool {
        self.players[self.turn].ai
    }

    pub fn active_count(&self) -> usize {
        self.players.iter().filter(|p| !p.bankrupt).count()
    }

    pub fn alive(&self) -> Vec<usize> {
        (0..self.players.len())
            .filter(|&i| !self.players[i].bankrupt)
            .collect()
    }

    /// First space of this kind ahead of the player, wrapping around; the
    /// board decides where the stations and the utilities are.
    pub fn nearest_kind(&self, kind: Kind) -> usize {
        let pos = self.players[self.turn].pos;
        let mut fallback = None;
        for step in 1..=self.spaces.len() {
            let i = (pos + step) % self.spaces.len();
            if self.spaces[i].kind == kind {
                if i > pos {
                    return i;
                }
                fallback.get_or_insert(i);
            }
        }
        fallback.unwrap_or(0)
    }

    pub fn group_tiles(&self, group: Group) -> Vec<usize> {
        (0..40).filter(|&i| self.spaces[i].group == group).collect()
    }

    pub fn group_owned(&self, p: usize, group: Group) -> usize {
        (0..40)
            .filter(|&i| self.spaces[i].group == group && self.props[i].owner == Some(p))
            .count()
    }

    pub fn group_size(&self, group: Group) -> usize {
        (0..40).filter(|&i| self.spaces[i].group == group).count()
    }

    pub fn has_monopoly(&self, p: usize, group: Group) -> bool {
        group.street() && self.group_size(group) > 0 && self.group_owned(p, group) == self.group_size(group)
    }

    /// Rent owed for landing on `tile`, given the dice total and any card mod.
    pub fn rent_for(&self, tile: usize, dice: u8, mod_: LandingMod) -> i64 {
        let space = &self.spaces[tile];
        let Some(owner) = self.props[tile].owner else {
            return 0;
        };
        if self.props[tile].mortgaged || self.players[owner].bankrupt {
            return 0;
        }
        match space.group {
            Group::Railroad => {
                let n = self.group_owned(owner, Group::Railroad) as i64;
                let base = 25 * 2i64.pow((n.max(1) - 1) as u32);
                if mod_ == LandingMod::RailroadDouble {
                    base * 2
                } else {
                    base
                }
            }
            Group::Utility => {
                let n = self.group_owned(owner, Group::Utility);
                let mult = if n >= 2 { 10 } else { 4 };
                let d = dice.max(1) as i64;
                if mod_ == LandingMod::UtilityDice {
                    10 * d
                } else {
                    mult * d
                }
            }
            _ => {
                let h = self.props[tile].houses.min(MAX_HOUSES) as usize;
                let base = space.rent[h] as i64;
                if h == 0 && self.has_monopoly(owner, space.group) {
                    base * 2
                } else {
                    base
                }
            }
        }
    }

    pub fn mortgage_value(&self, tile: usize) -> i64 {
        (self.spaces[tile].price / 2) as i64
    }

    pub fn unmortgage_cost(&self, tile: usize) -> i64 {
        let m = self.mortgage_value(tile);
        m + (m + 9) / 10
    }

    pub fn owned_tiles(&self, p: usize) -> Vec<usize> {
        (0..40).filter(|&i| self.props[i].owner == Some(p)).collect()
    }

    pub fn net_worth(&self, p: usize) -> i64 {
        let mut total = self.players[p].cash;
        for i in 0..40 {
            if self.props[i].owner == Some(p) {
                if self.props[i].mortgaged {
                    total += self.mortgage_value(i);
                } else {
                    total += self.spaces[i].price as i64;
                    total += self.props[i].houses as i64 * self.spaces[i].house_cost as i64;
                }
            }
        }
        total
    }

    fn toggle(list: &mut Vec<usize>, tile: usize) {
        if let Some(pos) = list.iter().position(|t| *t == tile) {
            list.remove(pos);
        } else {
            list.push(tile);
        }
    }

    pub fn can_manage(&self) -> bool {
        matches!(self.phase, Phase::Roll | Phase::Manage | Phase::Debt)
    }

    // ------------------------------------------------------------ turn engine

    pub fn roll(&mut self) {
        if self.phase != Phase::Roll {
            return;
        }
        sfx(Sfx::Dice);
        let p = self.turn;
        // Roll with the physics engine: the values are whatever faces ended up
        // pointing at the sky, and the recorded poses drive the animation.
        let throw = crate::dice_physics::roll();
        let (d1, d2) = throw.values;
        self.dice_anim = if self.fast { 0.0 } else { throw.duration };
        self.dice_throw = Some(throw);
        self.dice = (d1, d2);
        let name = self.pname(p);
        self.log(crate::tr!(
            "{} rolls {} + {} = {}",
            "{} lance {} + {} = {}",
            name,
            d1,
            d2,
            d1 + d2
        ));

        if self.players[p].in_jail {
            if d1 == d2 {
                self.players[p].in_jail = false;
                self.players[p].jail_turns = 0;
                self.rolled_double = false;
                self.double_count = 0;
                self.log(crate::tr!(
                    "{} rolls doubles and leaves jail!",
                    "{} fait un double et sort de prison !",
                    self.pname(p)
                ));
                self.begin_move(p, (d1 + d2) as usize);
            } else {
                self.players[p].jail_turns += 1;
                if self.players[p].jail_turns >= 3 {
                    self.log(crate::tr!(
                        "{} failed three times and must pay the ${} fine.",
                        "{} a échoué trois fois et doit payer l'amende de {} $.",
                        self.pname(p),
                        JAIL_FINE
                    ));
                    self.pending_jail_steps = (d1 + d2) as usize;
                    self.charge(p, None, JAIL_FINE, Resume::JailMove);
                    if self.phase != Phase::Debt {
                        self.players[p].in_jail = false;
                        self.players[p].jail_turns = 0;
                        self.begin_move(p, (d1 + d2) as usize);
                    }
                } else {
                    self.log(crate::tr!(
                        "{} stays in jail (attempt {} of 3).",
                        "{} reste en prison (tentative {} sur 3).",
                        self.pname(p),
                        self.players[p].jail_turns
                    ));
                    self.phase = Phase::Manage;
                }
            }
            return;
        }

        if d1 == d2 {
            self.double_count += 1;
            self.rolled_double = true;
            if self.double_count >= 3 {
                self.log(crate::tr!(
                    "{} rolled doubles three times in a row - straight to jail!",
                    "{} a fait trois doubles d'affilée : direction la prison !",
                    self.pname(p)
                ));
                self.send_to_jail(p);
                self.double_count = 0;
                self.rolled_double = false;
                self.phase = Phase::Manage;
                return;
            }
            self.log(crate::tr!(
                "Doubles! {} rolls again after moving.",
                "Double ! {} relancera les dés après son déplacement.",
                self.pname(p)
            ));
        } else {
            self.rolled_double = false;
        }
        self.pending_mod = LandingMod::Normal;
        self.begin_move(p, (d1 + d2) as usize);
    }

    fn begin_move(&mut self, p: usize, steps: usize) {
        let start = self.players[p].pos;
        if steps == 0 {
            self.anim = None;
            self.resolve_landing(p);
            return;
        }
        let mut path = Vec::with_capacity(steps);
        let mut passed_go = false;
        for k in 1..=steps {
            let next = (start + k) % 40;
            if next == 0 {
                passed_go = true;
            }
            path.push(next);
        }
        if passed_go {
            self.players[p].cash += GO_SALARY;
            self.players[p].laps += 1;
            self.log(crate::tr!(
                "{} passes GO and collects ${}.",
                "{} passe par la case DÉPART et reçoit {} $.",
                self.pname(p),
                GO_SALARY
            ));
        }
        self.anim = Some(MoveAnim {
            player: p,
            path,
            idx: 0,
            t: 0.0,
        });
        self.phase = Phase::Moving;
    }

    fn move_back(&mut self, p: usize, steps: usize) {
        let start = self.players[p].pos;
        let mut path = Vec::with_capacity(steps);
        for k in 1..=steps {
            path.push((start + 40 - k) % 40);
        }
        self.anim = Some(MoveAnim {
            player: p,
            path,
            idx: 0,
            t: 0.0,
        });
        self.phase = Phase::Moving;
    }

    /// Advance the current player to a specific tile (Chance/Community Chest).
    fn advance_to(&mut self, tile: usize, mod_: LandingMod) {
        let p = self.turn;
        let pos = self.players[p].pos;
        self.pending_mod = mod_;
        if pos == tile {
            self.resolve_landing(p);
        } else {
            let steps = (tile + 40 - pos) % 40;
            self.begin_move(p, steps);
        }
    }

    fn resolve_landing(&mut self, p: usize) {
        let pos = self.players[p].pos;
        let kind = self.spaces[pos].kind;
        let name = self.spaces[pos].name.clone();
        let pname = self.pname(p);

        match kind {
            Kind::Go => self.log(crate::tr!(
                "{} lands on GO.",
                "{} s'arrête sur la case DÉPART.",
                pname
            )),
            Kind::Street | Kind::Railroad | Kind::Utility => {
                match self.props[pos].owner {
                    Some(owner) if owner != p => {
                        if self.props[pos].mortgaged {
                            self.log(crate::tr!(
                                "{} is mortgaged - no rent is due.",
                                "{} est hypothéqué : aucun loyer n'est dû.",
                                name
                            ));
                        } else {
                            let rent = self.rent_for(pos, self.dice.0 + self.dice.1, self.pending_mod);
                            let owner_name = self.pname(owner);
                            self.log(crate::tr!(
                                "{} lands on {} owned by {} - rent ${}.",
                                "{} s'arrête sur {} appartenant à {} : loyer de {} $.",
                                pname, name, owner_name, rent
                            ));
                            self.charge(p, Some(owner), rent, Resume::FinishTurn);
                        }
                    }
                    Some(_) => self.log(crate::tr!(
                        "{} rests on their own {}.",
                        "{} s'arrête sur son propre {}.",
                        pname, name
                    )),
                    None => {
                        let price = self.spaces[pos].price;
                        self.log(crate::tr!(
                            "{} lands on {} - unowned, ${}.",
                            "{} s'arrête sur {} : case libre, {} $.",
                            pname, name, price
                        ));
                        self.phase = Phase::AwaitBuy;
                        return;
                    }
                }
            }
            Kind::Tax => {
                let amount = self.spaces[pos].tax as i64;
                self.log(crate::tr!(
                    "{} pays ${} for {}.",
                    "{} paie {} $ pour {}.",
                    pname, amount, name
                ));
                self.charge(p, None, amount, Resume::FinishTurn);
            }
            Kind::Chance | Kind::Chest => {
                self.draw_card(kind == Kind::Chance);
            }
            Kind::GoToJail => {
                self.log(crate::tr!(
                    "{} is sent to jail!",
                    "{} est envoyé en prison !",
                    pname
                ));
                self.send_to_jail(p);
            }
            _ => {
                self.log(crate::tr!(
                    "{} lands on {}.",
                    "{} s'arrête sur {}.",
                    pname, name
                ));
            }
        }

        if self.anim.is_some() || self.phase == Phase::Debt || self.phase == Phase::AwaitBuy {
            return;
        }
        self.finish_landing();
    }

    fn finish_landing(&mut self) {
        self.pending_mod = LandingMod::Normal;
        let p = self.turn;
        if self.players[p].bankrupt {
            return;
        }
        if self.players[p].in_jail {
            self.phase = Phase::Manage;
            return;
        }
        if self.rolled_double && self.double_count < 3 {
            self.phase = Phase::Roll;
        } else {
            self.phase = Phase::Manage;
        }
    }

    pub fn send_to_jail(&mut self, p: usize) {
        self.players[p].pos = JAIL_POS;
        self.players[p].in_jail = true;
        sfx(Sfx::Jail);
        self.players[p].jail_turns = 0;
        self.rolled_double = false;
        self.pending_mod = LandingMod::Normal;
        let name = self.pname(p);
        self.log(crate::tr!(
            "{} goes to jail.",
            "{} va en prison.",
            name
        ));
    }

    // ------------------------------------------------------------------ cards

    fn draw_card(&mut self, chance: bool) {
        let card: Card = if chance {
            self.chance.draw()
        } else {
            self.chest.draw()
        };
        let deck_name = if chance {
            i18n::t("CHANCE", "CHANCE")
        } else {
            i18n::t("COMMUNITY CHEST", "CAISSE DE COMMUNAUTÉ")
        };
        let who = self.pname(self.turn);
        let text = card.sentence(&who, &self.board);
        self.log(format!("{} - {}", deck_name, text));
        self.banner = Some((text, 5.0));
        sfx(Sfx::Card);
        self.apply_card(card);
    }

    fn apply_card(&mut self, card: Card) {
        let p = self.turn;
        match card.effect {
            Effect::AdvanceToGo => self.advance_to(0, LandingMod::Normal),
            Effect::AdvanceTo(tile) => self.advance_to(tile, LandingMod::Normal),
            Effect::NearestUtility => {
                let target = self.nearest_kind(Kind::Utility);
                self.advance_to(target, LandingMod::UtilityDice);
            }
            Effect::NearestRailroad => {
                let target = self.nearest_kind(Kind::Railroad);
                self.advance_to(target, LandingMod::RailroadDouble);
            }
            Effect::Back(steps) => self.move_back(p, steps),
            Effect::GoToJail => self.send_to_jail(p),
            Effect::JailFree => {
                self.players[p].jail_cards += 1;
                self.log(crate::tr!(
                    "{} keeps a Get Out of Jail Free card.",
                    "{} conserve une carte Sortie de prison.",
                    self.pname(p)
                ));
            }
            Effect::Collect(amount) => {
                self.players[p].cash += amount;
                self.log(crate::tr!(
                    "{} collects ${}.",
                    "{} reçoit {} $.",
                    self.pname(p),
                    amount
                ));
            }
            Effect::Pay(amount) => {
                self.charge(p, None, amount, Resume::FinishTurn);
            }
            Effect::CollectFromEach(amount) => {
                let others = self.alive();
                let mut total = 0;
                for o in others {
                    if o == p {
                        continue;
                    }
                    let take = self.players[o].cash.min(amount);
                    self.players[o].cash -= take;
                    total += take;
                }
                self.players[p].cash += total;
                self.log(crate::tr!(
                    "{} collects ${} in total.",
                    "{} reçoit {} $ au total.",
                    self.pname(p),
                    total
                ));
            }
            Effect::PayEach(amount) => {
                let others: Vec<usize> = self.alive().into_iter().filter(|o| *o != p).collect();
                let total = amount * others.len() as i64;
                if self.players[p].cash >= total {
                    for o in &others {
                        self.players[*o].cash += amount;
                    }
                    self.players[p].cash -= total;
                    self.log(crate::tr!(
                        "{} pays ${} to each player.",
                        "{} paie {} $ à chaque joueur.",
                        self.pname(p),
                        amount
                    ));
                } else {
                    self.log(crate::tr!(
                        "{} cannot afford ${} - the bank must be paid.",
                        "{} ne peut pas payer {} $ : la banque doit être remboursée.",
                        self.pname(p),
                        total
                    ));
                    self.charge(p, None, total, Resume::FinishTurn);
                }
            }
            Effect::Repairs { house, hotel } => {
                let mut houses = 0i64;
                let mut hotels = 0i64;
                for i in self.owned_tiles(p) {
                    match self.props[i].houses {
                        5 => hotels += 1,
                        h => houses += h as i64,
                    }
                }
                let total = houses * house + hotels * hotel;
                self.log(crate::tr!(
                    "{} owes ${} for repairs ({} houses, {} hotels).",
                    "{} doit {} $ de réparations ({} maisons, {} hôtels).",
                    self.pname(p),
                    total,
                    houses,
                    hotels
                ));
                self.charge(p, None, total, Resume::FinishTurn);
            }
        }
    }

    // --------------------------------------------------------------- purchase

    pub fn buy(&mut self) {
        if self.phase != Phase::AwaitBuy {
            return;
        }
        let p = self.turn;
        let pos = self.players[p].pos;
        let price = self.spaces[pos].price as i64;
        if price == 0 || self.players[p].cash < price {
            return;
        }
        self.players[p].cash -= price;
        self.props[pos].owner = Some(p);
        self.props[pos].mortgaged = false;
        sfx(Sfx::Buy);
        let name = self.spaces[pos].name.clone();
        self.log(crate::tr!(
            "{} buys {} for ${}.",
            "{} achète {} pour {} $.",
            self.pname(p),
            name,
            price
        ));
        self.finish_landing();
    }

    pub fn start_auction_current(&mut self) {
        if self.phase != Phase::AwaitBuy {
            return;
        }
        let tile = self.players[self.turn].pos;
        self.start_auction(tile);
    }

    pub fn start_auction(&mut self, tile: usize) {
        let n = self.players.len();
        let active: Vec<bool> = self.players.iter().map(|p| !p.bankrupt).collect();
        let mut turn = self.turn;
        for _ in 0..n {
            if active[turn] {
                break;
            }
            turn = (turn + 1) % n;
        }
        self.auction = Some(Auction {
            tile,
            bid: 0,
            high: None,
            active,
            turn,
        });
        self.phase = Phase::Auction;
        self.ai_timer = 0.8;
        let name = self.spaces[tile].name.clone();
        self.log(crate::tr!(
            "{} goes up for auction.",
            "{} passe aux enchères.",
            name
        ));
    }

    pub fn auction_bid(&mut self, inc: i64) {
        let p = match &self.auction {
            Some(a) => a.turn,
            None => return,
        };
        let new_bid = self.auction.as_ref().map(|a| a.bid).unwrap_or(0) + inc;
        if new_bid > self.players[p].cash || inc <= 0 {
            return;
        }
        if let Some(a) = self.auction.as_mut() {
            a.bid = new_bid;
            a.high = Some(p);
        }
        let name = self.pname(p);
        self.log(crate::tr!(
            "{} bids ${}.",
            "{} mise {} $.",
            name,
            new_bid
        ));
        self.auction_next();
    }

    pub fn auction_pass(&mut self) {
        let p = match &self.auction {
            Some(a) => a.turn,
            None => return,
        };
        if let Some(a) = self.auction.as_mut() {
            a.active[p] = false;
        }
        let name = self.pname(p);
        self.log(crate::tr!("{} passes.", "{} passe.", name));
        self.auction_next();
    }

    fn auction_next(&mut self) {
        let Some(auction) = self.auction.as_ref() else {
            return;
        };
        let n = self.players.len();
        let high = auction.high;
        let mut t = auction.turn;
        let mut next = None;
        for _ in 0..n {
            t = (t + 1) % n;
            if auction.active[t] && Some(t) != high {
                next = Some(t);
                break;
            }
        }
        match next {
            Some(t) => {
                if let Some(auction) = self.auction.as_mut() {
                    auction.turn = t;
                }
                self.ai_timer = 0.7;
            }
            None => self.auction_finish(high.is_some()),
        }
    }

    fn auction_finish(&mut self, sold: bool) {
        let Some(a) = self.auction.take() else {
            return;
        };
        if sold {
            if let Some(winner) = a.high {
                let price = a.bid;
                self.players[winner].cash -= price;
                self.props[a.tile].owner = Some(winner);
                let (wname, tname) = (self.pname(winner), self.spaces[a.tile].name.clone());
                self.log(crate::tr!(
                    "{} wins {} for ${}.",
                    "{} remporte {} pour {} $.",
                    wname,
                    tname,
                    price
                ));
            }
        } else {
            let name = self.spaces[a.tile].name.clone();
            self.log(crate::tr!(
                "No bids for {} - it stays with the bank.",
                "Aucune offre pour {} : la propriété reste à la banque.",
                name
            ));
        }
        self.auction = None;
        self.finish_landing();
    }

    // ------------------------------------------------------------------- debt

    pub fn charge(&mut self, from: usize, to: Option<usize>, amount: i64, resume: Resume) {
        if amount <= 0 {
            return;
        }
        if self.players[from].cash >= amount {
            sfx(Sfx::Cash);
            self.players[from].cash -= amount;
            if let Some(t) = to {
                self.players[t].cash += amount;
            }
        } else {
            let name = self.pname(from);
            let creditor = match to {
                Some(t) => crate::tr!("to {}", "à {}", self.pname(t)),
                None => i18n::t("to the bank", "à la banque").to_string(),
            };
            self.log(crate::tr!(
                "{} owes ${} {} but only has ${}.",
                "{} doit {} $ {} mais ne possède que {} $.",
                name,
                amount,
                creditor,
                self.players[from].cash
            ));
            self.debt = Some(Debt {
                creditor: to,
                amount,
                resume,
            });
            self.phase = Phase::Debt;
            sfx(Sfx::Bad);
        }
    }

    pub fn debt_amount(&self) -> i64 {
        self.debt.as_ref().map(|d| d.amount).unwrap_or(0)
    }

    pub fn pay_debt(&mut self) {
        let Some(d) = self.debt.take() else {
            return;
        };
        let p = self.turn;
        if self.players[p].cash < d.amount {
            self.debt = Some(d);
            return;
        }
        self.players[p].cash -= d.amount;
        sfx(Sfx::Cash);
        if let Some(c) = d.creditor {
            self.players[c].cash += d.amount;
        }
        let name = self.pname(p);
        self.log(crate::tr!("{} pays ${}.", "{} paie {} $.", name, d.amount));
        match d.resume {
            Resume::FinishTurn => self.finish_landing(),
            Resume::JailRoll => self.phase = Phase::Roll,
            Resume::JailMove => {
                self.players[p].in_jail = false;
                self.players[p].jail_turns = 0;
                let steps = self.pending_jail_steps;
                self.begin_move(p, steps);
            }
        }
    }

    pub fn declare_bankrupt(&mut self) {
        let p = self.turn;
        let creditor = self
            .debt
            .as_ref()
            .and_then(|d| d.creditor)
            .filter(|c| *c != p);
        let name = self.pname(p);

        for i in 0..40 {
            if self.props[i].owner == Some(p) {
                let h = self.props[i].houses;
                if h == 5 {
                    self.hotels_left += 1;
                    self.houses_left += 4;
                } else {
                    self.houses_left += h as u32;
                }
                self.props[i].houses = 0;
            }
        }
        let cash = self.players[p].cash;
        let jail_cards = self.players[p].jail_cards;

        // Every deed goes back on the market, free of mortgage: anyone who
        // lands on it from now on can buy it again.
        let mut released = 0;
        for i in 0..40 {
            if self.props[i].owner == Some(p) {
                self.props[i].owner = None;
                self.props[i].mortgaged = false;
                released += 1;
            }
        }

        if let Some(c) = creditor {
            self.players[c].cash += cash;
            self.players[c].jail_cards += jail_cards;
            let cname = self.pname(c);
            self.log(crate::tr!(
                "{} is bankrupt - ${} passes to {}.",
                "{} est en faillite : {} $ reviennent à {}.",
                name, cash, cname
            ));
        } else {
            self.log(crate::tr!("{} is bankrupt.", "{} est en faillite.", name));
        }
        if released > 0 {
            self.log(crate::tr!(
                "Their {} deed(s) return to the bank and can be bought again.",
                "Ses {} titre(s) retournent à la banque et peuvent être rachetés.",
                released
            ));
        }

        self.players[p].cash = 0;
        self.players[p].jail_cards = 0;
        self.eliminated.push(p);
        self.players[p].bankrupt = true;
        self.players[p].in_jail = false;
        self.debt = None;
        self.trade = None;
        self.phase = Phase::Manage;

        if self.active_count() <= 1 {
            self.declare_winner();
        } else {
            self.end_turn();
        }
    }

    pub fn can_afford_debt(&self) -> bool {
        let p = self.turn;
        self.players[p].cash >= self.debt_amount()
    }

    // --------------------------------------------------------------- building

    pub fn can_build(&self, tile: usize) -> bool {
        let p = self.turn;
        let space = &self.spaces[tile];
        if self.props[tile].owner != Some(p) || !space.group.street() {
            return false;
        }
        if !self.has_monopoly(p, space.group) || self.props[tile].mortgaged {
            return false;
        }
        if self.group_tiles(space.group)
            .iter()
            .any(|i| self.props[*i].mortgaged)
        {
            return false;
        }
        let h = self.props[tile].houses;
        if h >= MAX_HOUSES {
            return false;
        }
        // Even-building rule: only the least-developed lots may be built on.
        if self
            .group_tiles(space.group)
            .iter()
            .any(|i| self.props[*i].houses < h)
        {
            return false;
        }
        true
    }

    pub fn build_house(&mut self, tile: usize) -> bool {
        if !self.can_build(tile) {
            return false;
        }
        let p = self.turn;
        let cost = self.spaces[tile].house_cost as i64;
        if self.players[p].cash < cost {
            return false;
        }
        self.players[p].cash -= cost;
        sfx(Sfx::Build);
        let h = self.props[tile].houses;
        self.props[tile].houses = if h == 4 { 5 } else { h + 1 };
        let name = self.spaces[tile].name.clone();
        let what = if self.props[tile].houses == 5 {
            i18n::t("a hotel", "un hôtel")
        } else {
            i18n::t("a house", "une maison")
        };
        self.log(crate::tr!(
            "{} builds {} on {} (${}).",
            "{} construit {} sur {} ({} $).",
            self.pname(p),
            what,
            name,
            cost
        ));
        true
    }

    pub fn can_sell_house(&self, tile: usize) -> bool {
        let p = self.turn;
        if self.props[tile].owner != Some(p) {
            return false;
        }
        let space = &self.spaces[tile];
        if !space.group.street() {
            return false;
        }
        let h = self.props[tile].houses;
        if h == 0 {
            return false;
        }
        let group = self.group_tiles(space.group);
        // Even-selling rule: sell from the most developed lots first.
        if group.iter().any(|i| self.props[*i].houses > h) {
            return false;
        }
        true
    }

    pub fn sell_house(&mut self, tile: usize) -> bool {
        if !self.can_sell_house(tile) {
            return false;
        }
        let p = self.turn;
        let cost = self.spaces[tile].house_cost as i64;
        let h = self.props[tile].houses;
        let refund = if h == 5 { cost * 5 / 2 } else { cost / 2 };
        self.props[tile].houses = if h == 5 { 4 } else { h - 1 };
        self.players[p].cash += refund;
        let name = self.spaces[tile].name.clone();
        self.log(crate::tr!(
            "{} sells a building on {} for ${}.",
            "{} revend une construction sur {} pour {} $.",
            self.pname(p),
            name,
            refund
        ));
        true
    }

    pub fn mortgage(&mut self, tile: usize) -> bool {
        let p = self.turn;
        if self.props[tile].owner != Some(p)
            || self.props[tile].mortgaged
            || self.props[tile].houses > 0
        {
            return false;
        }
        let value = self.mortgage_value(tile);
        self.props[tile].mortgaged = true;
        self.players[p].cash += value;
        let name = self.spaces[tile].name.clone();
        self.log(crate::tr!(
            "{} mortgages {} for ${}.",
            "{} hypothèque {} pour {} $.",
            self.pname(p),
            name,
            value
        ));
        true
    }

    pub fn unmortgage(&mut self, tile: usize) -> bool {
        let p = self.turn;
        if self.props[tile].owner != Some(p) || !self.props[tile].mortgaged {
            return false;
        }
        let cost = self.unmortgage_cost(tile);
        if self.players[p].cash < cost {
            return false;
        }
        self.players[p].cash -= cost;
        self.props[tile].mortgaged = false;
        let name = self.spaces[tile].name.clone();
        self.log(crate::tr!(
            "{} lifts the mortgage on {} for ${}.",
            "{} lève l'hypothèque sur {} pour {} $.",
            self.pname(p),
            name,
            cost
        ));
        true
    }

    pub fn use_jail_card(&mut self) -> bool {
        let p = self.turn;
        if self.phase != Phase::Roll || !self.players[p].in_jail || self.players[p].jail_cards == 0 {
            return false;
        }
        self.players[p].jail_cards -= 1;
        self.players[p].in_jail = false;
        self.players[p].jail_turns = 0;
        self.log(crate::tr!(
            "{} uses a Get Out of Jail Free card.",
            "{} utilise une carte Sortie de prison.",
            self.pname(p)
        ));
        true
    }

    pub fn pay_jail_fine(&mut self) {
        let p = self.turn;
        if self.phase != Phase::Roll || !self.players[p].in_jail {
            return;
        }
        self.players[p].in_jail = false;
        self.players[p].jail_turns = 0;
        self.log(crate::tr!(
            "{} pays the ${} fine.",
            "{} paie l'amende de {} $.",
            self.pname(p),
            JAIL_FINE
        ));
        self.charge(p, None, JAIL_FINE, Resume::JailRoll);
    }

    // ------------------------------------------------------------------ trade

    pub fn open_trade(&mut self) {
        if self.trade.is_some() || self.players[self.turn].bankrupt {
            return;
        }
        let from = self.turn;
        let to = (0..self.players.len()).find(|&i| i != from && !self.players[i].bankrupt);
        let Some(to) = to else {
            return;
        };
        self.trade = Some(Trade {
            from,
            to,
            give: Vec::new(),
            get: Vec::new(),
            give_cash: 0,
            get_cash: 0,
            state: TradeState::Draft,
            notice: String::new(),
        });
    }

    pub fn close_trade(&mut self) {
        self.trade = None;
    }

    pub fn cycle_trade_partner(&mut self) {
        let from = self.turn;
        let n = self.players.len();
        let Some(t) = self.trade.as_mut() else {
            return;
        };
        if t.state == TradeState::Proposed || t.from != from {
            return;
        }
        let mut i = t.to;
        for _ in 0..n {
            i = (i + 1) % n;
            if i != from && !self.players[i].bankrupt {
                t.to = i;
                t.give.clear();
                t.get.clear();
                t.give_cash = 0;
                t.get_cash = 0;
                t.notice.clear();
                break;
            }
        }
    }

    pub fn trade_toggle(&mut self, tile: usize) {
        let turn = self.turn;
        let owner = self.props[tile].owner;
        let houses = self.props[tile].houses;
        let Some(t) = self.trade.as_mut() else {
            return;
        };
        if t.state != TradeState::Draft || t.from != turn {
            return;
        }
        if owner == Some(t.from) {
            if houses > 0 {
                t.notice = i18n::t(
                    "Sell the buildings before trading this lot.",
                    "Vendez les constructions avant d'échanger cette case.",
                )
                .to_string();
                return;
            }
            Self::toggle(&mut t.give, tile);
        } else if owner == Some(t.to) {
            if houses > 0 {
                t.notice = i18n::t(
                    "Your partner must sell the buildings first.",
                    "Votre partenaire doit d'abord vendre les constructions.",
                )
                .to_string();
                return;
            }
            Self::toggle(&mut t.get, tile);
        }
    }

    /// Adjust cash in the draft. `from_side` targets what the proposer pays.
    pub fn trade_cash(&mut self, from_side: bool, delta: i64) {
        let from = self.turn;
        let Some(probe) = self.trade.as_ref() else {
            return;
        };
        let from_cash = self.players[probe.from].cash;
        let to_cash = self.players[probe.to].cash;
        let Some(t) = self.trade.as_mut() else {
            return;
        };
        if t.state != TradeState::Draft || t.from != from {
            return;
        }
        if from_side {
            t.give_cash = (t.give_cash + delta).clamp(0, from_cash);
        } else {
            t.get_cash = (t.get_cash + delta).clamp(0, to_cash);
        }
    }

    pub fn propose_trade(&mut self) {
        let from = self.turn;
        let Some(probe) = self.trade.as_ref() else {
            return;
        };
        let from_cash = self.players[probe.from].cash;
        let to_cash = self.players[probe.to].cash;
        let to_name = self.pname(probe.to);
        let Some(t) = self.trade.as_mut() else {
            return;
        };
        if t.state != TradeState::Draft || t.from != from {
            return;
        }
        if t.give.is_empty() && t.get.is_empty() && t.give_cash == 0 && t.get_cash == 0 {
            t.notice = i18n::t(
                "Select something to trade.",
                "Sélectionnez quelque chose à échanger.",
            )
            .to_string();
            return;
        }
        if t.give_cash > from_cash || t.get_cash > to_cash {
            t.notice = i18n::t(
                "Neither side has that much cash.",
                "L'un des deux camps n'a pas assez d'argent.",
            )
            .to_string();
            return;
        }
        t.state = TradeState::Proposed;
        t.notice.clear();
        let from_name = self.pname(from);
        self.log(crate::tr!(
            "{} proposes a trade to {}.",
            "{} propose un échange à {}.",
            from_name,
            to_name
        ));
        self.trade_timer = 0.8;
        self.ai_timer = 0.8;
    }

    pub fn accept_trade(&mut self) {
        let Some(t) = self.trade.take() else {
            return;
        };
        if t.state != TradeState::Proposed {
            self.trade = Some(t);
            return;
        }
        let (from, to) = (t.from, t.to);
        self.players[from].cash -= t.give_cash;
        self.players[to].cash += t.give_cash;
        self.players[to].cash -= t.get_cash;
        self.players[from].cash += t.get_cash;
        for tile in &t.give {
            if self.props[*tile].owner == Some(from) {
                self.props[*tile].owner = Some(to);
            }
        }
        for tile in &t.get {
            if self.props[*tile].owner == Some(to) {
                self.props[*tile].owner = Some(from);
            }
        }
        let (a, b) = (self.pname(from), self.pname(to));
        self.log(crate::tr!(
            "{} and {} complete a trade.",
            "{} et {} concluent un échange.",
            a,
            b
        ));
        self.ai_timer = 0.8;
    }

    pub fn decline_trade(&mut self) {
        let Some(trade) = self.trade.as_ref() else {
            return;
        };
        let (from, to) = (trade.from, trade.to);
        if self.players[from].ai {
            // A bot's own offer: close it outright so the game can continue.
            self.trade = None;
        } else if let Some(t) = self.trade.as_mut() {
            t.state = TradeState::Draft;
            t.notice = i18n::t("Trade declined.", "Échange refusé.").to_string();
        }
        let name = self.pname(to);
        self.log(crate::tr!(
            "{} declines the trade.",
            "{} refuse l'échange.",
            name
        ));
        self.ai_timer = 0.8;
    }

    pub fn withdraw_trade(&mut self) {
        self.trade = None;
        self.ai_timer = 0.8;
    }

    /// A bot offers cash for the single lot it needs to complete a colour group.
    pub fn bot_trade_offer(&mut self) -> bool {
        let p = self.turn;
        if self.trade.is_some() || !self.players[p].ai || self.bot_offer_done {
            return false;
        }
        self.bot_offer_done = true;
        if self.players[p].cash < 300 {
            return false;
        }
        for group in self.board.street_groups() {
            let tiles = self.group_tiles(group);
            if tiles.is_empty() || self.group_owned(p, group) + 1 != tiles.len() {
                continue;
            }
            let Some(target) = tiles.iter().copied().find(|t| self.props[*t].owner != Some(p)) else {
                continue;
            };
            let Some(owner) = self.props[target].owner else {
                continue;
            };
            if self.players[owner].bankrupt {
                continue;
            }
            let price = self.spaces[target].price as i64;
            let name = self.spaces[target].name.clone();
            let bot = self.pname(p);
            if self.players[owner].ai {
                // Two bots settle on the spot: raise the offer step by step, but
                // never above what we can spare nor 2.5x the list price.
                let cap = (self.players[p].cash - 200).min(price * 9 / 2);
                let step = (price / 6).max(25);
                let mut asked = (price * 2).min(cap);
                let mut accepted = false;
                while asked >= price / 2 && asked <= cap {
                    self.trade = Some(Trade {
                        from: p,
                        to: owner,
                        give: Vec::new(),
                        get: vec![target],
                        give_cash: asked,
                        get_cash: 0,
                        state: TradeState::Proposed,
                        notice: String::new(),
                    });
                    accepted = crate::ai::evaluate_trade(self);
                    self.trade = None;
                    if accepted {
                        break;
                    }
                    asked += step;
                }
                if !accepted {
                    continue;
                }
                self.log(crate::tr!(
                    "{} offers ${} for {}.",
                    "{} propose {} $ pour {}.",
                    bot,
                    asked,
                    name
                ));
                self.trade = Some(Trade {
                    from: p,
                    to: owner,
                    give: Vec::new(),
                    get: vec![target],
                    give_cash: asked,
                    get_cash: 0,
                    state: TradeState::Proposed,
                    notice: String::new(),
                });
                self.accept_trade();
                return true;
            }
            let offer = ((price as f64) * 1.7) as i64;
            let offer = offer.min(self.players[p].cash - 200);
            if offer < price / 2 {
                continue;
            }
            self.trade = Some(Trade {
                from: p,
                to: owner,
                give: Vec::new(),
                get: vec![target],
                give_cash: offer,
                get_cash: 0,
                state: TradeState::Proposed,
                notice: String::new(),
            });
            self.log(crate::tr!(
                "{} offers ${} for {}.",
                "{} propose {} $ pour {}.",
                bot,
                offer,
                name
            ));
            self.trade_timer = 0.0;
            return true;
        }
        false
    }

    /// Absolute value of a package of properties plus cash, from one player's
    /// point of view. Used by the bot brain.
    pub fn package_value(&self, tiles: &[usize], cash: i64, for_player: usize) -> f64 {
        let mut total = cash as f64;
        let mut groups: Vec<(Group, usize)> = Vec::new();
        for tile in tiles {
            total += ai::property_value(self, *tile, for_player);
            let g = self.spaces[*tile].group;
            if let Some(entry) = groups.iter_mut().find(|(gr, _)| *gr == g) {
                entry.1 += 1;
            } else {
                groups.push((g, 1));
            }
        }
        // Completing a colour group is worth far more than its face value.
        for (g, count) in groups {
            if g.street() {
                let owned = self.group_owned(for_player, g);
                if owned + count >= self.group_size(g) {
                    total += 150.0;
                }
            }
        }
        total
    }

    // ------------------------------------------------------------ turn ending

    pub fn end_turn(&mut self) {
        if self.phase == Phase::Over {
            return;
        }
        if self.active_count() <= 1 {
            self.declare_winner();
            return;
        }
        self.rolled_double = false;
        self.double_count = 0;
        self.pending_mod = LandingMod::Normal;
        self.trade = None;
        self.auction = None;
        self.debt = None;
        self.bot_offer_done = false;
        let n = self.players.len();
        let mut t = self.turn;
        for _ in 0..n {
            t = (t + 1) % n;
            if !self.players[t].bankrupt {
                self.turn = t;
                break;
            }
        }
        self.phase = Phase::Roll;
        self.ai_timer = 0.9;
        let name = self.current_name();
        self.log(crate::tr!("-- {}'s turn --", "-- Tour de {} --", name));
    }

    /// Final ranking: the last player standing first, then the others in the
    /// reverse order they went bankrupt, so whoever fell first ranks last.
    pub fn standings(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.players.len()).collect();
        order.sort_by(|a, b| {
            match (self.players[*a].bankrupt, self.players[*b].bankrupt) {
                (false, false) => self.net_worth(*b).cmp(&self.net_worth(*a)),
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                (true, true) => self.elimination_rank(*b).cmp(&self.elimination_rank(*a)),
            }
        });
        order
    }

    /// How long a bankrupt player lasted: the higher, the later they fell.
    /// Anyone still standing (or unknown, from an older save) counts as 0.
    pub fn elimination_rank(&self, player: usize) -> usize {
        self.eliminated
            .iter()
            .position(|p| *p == player)
            .map(|i| i + 1)
            .unwrap_or(0)
    }

    pub fn declare_winner(&mut self) {
        let winner = self.players.iter().position(|p| !p.bankrupt);
        self.winner = winner;
        self.phase = Phase::Over;
        self.anim = None;
        sfx(Sfx::Win);
        if let Some(w) = winner {
            let name = self.pname(w);
            self.log(crate::tr!(
                "{} owns the board - game over!",
                "{} possède tout le plateau - partie terminée !",
                name
            ));
        }
    }

    /// Ends the game early: the richest player (cash + deeds + buildings) wins.
    pub fn finish_early(&mut self) {
        if self.phase == Phase::Over {
            return;
        }
        let mut best: Option<usize> = None;
        let mut top = i64::MIN;
        for p in 0..self.players.len() {
            if self.players[p].bankrupt {
                continue;
            }
            let worth = self.net_worth(p);
            if worth > top {
                top = worth;
                best = Some(p);
            }
        }
        self.winner = best;
        self.phase = Phase::Over;
        self.anim = None;
        self.auction = None;
        self.trade = None;
        self.debt = None;
        if let Some(w) = best {
            let name = self.pname(w);
            self.log(crate::tr!(
                "The table stops here - {} wins with {}.",
                "La table s'arrête là - {} l'emporte avec {}.",
                name,
                top
            ));
            sfx(Sfx::Win);
        }
    }

    // ------------------------------------------------------------------ frame

    fn awaiting_ai(&self) -> bool {
        if self.trade.is_some() {
            return false;
        }
        match self.phase {
            Phase::Roll | Phase::Manage | Phase::AwaitBuy | Phase::Debt => self.players[self.turn].ai,
            Phase::Auction => self
                .auction
                .as_ref()
                .map(|a| self.players[a.turn].ai)
                .unwrap_or(false),
            _ => false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        if let Some(b) = self.banner.as_mut() {
            b.1 -= dt;
        }
        if self.banner.as_ref().map(|b| b.1 <= 0.0).unwrap_or(false) {
            self.banner = None;
        }
        if self.dice_anim > 0.0 {
            self.dice_anim = (self.dice_anim - dt).max(0.0);
        }

        // Animate the walking token one tile at a time (instant in fast mode).
        let mut arrived = false;
        if self.fast {
            if let Some(a) = self.anim.take() {
                if let Some(last) = a.path.last() {
                    self.players[a.player].pos = *last;
                }
                arrived = true;
            }
        } else if let Some(a) = self.anim.as_mut() {
            let step = 0.075;
            a.t += dt / step;
            while a.t >= 1.0 {
                if a.idx + 1 < a.path.len() {
                    a.idx += 1;
                    a.t -= 1.0;
                    sfx(Sfx::Step);
                } else {
                    a.t = 0.0;
                    arrived = true;
                    break;
                }
            }
            let pos = a.path[a.idx];
            let who = a.player;
            self.players[who].pos = pos;
        }
        if arrived {
            self.anim = None;
            let p = self.turn;
            self.resolve_landing(p);
        }

        if self.phase == Phase::Over {
            return;
        }

        // Let a bot answer a trade offer.
        let bot_trade = match &self.trade {
            Some(t) => t.state == TradeState::Proposed && self.players[t.to].ai,
            None => false,
        };
        if bot_trade {
            if self.fast {
                self.trade_timer = 0.0;
            }
            self.trade_timer -= dt;
            if self.trade_timer <= 0.0 {
                let accept = ai::evaluate_trade(self);
                if accept {
                    self.accept_trade();
                } else {
                    self.decline_trade();
                }
            }
        }

        if self.phase != Phase::Moving && self.awaiting_ai() {
            if self.fast {
                self.ai_timer = 0.0;
            }
            self.ai_timer -= dt;
            if self.ai_timer <= 0.0 {
                ai::act(self);
                self.ai_timer = 0.8;
            }
        }
    }
}





/// Play one of the game's sound effects (a no-op when audio is unavailable).
fn sfx(kind: Sfx) {
    crate::audio::sfx(kind);
}

use crate::audio::Sfx;

#[cfg(test)]
mod standings_tests {
    use super::{Game, Player, PLAYER_COLORS};

    #[test]
    fn the_ranking_follows_the_order_players_fell() {
        let players = (0..3)
            .map(|i| Player::new(format!("P{}", i + 1), PLAYER_COLORS[i], true))
            .collect();
        let mut g = Game::new(players, crate::board::default_board());
        // Seen from the board, not from the table: the turn drives who folds.
        g.turn = 2;
        g.declare_bankrupt();
        g.turn = 1;
        g.declare_bankrupt();
        assert_eq!(g.standings(), vec![0, 1, 2], "first out ranks last");
        assert!(g.players[2].bankrupt && g.players[1].bankrupt);
    }
}
