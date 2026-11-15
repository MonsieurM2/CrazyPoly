//! Bot players. Each bot draws a temperament at the start of the game - the
//! builder, the banker, the dealer or the gambler - which only shifts the
//! thresholds it uses to buy, build, bid and answer a trade. Nothing shows it
//! on screen unless the debug mode is on.
//!
//! The bots buy sensible lots, bid up to a valuation, build on monopolies,
//! raise cash when they owe money, propose trades for the one lot that
//! completes a colour group, and answer offers made by humans.

use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};

use crate::board::Group;
use crate::game::{Game, Phase, TradeState};

/// How a bot weighs its cash against a lot, a house or a deal.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Strategy {
    /// Buys into colour groups early and builds at the first chance.
    Builder,
    /// Sits on a deep cash cushion, keeps its money, and builds late.
    #[default]
    Banker,
    /// Trades a lot, and accepts near-equal deals to complete a group.
    Dealer,
    /// Spends almost everything, bids high, builds as soon as it can.
    Gambler,
}

/// The numbers behind a [`Strategy`].
pub struct Style {
    /// Share of its cash a bot pays for an ordinary lot.
    share: f64,
    /// Same, for the lot that completes a colour group.
    monopoly_share: f64,
    /// Cash it will not touch, whatever the lot costs.
    floor: i64,
    /// Cash kept aside before it builds.
    cushion: i64,
    /// Houses (or hotels) it puts up per turn.
    houses: usize,
    /// How much more it wants than it gives before accepting a trade.
    margin: f64,
    /// Extra it charges for handing over a lot that completes a group.
    penalty: f64,
    /// Multiplier on its own valuation when bidding in an auction.
    auction: f64,
    /// Cash above which it pays the jail fine instead of rolling for doubles.
    bail: i64,
}

impl Strategy {
    pub const ALL: [Strategy; 4] = [
        Strategy::Builder,
        Strategy::Banker,
        Strategy::Dealer,
        Strategy::Gambler,
    ];

    pub fn style(self) -> Style {
        match self {
            Strategy::Builder => Style {
                share: 0.55,
                monopoly_share: 0.82,
                floor: 70,
                cushion: 130,
                houses: 2,
                margin: 1.08,
                penalty: 300.0,
                auction: 1.00,
                bail: 200,
            },
            Strategy::Banker => Style {
                share: 0.38,
                monopoly_share: 0.72,
                floor: 240,
                cushion: 380,
                houses: 2,
                margin: 1.10,
                penalty: 340.0,
                auction: 1.10,
                bail: 420,
            },
            Strategy::Dealer => Style {
                share: 0.50,
                monopoly_share: 0.78,
                floor: 120,
                cushion: 220,
                houses: 2,
                margin: 0.90,
                penalty: 240.0,
                auction: 1.06,
                bail: 260,
            },
            Strategy::Gambler => Style {
                share: 0.80,
                monopoly_share: 0.96,
                floor: 0,
                cushion: 40,
                houses: 4,
                margin: 1.02,
                penalty: 200.0,
                auction: 1.30,
                bail: 120,
            },
        }
    }

    /// Name shown by the debug mode.
    pub fn label(self) -> &'static str {
        match self {
            Strategy::Builder => crate::i18n::t("Builder", "Bâtisseur"),
            Strategy::Banker => crate::i18n::t("Banker", "Banquier"),
            Strategy::Dealer => crate::i18n::t("Dealer", "Négociateur"),
            Strategy::Gambler => crate::i18n::t("Gambler", "Flambeur"),
        }
    }

    /// A second line for the debug mode: what the temperament actually means.
    pub fn note(self) -> String {
        let style = self.style();
        crate::tr!(
            "keeps ${} · {} house(s)/turn",
            "garde {} $ · {} maison(s)/tour",
            style.floor,
            style.houses
        )
    }
}

fn style(g: &Game, player: usize) -> Style {
    g.players[player].strategy.style()
}

/// Rough worth of a property to a particular player, in dollars.
pub fn property_value(g: &Game, tile: usize, for_player: usize) -> f64 {
    let space = &g.spaces[tile];
    let mut value = space.price as f64;
    if space.group.street() {
        let owned = g.group_owned(for_player, space.group);
        let size = g.group_size(space.group);
        if owned + 1 >= size {
            value *= 1.8;
        } else if owned > 0 {
            value *= 1.25;
        }
        value += 0.6 * g.props[tile].houses as f64 * space.house_cost as f64;
    } else if space.group == Group::Railroad || space.group == Group::Utility {
        value *= 1.1;
    }
    value
}

/// Called by `Game::update` whenever the bot is the one who must act.
pub fn act(g: &mut Game) {
    match g.phase {
        Phase::Roll => {
            let p = g.turn;
            if g.players[p].in_jail {
                if g.players[p].jail_cards > 0 {
                    g.use_jail_card();
                } else if g.players[p].cash > style(g, p).bail {
                    g.pay_jail_fine();
                }
            }
            g.roll();
        }
        Phase::AwaitBuy => {
            let p = g.turn;
            let pos = g.players[p].pos;
            let price = g.spaces[pos].price as i64;
            let cash = g.players[p].cash;
            let group = g.spaces[pos].group;
            let completes = group.street() && g.group_owned(p, group) + 1 >= g.group_size(group);
            let st = style(g, p);
            let share = if completes {
                st.monopoly_share
            } else {
                st.share
            };
            // What it will put on one lot, and what it refuses to touch.
            let cap = (cash as f64 * share) as i64;
            if price > 0 && price <= cap && cash - price >= st.floor {
                g.buy();
            } else {
                g.start_auction_current();
            }
        }
        Phase::Auction => auction_act(g),
        Phase::Debt => debt_act(g),
        Phase::Manage => {
            // Never act while an offer is on the table.
            if g.trade.is_some() {
                return;
            }
            if !g.bot_offer_done && g.bot_trade_offer() {
                return;
            }
            manage_act(g);
            if g.trade.is_none() {
                g.end_turn();
            }
        }
        _ => {}
    }
}

fn auction_act(g: &mut Game) {
    let Some(auction) = g.auction.as_ref() else {
        return;
    };
    let p = auction.turn;
    if !g.players[p].ai {
        return;
    }
    let tile = auction.tile;
    let bid = auction.bid;
    let cash = g.players[p].cash;
    let mut limit = property_value(g, tile, p) * style(g, p).auction;
    if cash > 900 {
        limit *= 1.2;
    }
    limit = limit.min(cash as f64);
    let increment = if limit >= 250.0 { 25 } else { 10 };
    let affordable = cash >= bid + increment;
    let worth_it = (bid + increment) as f64 <= limit;
    if affordable && worth_it {
        g.auction_bid(increment);
    } else {
        g.auction_pass();
    }
}

/// Sell buildings, then mortgage lots, until the debt can be paid.
fn debt_act(g: &mut Game) {
    let p = g.turn;
    let amount = g.debt_amount();
    for _ in 0..300 {
        if g.players[p].cash >= amount {
            break;
        }
        let mut acted = false;

        let mut sellable: Vec<usize> = g
            .owned_tiles(p)
            .into_iter()
            .filter(|t| g.can_sell_house(*t))
            .collect();
        sellable.sort_by(|a, b| g.spaces[*b].house_cost.cmp(&g.spaces[*a].house_cost));
        for tile in sellable {
            if g.sell_house(tile) {
                acted = true;
                break;
            }
        }

        if !acted {
            let mut mortgageable: Vec<usize> = g
                .owned_tiles(p)
                .into_iter()
                .filter(|t| !g.props[*t].mortgaged && g.props[*t].houses == 0)
                .collect();
            mortgageable.sort_by(|a, b| g.spaces[*b].price.cmp(&g.spaces[*a].price));
            for tile in mortgageable {
                if g.mortgage(tile) {
                    acted = true;
                    break;
                }
            }
        }

        if !acted {
            break;
        }
    }

    if g.can_afford_debt() {
        g.pay_debt();
    } else {
        g.declare_bankrupt();
    }
}

/// Build on monopolies while keeping its own cushion.
fn manage_act(g: &mut Game) {
    let p = g.turn;
    let st = style(g, p);
    let strategy = g.players[p].strategy;
    for _ in 0..st.houses {
        if g.players[p].cash <= st.cushion {
            break;
        }
        let candidates: Vec<usize> = (0..40).filter(|t| g.can_build(*t)).collect();
        let Some(best) = candidates.into_iter().max_by(|a, b| {
            // The builder and the banker go for rent per dollar, the dealer
            // for the biggest rent, the gambler for whichever lot is tallest.
            let score = |t: usize| {
                let space = &g.spaces[t];
                let next = space.rent[(g.props[t].houses + 1) as usize] as f64;
                match strategy {
                    Strategy::Builder | Strategy::Banker => next / space.house_cost.max(1) as f64,
                    Strategy::Dealer => next,
                    Strategy::Gambler => next * (1.0 + 0.35 * g.props[t].houses as f64),
                }
            };
            score(*a).partial_cmp(&score(*b)).unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            break;
        };
        let cost = g.spaces[best].house_cost as i64;
        if g.players[p].cash < cost + st.cushion {
            break;
        }
        if !g.build_house(best) {
            break;
        }
    }
}

/// Bot response to a trade (the bot is the recipient, `trade.to`).
pub fn evaluate_trade(g: &Game) -> bool {
    let Some(trade) = &g.trade else {
        return false;
    };
    if trade.state != TradeState::Proposed {
        return false;
    }
    let me = trade.to;
    let opponent = trade.from;
    if me == opponent || !g.players[me].ai {
        return false;
    }

    // The bot receives `give` + `give_cash` and hands over `get` + `get_cash`.
    let gain = g.package_value(&trade.give, trade.give_cash, me);
    let loss = g.package_value(&trade.get, trade.get_cash, me);

    // Refuse to hand the opponent a monopoly, or to break up our own.
    let st = style(g, me);
    let mut penalty = 0.0;
    for tile in &trade.get {
        let group = g.spaces[*tile].group;
        if group.street() && g.group_owned(opponent, group) + 1 >= g.group_size(group) {
            penalty += st.penalty;
        }
        if group.street() && g.has_monopoly(me, group) {
            penalty += st.penalty * 0.7;
        }
    }

    gain >= loss * st.margin + penalty
}

/// Deal the temperaments out: every bot seat gets one, all different while
/// there are four or fewer bots.
pub fn deal_strategies(g: &mut Game) {
    let mut pool = Strategy::ALL;
    for i in 0..pool.len() {
        let j = gen_range(0, pool.len());
        pool.swap(i, j);
    }
    let mut next = 0;
    for player in g.players.iter_mut() {
        if player.ai {
            player.strategy = pool[next % pool.len()];
            next += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Player, PLAYER_COLORS};

    /// A table of four bots, one of each temperament, played out by the AI.
    fn table() -> Game {
        let players = (0..4)
            .map(|i| Player::new(format!("Bot {}", i + 1), PLAYER_COLORS[i], true))
            .collect();
        let mut g = Game::new(players);
        g.fast = true;
        for (i, player) in g.players.iter_mut().enumerate() {
            player.strategy = Strategy::ALL[i];
        }
        g
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "plays dozens of whole games; run with --release")]
    fn every_temperament_can_win() {
        let mut wins = [0usize; Strategy::ALL.len()];
        let mut played = 0;
        for _ in 0..40 {
            let mut g = table();
            for _ in 0..20_000 {
                if g.phase == Phase::Over {
                    break;
                }
                g.update(0.5);
            }
            if let Some(winner) = g.winner {
                wins[winner] += 1;
                played += 1;
            }
        }
        println!("wins {:?} over {} games", wins, played);
        assert!(played >= 28, "only {} of 40 games reached an end", played);
        assert!(
            wins.iter().all(|w| *w > 0),
            "a temperament never won: {:?}",
            wins
        );
    }
}
