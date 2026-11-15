//! The board: a table of 40 spaces read from a JSON file.
//!
//! Two boards ship with the game, the classic English one and a French one,
//! both stored under `assets/boards/`. Anything else can be imported at run
//! time: the same file format describes the names, prices, colours and order
//! of every space, so a player can build their own table without touching the
//! code. See `README.md` for the format.
//!
//! Spaces are stored in playing order, starting at GO (index 0) and running
//! counter-clockwise when seen from the players' side of the board:
//! bottom edge (right -> left), left edge (bottom -> top), top edge
//! (left -> right), right edge (top -> bottom).

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

/// Number of spaces on a board: the layout is built for exactly this many.
pub const SPACES: usize = 40;

/// Colour group of a space. Streets carry the index of their colour band in
/// the board; Railroad and Utility are their own groups for rent purposes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Street(u8),
    Railroad,
    Utility,
    Special,
}

impl Group {
    pub fn street(self) -> bool {
        matches!(self, Group::Street(_))
    }

    /// Index of the band in the board's colour table, for a street.
    pub fn band(self) -> Option<u8> {
        match self {
            Group::Street(i) => Some(i),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

impl Kind {
    pub fn code(self) -> &'static str {
        match self {
            Kind::Go => "go",
            Kind::Street => "street",
            Kind::Chest => "chest",
            Kind::Tax => "tax",
            Kind::Railroad => "railroad",
            Kind::Chance => "chance",
            Kind::Jail => "jail",
            Kind::Utility => "utility",
            Kind::FreeParking => "free_parking",
            Kind::GoToJail => "go_to_jail",
        }
    }
}

#[derive(Clone)]
pub struct Space {
    pub name: String,
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

/// One colour band: its name (for the deed and the log) and its colour.
#[derive(Clone)]
pub struct ColorGroup {
    pub name: String,
    pub color: Color,
}

/// A whole board, ready to play.
#[derive(Clone)]
pub struct Board {
    pub name: String,
    /// `Some("en")` or `Some("fr")` for the boards shipped with the game,
    /// `None` once a board has been imported: an imported board keeps its own
    /// names whatever the interface language is.
    pub language: Option<String>,
    pub groups: Vec<ColorGroup>,
    pub spaces: Vec<Space>,
}

impl Board {
    pub fn color(&self, group: Group) -> Color {
        match group.band() {
            Some(i) => self
                .groups
                .get(i as usize)
                .map(|g| g.color)
                .unwrap_or(Color::from_rgba(200, 200, 200, 255)),
            None => match group {
                Group::Railroad => Color::from_rgba(52, 52, 52, 255),
                Group::Utility => Color::from_rgba(120, 120, 120, 255),
                _ => Color::from_rgba(214, 209, 192, 255),
            },
        }
    }

    pub fn group_title(&self, group: Group) -> &str {
        match group.band() {
            Some(i) => self
                .groups
                .get(i as usize)
                .map(|g| g.name.as_str())
                .unwrap_or(""),
            None => match group {
                Group::Railroad => crate::i18n::t("Railroad", "Chemin de fer"),
                Group::Utility => crate::i18n::t("Utility", "Service"),
                _ => "",
            },
        }
    }

    /// How many spaces the group holds.
    pub fn group_size(&self, group: Group) -> usize {
        self.spaces.iter().filter(|s| s.group == group).count()
    }

    pub fn group_tiles(&self, group: Group) -> Vec<usize> {
        (0..self.spaces.len())
            .filter(|i| self.spaces[*i].group == group)
            .collect()
    }

    /// Every street group of the board, in the order of its colour table.
    pub fn street_groups(&self) -> Vec<Group> {
        (0..self.groups.len()).map(|i| Group::Street(i as u8)).collect()
    }

    /// The file that would reproduce this board.
    pub fn to_file(&self) -> BoardFile {
        BoardFile {
            name: self.name.clone(),
            language: self.language.clone(),
            groups: self
                .groups
                .iter()
                .map(|g| GroupFile {
                    name: g.name.clone(),
                    color: hex_of(g.color),
                })
                .collect(),
            spaces: self
                .spaces
                .iter()
                .map(|s| SpaceFile {
                    kind: s.kind,
                    name: s.name.clone(),
                    group: s.group.band(),
                    price: s.price,
                    rent: s.rent,
                    house: s.house_cost,
                    tax: s.tax,
                })
                .collect(),
        }
    }

    /// Reads and checks a board file. Every problem is reported in words the
    /// player can act on, since the file may come from anywhere.
    pub fn parse(text: &str) -> Result<Board, String> {
        let file: BoardFile = serde_json::from_str(text)
            .map_err(|e| crate::tr!("Not a board file: {}", "Ce n'est pas un plateau : {}", e))?;
        Board::from_file(file)
    }

    pub fn from_file(file: BoardFile) -> Result<Board, String> {
        let name = file.name.trim().to_string();
        if name.is_empty() {
            return Err(crate::i18n::t(
                "The board has no name.",
                "Le plateau n'a pas de nom.",
            )
            .to_string());
        }
        if file.spaces.len() != SPACES {
            return Err(crate::tr!(
                "A board needs exactly {} spaces, this one has {}.",
                "Un plateau compte exactement {} cases, celui-ci en a {}.",
                SPACES,
                file.spaces.len()
            ));
        }
        let mut groups = Vec::with_capacity(file.groups.len());
        for (i, g) in file.groups.iter().enumerate() {
            let color = parse_hex(&g.color).ok_or_else(|| {
                crate::tr!(
                    "Colour {} of group {} is not a #rrggbb value.",
                    "La couleur {} du groupe {} n'est pas au format #rrggbb.",
                    g.color,
                    i + 1
                )
            })?;
            groups.push(ColorGroup {
                name: g.name.trim().to_string(),
                color,
            });
        }

        // The four corners anchor the layout: GO, jail, free parking, go to jail.
        let corners = [Kind::Go, Kind::Jail, Kind::FreeParking, Kind::GoToJail];
        for (i, want) in corners.iter().enumerate() {
            let at = i * 10;
            let got = file.spaces[at].kind;
            if *want != got {
                return Err(crate::tr!(
                    "Space {} must be {}, not {}.",
                    "La case {} doit être {}, pas {}.",
                    at,
                    corner_label(*want),
                    corner_label(got)
                ));
            }
        }

        let mut spaces = Vec::with_capacity(SPACES);
        for (i, s) in file.spaces.iter().enumerate() {
            let name = s.name.trim().to_string();
            if name.is_empty() {
                return Err(crate::tr!(
                    "Space {} has no name.",
                    "La case {} n'a pas de nom.",
                    i
                ));
            }
            let group = match s.kind {
                Kind::Street => {
                    let band = s.group.ok_or_else(|| {
                        crate::tr!(
                            "Street {} ({}) has no colour group.",
                            "La rue {} ({}) n'a pas de groupe de couleur.",
                            i,
                            name
                        )
                    })?;
                    if band as usize >= groups.len() {
                        return Err(crate::tr!(
                            "Street {} ({}) points at colour group {}, which does not exist.",
                            "La rue {} ({}) renvoie au groupe de couleur {}, qui n'existe pas.",
                            i,
                            name,
                            band + 1
                        ));
                    }
                    Group::Street(band)
                }
                Kind::Railroad => Group::Railroad,
                Kind::Utility => Group::Utility,
                _ => Group::Special,
            };
            spaces.push(Space {
                name,
                kind: s.kind,
                group,
                price: s.price,
                rent: s.rent,
                house_cost: s.house,
                tax: s.tax,
            });
        }

        Ok(Board {
            name,
            language: file.language.map(|l| l.to_ascii_lowercase()),
            groups,
            spaces,
        })
    }
}

fn corner_label(kind: Kind) -> &'static str {
    match kind {
        Kind::Go => "go",
        Kind::Jail => "jail",
        Kind::FreeParking => "free_parking",
        Kind::GoToJail => "go_to_jail",
        _ => "another kind",
    }
}

/// `#rrggbb` to a colour.
fn parse_hex(text: &str) -> Option<Color> {
    let t = text.trim().trim_start_matches('#');
    if t.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&t[0..2], 16).ok()?;
    let g = u8::from_str_radix(&t[2..4], 16).ok()?;
    let b = u8::from_str_radix(&t[4..6], 16).ok()?;
    Some(Color::from_rgba(r, g, b, 255))
}

fn hex_of(color: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8
    )
}

// ------------------------------------------------------------------- files

#[derive(Serialize, Deserialize)]
pub struct BoardFile {
    pub name: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub groups: Vec<GroupFile>,
    pub spaces: Vec<SpaceFile>,
}

#[derive(Serialize, Deserialize)]
pub struct GroupFile {
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Deserialize)]
pub struct SpaceFile {
    pub kind: Kind,
    pub name: String,
    #[serde(default)]
    pub group: Option<u8>,
    #[serde(default)]
    pub price: u32,
    #[serde(default)]
    pub rent: [u32; 6],
    #[serde(default)]
    pub house: u32,
    #[serde(default)]
    pub tax: u32,
}

/// The two boards that ship with the game.
const EN_BOARD: &str = include_str!("../assets/boards/classic-en.json");
const FR_BOARD: &str = include_str!("../assets/boards/classic-fr.json");

pub fn builtin(language: &str) -> Option<Board> {
    let text = match language {
        "en" => EN_BOARD,
        "fr" => FR_BOARD,
        _ => return None,
    };
    Board::parse(text).ok()
}

/// Board used when nothing else is chosen.
pub fn default_board() -> Board {
    builtin(crate::i18n::code().to_ascii_lowercase().as_str())
        .or_else(|| builtin("en"))
        .expect("the boards shipped with the game must parse")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_board_survives_a_round_trip_through_a_save() {
        let french = builtin("fr").expect("the French board parses");
        let json = serde_json::to_string(&french.to_file()).expect("a board serialises");
        let back = Board::parse(&json).expect("a written board reads back");
        assert_eq!(back.name, french.name);
        assert_eq!(back.language.as_deref(), Some("fr"));
        assert_eq!(back.spaces.len(), SPACES);
        assert_eq!(back.groups.len(), french.groups.len());
        for (a, b) in french.spaces.iter().zip(&back.spaces) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.group, b.group);
            assert_eq!(a.price, b.price);
            assert_eq!(a.rent, b.rent);
            assert_eq!(a.house_cost, b.house_cost);
            assert_eq!(a.tax, b.tax);
        }
    }
}
