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

    /// The printed sentence, naming the player who drew the card.
    pub fn sentence(&self, who: &str) -> String {
        self.text().replacen("{}", who, 1)
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
            "{} advances to GO and collects $200.",
            "{} avance jusqu'à la case DÉPART et reçoit 200 $.",
            AdvanceToGo,
        ),
        card(
            "{} advances to Illinois Avenue, collecting $200 if GO is passed.",
            "{} avance jusqu'à l'avenue Illinois et reçoit 200 $ en passant par la case DÉPART.",
            AdvanceTo(24),
        ),
        card(
            "{} advances to St. Charles Place, collecting $200 if GO is passed.",
            "{} avance jusqu'à la place Saint-Charles et reçoit 200 $ en passant par la case DÉPART.",
            AdvanceTo(11),
        ),
        card(
            "{} advances to the nearest utility: free to buy if unowned, otherwise the dice are thrown and ten times the roll goes to its owner.",
            "{} avance jusqu'à la compagnie de service public la plus proche : libre, elle peut être achetée ; sinon les dés sont lancés et dix fois le montant obtenu revient au propriétaire.",
            NearestUtility,
        ),
        card(
            "{} advances to the nearest station and pays its owner twice the usual rent.",
            "{} avance jusqu'à la gare la plus proche et paie au propriétaire le double du loyer habituel.",
            NearestRailroad,
        ),
        card(
            "The bank pays {} a dividend of $50.",
            "La banque verse à {} un dividende de 50 $.",
            Collect(50),
        ),
        card(
            "{} draws a Get Out of Jail Free card, kept until it is needed.",
            "{} obtient une carte « Sortez de prison », conservée jusqu'à son utilisation.",
            JailFree,
        ),
        card(
            "{} goes back three spaces.",
            "{} recule de trois cases.",
            Back(3),
        ),
        card(
            "{} goes to Jail directly, without passing GO and without collecting $200.",
            "{} va directement en prison, sans passer par la case DÉPART et sans recevoir 200 $.",
            GoToJail,
        ),
        card(
            "{} pays for repairs on every building: $25 per house, $100 per hotel.",
            "{} paie des réparations sur toutes ses constructions : 25 $ par maison, 100 $ par hôtel.",
            Repairs {
                house: 25,
                hotel: 100,
            },
        ),
        card(
            "{} is fined $15 for speeding.",
            "{} reçoit une amende de 15 $ pour excès de vitesse.",
            Pay(15),
        ),
        card(
            "{} travels to Reading Station, collecting $200 if GO is passed.",
            "{} fait un voyage jusqu'à la gare de Reading et reçoit 200 $ en passant par la case DÉPART.",
            AdvanceTo(5),
        ),
        card(
            "{} advances to Boardwalk.",
            "{} avance jusqu'au Boardwalk.",
            AdvanceTo(39),
        ),
        card(
            "{} is elected Chairman of the Board and pays each player $50.",
            "{} est élu président du conseil et paie 50 $ à chaque joueur.",
            PayEach(50),
        ),
        card(
            "The building loan of {} matures: $150 collected.",
            "L'emprunt de construction de {} arrive à échéance : 150 $ reçus.",
            Collect(150),
        ),
        card(
            "{} advances to the nearest station and pays double rent.",
            "{} avance jusqu'à la gare la plus proche et paie le double du loyer.",
            NearestRailroad,
        ),
    ]
}

pub fn chest_cards() -> Vec<Card> {
    use Effect::*;
    vec![
        card(
            "{} advances to GO and collects $200.",
            "{} avance jusqu'à la case DÉPART et reçoit 200 $.",
            AdvanceToGo,
        ),
        card(
            "A bank error in the favour of {}: $200 collected.",
            "Erreur de la banque en faveur de {} : 200 $ reçus.",
            Collect(200),
        ),
        card(
            "{} pays the doctor's fee of $50.",
            "{} paie 50 $ de frais de médecin.",
            Pay(50),
        ),
        card(
            "{} receives $50 from the sale of stock.",
            "{} reçoit 50 $ de la vente de ses titres.",
            Collect(50),
        ),
        card(
            "{} draws a Get Out of Jail Free card, kept until it is needed.",
            "{} obtient une carte « Sortez de prison », conservée jusqu'à son utilisation.",
            JailFree,
        ),
        card(
            "{} goes to Jail directly, without passing GO and without collecting $200.",
            "{} va directement en prison, sans passer par la case DÉPART et sans recevoir 200 $.",
            GoToJail,
        ),
        card(
            "The holiday fund of {} matures: $100 collected.",
            "La caisse de vacances de {} est échue : 100 $ reçus.",
            Collect(100),
        ),
        card(
            "{} receives an income tax refund of $20.",
            "{} reçoit un remboursement d'impôt sur le revenu de 20 $.",
            Collect(20),
        ),
        card(
            "It is the birthday of {}: $10 collected from every player.",
            "C'est l'anniversaire de {} : 10 $ reçus de chaque joueur.",
            CollectFromEach(10),
        ),
        card(
            "The life insurance of {} matures: $100 collected.",
            "L'assurance-vie de {} est échue : 100 $ reçus.",
            Collect(100),
        ),
        card(
            "{} pays hospital fees of $100.",
            "{} paie 100 $ de frais d'hôpital.",
            Pay(100),
        ),
        card(
            "{} pays school fees of $50.",
            "{} paie 50 $ de frais de scolarité.",
            Pay(50),
        ),
        card(
            "{} receives a consultancy fee of $25.",
            "{} reçoit 25 $ d'honoraires de conseil.",
            Collect(25),
        ),
        card(
            "{} pays for street repairs: $40 per house, $115 per hotel.",
            "{} paie des réparations de voirie : 40 $ par maison, 115 $ par hôtel.",
            Repairs {
                house: 40,
                hotel: 115,
            },
        ),
        card(
            "{} wins second prize in a beauty contest: $10.",
            "{} remporte le deuxième prix du concours de beauté : 10 $.",
            Collect(10),
        ),
        card(
            "{} inherits $100.",
            "{} hérite de 100 $.",
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
