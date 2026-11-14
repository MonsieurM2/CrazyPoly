//! Bot players. The AI buys sensible lots, bids in auctions up to a
//! valuation, builds on monopolies, raises cash when it owes money, proposes
//! trades for the one lot that completes a colour group, and answers offers
//! made by humans.

use crate::board::Group;
use crate::game::{Game, Phase, TradeState};

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
                } else if g.players[p].cash > 220 {
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
            let reserve = if completes {
                60
            } else if cash > 800 {
                180
            } else {
                320
            };
            if price > 0 && cash >= price + reserve {
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
    let mut limit = property_value(g, tile, p);
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

/// Build evenly on monopolies while keeping a cash cushion.
fn manage_act(g: &mut Game) {
    let p = g.turn;
    for _ in 0..3 {
        if g.players[p].cash <= 250 {
            break;
        }
        let candidates: Vec<usize> = (0..40).filter(|t| g.can_build(*t)).collect();
        let Some(best) = candidates
            .into_iter()
            .max_by_key(|t| g.spaces[*t].rent[1] + g.spaces[*t].price)
        else {
            break;
        };
        let cost = g.spaces[best].house_cost as i64;
        if g.players[p].cash < cost + 250 {
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
    let mut penalty = 0.0;
    for tile in &trade.get {
        let group = g.spaces[*tile].group;
        if group.street() && g.group_owned(opponent, group) + 1 >= g.group_size(group) {
            penalty += 260.0;
        }
        if group.street() && g.has_monopoly(me, group) {
            penalty += 180.0;
        }
    }

    gain >= loss * 1.06 + penalty
}
