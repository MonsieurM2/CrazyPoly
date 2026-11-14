//! Static definition of the classic 40-space Monopoly board.
//!
//! Spaces are stored in playing order, starting at GO (index 0) and running
//! counter-clockwise when seen from the players' side of the board:
//! bottom edge (right -> left), left edge (bottom -> top), top edge
//! (left -> right), right edge (top -> bottom).

use macroquad::prelude::*;

/// Colour group of a space. Street groups are the eight purchasable colour
/// bands; Railroad and Utility are their own groups for rent purposes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    DarkBlue,
    Railroad,
    Utility,
    Special,
}

impl Group {
    /// Official board colour for the group band.
    pub fn color(self) -> Color {
        match self {
            Group::Brown => Color::from_rgba(149, 84, 54, 255),
            Group::LightBlue => Color::from_rgba(170, 224, 250, 255),
            Group::Pink => Color::from_rgba(217, 49, 150, 255),
            Group::Orange => Color::from_rgba(247, 148, 29, 255),
            Group::Red => Color::from_rgba(237, 27, 36, 255),
            Group::Yellow => Color::from_rgba(254, 242, 0, 255),
            Group::Green => Color::from_rgba(0, 166, 81, 255),
            Group::DarkBlue => Color::from_rgba(0, 114, 188, 255),
            Group::Railroad => Color::from_rgba(52, 52, 52, 255),
            Group::Utility => Color::from_rgba(120, 120, 120, 255),
            Group::Special => Color::from_rgba(214, 209, 192, 255),
        }
    }

    pub fn street(self) -> bool {
        matches!(
            self,
            Group::Brown
                | Group::LightBlue
                | Group::Pink
                | Group::Orange
                | Group::Red
                | Group::Yellow
                | Group::Green
                | Group::DarkBlue
        )
    }

    pub fn title(self) -> &'static str {
        match self {
            Group::Brown => "Brown",
            Group::LightBlue => "Light Blue",
            Group::Pink => "Pink",
            Group::Orange => "Orange",
            Group::Red => "Red",
            Group::Yellow => "Yellow",
            Group::Green => "Green",
            Group::DarkBlue => "Dark Blue",
            Group::Railroad => "Railroad",
            Group::Utility => "Utility",
            Group::Special => "",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Go,
    Street,
    Chest,
    Tax,
    Railroad,
    Chance,
    Jail,
    Utility,
    FreeParking,
    GoToJail,
}

#[derive(Clone)]
pub struct Space {
    pub name: &'static str,
    pub kind: Kind,
    pub group: Group,
    pub price: u32,
    /// [base rent, 1 house, 2 houses, 3 houses, 4 houses, hotel]
    pub rent: [u32; 6],
    pub house_cost: u32,
    pub tax: u32,
}

impl Space {
    pub fn purchasable(&self) -> bool {
        matches!(self.kind, Kind::Street | Kind::Railroad | Kind::Utility)
    }

    pub fn corner(&self) -> bool {
        matches!(
            self.kind,
            Kind::Go | Kind::Jail | Kind::FreeParking | Kind::GoToJail
        )
    }

    /// Colour band drawn on the tile (streets only).
    pub fn has_band(&self) -> bool {
        self.kind == Kind::Street
    }
}

fn street(
    name: &'static str,
    group: Group,
    price: u32,
    rent: [u32; 6],
    house_cost: u32,
) -> Space {
    Space {
        name,
        kind: Kind::Street,
        group,
        price,
        rent,
        house_cost,
        tax: 0,
    }
}

fn corner(name: &'static str, kind: Kind) -> Space {
    Space {
        name,
        kind,
        group: Group::Special,
        price: 0,
        rent: [0; 6],
        house_cost: 0,
        tax: 0,
    }
}

fn chance() -> Space {
    corner("Chance", Kind::Chance)
}

fn chest() -> Space {
    corner("Community Chest", Kind::Chest)
}

fn tax(name: &'static str, amount: u32) -> Space {
    Space {
        name,
        kind: Kind::Tax,
        group: Group::Special,
        price: 0,
        rent: [0; 6],
        house_cost: 0,
        tax: amount,
    }
}

fn railroad(name: &'static str) -> Space {
    Space {
        name,
        kind: Kind::Railroad,
        group: Group::Railroad,
        price: 200,
        rent: [25, 50, 100, 200, 0, 0],
        house_cost: 0,
        tax: 0,
    }
}

fn utility(name: &'static str) -> Space {
    Space {
        name,
        kind: Kind::Utility,
        group: Group::Utility,
        price: 150,
        rent: [0; 6],
        house_cost: 0,
        tax: 0,
    }
}

/// The official 40-space layout of the classic US Monopoly board.
pub fn spaces() -> Vec<Space> {
    vec![
        corner("GO", Kind::Go),
        street(
            "Mediterranean Avenue",
            Group::Brown,
            60,
            [2, 10, 30, 90, 160, 250],
            50,
        ),
        chest(),
        street(
            "Baltic Avenue",
            Group::Brown,
            60,
            [4, 20, 60, 180, 320, 450],
            50,
        ),
        tax("Income Tax", 200),
        railroad("Reading Railroad"),
        street(
            "Oriental Avenue",
            Group::LightBlue,
            100,
            [6, 30, 90, 270, 400, 550],
            50,
        ),
        chance(),
        street(
            "Vermont Avenue",
            Group::LightBlue,
            100,
            [6, 30, 90, 270, 400, 550],
            50,
        ),
        street(
            "Connecticut Avenue",
            Group::LightBlue,
            120,
            [8, 40, 100, 300, 450, 600],
            50,
        ),
        corner("JAIL", Kind::Jail),
        street(
            "St. Charles Place",
            Group::Pink,
            140,
            [10, 50, 150, 450, 625, 750],
            100,
        ),
        utility("Electric Company"),
        street(
            "States Avenue",
            Group::Pink,
            140,
            [10, 50, 150, 450, 625, 750],
            100,
        ),
        street(
            "Virginia Avenue",
            Group::Pink,
            160,
            [12, 60, 180, 500, 700, 900],
            100,
        ),
        railroad("Pennsylvania Railroad"),
        street(
            "St. James Place",
            Group::Orange,
            180,
            [14, 70, 200, 550, 750, 950],
            100,
        ),
        chest(),
        street(
            "Tennessee Avenue",
            Group::Orange,
            180,
            [14, 70, 200, 550, 750, 950],
            100,
        ),
        street(
            "New York Avenue",
            Group::Orange,
            200,
            [16, 80, 220, 600, 800, 1000],
            100,
        ),
        corner("FREE PARKING", Kind::FreeParking),
        street(
            "Kentucky Avenue",
            Group::Red,
            220,
            [18, 90, 250, 700, 875, 1050],
            150,
        ),
        chance(),
        street(
            "Indiana Avenue",
            Group::Red,
            220,
            [18, 90, 250, 700, 875, 1050],
            150,
        ),
        street(
            "Illinois Avenue",
            Group::Red,
            240,
            [20, 100, 300, 750, 925, 1100],
            150,
        ),
        railroad("B. & O. Railroad"),
        street(
            "Atlantic Avenue",
            Group::Yellow,
            260,
            [22, 110, 330, 800, 975, 1150],
            150,
        ),
        street(
            "Ventnor Avenue",
            Group::Yellow,
            260,
            [22, 110, 330, 800, 975, 1150],
            150,
        ),
        utility("Water Works"),
        street(
            "Marvin Gardens",
            Group::Yellow,
            280,
            [24, 120, 360, 850, 1025, 1200],
            150,
        ),
        corner("GO TO JAIL", Kind::GoToJail),
        street(
            "Pacific Avenue",
            Group::Green,
            300,
            [26, 130, 390, 900, 1100, 1275],
            200,
        ),
        street(
            "North Carolina Avenue",
            Group::Green,
            300,
            [26, 130, 390, 900, 1100, 1275],
            200,
        ),
        chest(),
        street(
            "Pennsylvania Avenue",
            Group::Green,
            320,
            [28, 150, 450, 1000, 1200, 1400],
            200,
        ),
        railroad("Short Line"),
        chance(),
        street(
            "Park Place",
            Group::DarkBlue,
            350,
            [35, 175, 500, 1100, 1300, 1500],
            200,
        ),
        tax("Luxury Tax", 100),
        street(
            "Boardwalk",
            Group::DarkBlue,
            400,
            [50, 200, 600, 1400, 1700, 2000],
            200,
        ),
    ]
}
