//! Chance and Community Chest decks, in English and French.
//!
//! A card carries both printed texts plus a machine-readable effect, so the
//! engine can resolve it without re-parsing strings. Decks are shuffled at
//! startup with a Fisher-Yates pass over macroquad's RNG and cycle back to the
//! top of the deck once exhausted (as in the physical game).

use macroquad::rand::gen_range;

use crate::i18n;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    AdvanceToGo,
    AdvanceTo(usize),
    NearestUtility,
    NearestRailroad,
    Back(usize),
    GoToJail,
    JailFree,
    Collect(i64),
    Pay(i64),
    CollectFromEach(i64),
    PayEach(i64),
    Repairs { house: i64, hotel: i64 },
}

#[derive(Clone, Copy)]
pub struct Card {
    pub en: &'static str,
    pub fr: &'static str,
    pub effect: Effect,
}

impl Card {
    pub fn text(&self) -> &'static str {
        i18n::t(self.en, self.fr)
    }
}

pub struct Deck {
    pub cards: Vec<Card>,
    pub idx: usize,
    pub label: &'static str,
}

impl Deck {
    fn new(label: &'static str, cards: Vec<Card>) -> Self {
        let mut deck = Deck {
            cards,
            idx: 0,
            label,
        };
        deck.shuffle();
        deck
    }

    /// Rebuild a deck that was saved mid-session (used by the save loader).
    pub fn from_cards(label: &'static str, cards: Vec<Card>, idx: usize) -> Self {
        let idx = idx.min(cards.len());
        Deck { cards, idx, label }
    }

    fn shuffle(&mut self) {
        if self.cards.len() < 2 {
            return;
        }
        for i in (1..self.cards.len()).rev() {
            let j = gen_range(0, (i + 1) as i32) as usize;
            self.cards.swap(i, j);
        }
        self.idx = 0;
    }

    /// Draw the top card, recycling the deck when it runs out.
    pub fn draw(&mut self) -> Card {
        if self.cards.is_empty() {
            return Card {
                en: "The deck is empty.",
                fr: "Le paquet est vide.",
                effect: Effect::Collect(0),
            };
        }
        if self.idx >= self.cards.len() {
            self.shuffle();
        }
        let card = self.cards[self.idx];
        self.idx += 1;
        card
    }

    pub fn remaining(&self) -> usize {
        self.cards.len().saturating_sub(self.idx)
    }
}

fn card(en: &'static str, fr: &'static str, effect: Effect) -> Card {
    Card { en, fr, effect }
}

pub fn chance_cards() -> Vec<Card> {
    use Effect::*;
    vec![
        card(
            "Advance to GO. Collect $200.",
            "Avancez jusqu'à la case DÉPART. Recevez 200 $.",
            AdvanceToGo,
        ),
        card(
            "Advance to Illinois Avenue. If you pass GO, collect $200.",
            "Avancez jusqu'à l'avenue Illinois. Si vous passez par la case DÉPART, recevez 200 $.",
            AdvanceTo(24),
        ),
        card(
            "Advance to St. Charles Place. If you pass GO, collect $200.",
            "Avancez jusqu'à la place Saint-Charles. Si vous passez par la case DÉPART, recevez 200 $.",
            AdvanceTo(11),
        ),
        card(
            "Advance to the nearest Utility. If unowned you may buy it. If owned, throw dice and pay the owner ten times the amount thrown.",
            "Avancez jusqu'à la compagnie de service public la plus proche. Si elle est libre, vous pouvez l'acheter ; sinon, lancez les dés et payez au propriétaire dix fois le montant obtenu.",
            NearestUtility,
        ),
        card(
            "Advance to the nearest Railroad. Pay the owner twice the rental to which they are otherwise entitled.",
            "Avancez jusqu'à la gare la plus proche. Payez au propriétaire le double du loyer habituel.",
            NearestRailroad,
        ),
        card(
            "Bank pays you dividend of $50.",
            "La banque vous verse un dividende de 50 $.",
            Collect(50),
        ),
        card(
            "Get out of Jail Free. This card may be kept until needed.",
            "Sortez de prison gratuitement. Cette carte peut être conservée jusqu'à son utilisation.",
            JailFree,
        ),
        card(
            "Go back 3 spaces.",
            "Reculez de trois cases.",
            Back(3),
        ),
        card(
            "Go to Jail. Go directly to Jail. Do not pass GO, do not collect $200.",
            "Allez en prison. Rendez-vous directement en prison sans passer par la case DÉPART et sans recevoir 200 $.",
            GoToJail,
        ),
        card(
            "Make general repairs on all your property: $25 per house, $100 per hotel.",
            "Faites des réparations sur tous vos biens : 25 $ par maison, 100 $ par hôtel.",
            Repairs {
                house: 25,
                hotel: 100,
            },
        ),
        card(
            "Speeding fine. Pay $15.",
            "Amende pour excès de vitesse : payez 15 $.",
            Pay(15),
        ),
        card(
            "Take a trip to Reading Railroad. If you pass GO, collect $200.",
            "Faites un voyage jusqu'à la gare de Reading. Si vous passez par la case DÉPART, recevez 200 $.",
            AdvanceTo(5),
        ),
        card(
            "Advance to Boardwalk.",
            "Avancez jusqu'au Boardwalk.",
            AdvanceTo(39),
        ),
        card(
            "You have been elected Chairman of the Board. Pay each player $50.",
            "Vous êtes élu président du conseil. Payez 50 $ à chaque joueur.",
            PayEach(50),
        ),
        card(
            "Your building loan matures. Collect $150.",
            "Votre emprunt de construction arrive à échéance. Recevez 150 $.",
            Collect(150),
        ),
        card(
            "Advance to the nearest Railroad and pay double rent.",
            "Avancez jusqu'à la gare la plus proche et payez le double du loyer.",
            NearestRailroad,
        ),
    ]
}

pub fn chest_cards() -> Vec<Card> {
    use Effect::*;
    vec![
        card(
            "Advance to GO. Collect $200.",
            "Avancez jusqu'à la case DÉPART. Recevez 200 $.",
            AdvanceToGo,
        ),
        card(
            "Bank error in your favour. Collect $200.",
            "Erreur de la banque en votre faveur. Recevez 200 $.",
            Collect(200),
        ),
        card(
            "Doctor's fee. Pay $50.",
            "Frais de médecin : payez 50 $.",
            Pay(50),
        ),
        card(
            "From sale of stock you get $50.",
            "Vente de titres : recevez 50 $.",
            Collect(50),
        ),
        card(
            "Get out of Jail Free. This card may be kept until needed.",
            "Sortez de prison gratuitement. Cette carte peut être conservée jusqu'à son utilisation.",
            JailFree,
        ),
        card(
            "Go to Jail. Go directly to Jail. Do not pass GO, do not collect $200.",
            "Allez en prison. Rendez-vous directement en prison sans passer par la case DÉPART et sans recevoir 200 $.",
            GoToJail,
        ),
        card(
            "Holiday fund matures. Receive $100.",
            "Votre caisse de vacances est échue. Recevez 100 $.",
            Collect(100),
        ),
        card(
            "Income tax refund. Collect $20.",
            "Remboursement d'impôt sur le revenu : recevez 20 $.",
            Collect(20),
        ),
        card(
            "It is your birthday. Collect $10 from every player.",
            "C'est votre anniversaire. Recevez 10 $ de chaque joueur.",
            CollectFromEach(10),
        ),
        card(
            "Life insurance matures. Collect $100.",
            "Votre assurance-vie est échue. Recevez 100 $.",
            Collect(100),
        ),
        card(
            "Pay hospital fees of $100.",
            "Frais d'hôpital : payez 100 $.",
            Pay(100),
        ),
        card(
            "Pay school fees of $50.",
            "Frais de scolarité : payez 50 $.",
            Pay(50),
        ),
        card(
            "Receive $25 consultancy fee.",
            "Honoraires de conseil : recevez 25 $.",
            Collect(25),
        ),
        card(
            "You are assessed for street repairs: $40 per house, $115 per hotel.",
            "Réparations de voirie : 40 $ par maison, 115 $ par hôtel.",
            Repairs {
                house: 40,
                hotel: 115,
            },
        ),
        card(
            "You have won second prize in a beauty contest. Collect $10.",
            "Vous avez remporté le deuxième prix au concours de beauté. Recevez 10 $.",
            Collect(10),
        ),
        card(
            "You inherit $100.",
            "Vous héritez de 100 $.",
            Collect(100),
        ),
    ]
}

pub fn chance_deck() -> Deck {
    Deck::new("CHANCE", chance_cards())
}

pub fn chest_deck() -> Deck {
    Deck::new("COMMUNITY CHEST", chest_cards())
}
