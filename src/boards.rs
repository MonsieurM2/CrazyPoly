//! Board files on disk: the two boards shipped with the game, plus the tables
//! the player imports.
//!
//! Imported boards are copied (and re-written in a normalised form) into
//! `boards/` next to the saves, so a board picked once is offered every time
//! the game starts. The shipped ones live inside the executable and can never
//! be overwritten or deleted.

use std::fs;
use std::path::{Path, PathBuf};

use crate::board::{self, Board};

/// Id of the English classic board: the fallback whenever no other id fits.
pub const EN: &str = "classic-en";

/// One board offered in the picker.
#[derive(Clone)]
pub struct Entry {
    pub id: String,
    pub name: String,
    /// `Some("en")`/`Some("fr")` for a shipped board, `None` once imported.
    pub language: Option<String>,
    pub builtin: bool,
}

impl Entry {
    /// Small tag drawn next to the name in the list.
    pub fn tag(&self) -> String {
        let origin = if self.builtin {
            crate::i18n::t("included", "inclus")
        } else {
            crate::i18n::t("imported", "importé")
        };
        match &self.language {
            Some(lang) => format!("{} · {}", lang.to_ascii_uppercase(), origin),
            None => origin.to_string(),
        }
    }
}

pub fn dir() -> PathBuf {
    crate::persist::save_dir().join("boards")
}

/// Where a shipped board lives in the binary.
fn builtin_board(language: &str) -> Option<Board> {
    board::builtin(language)
}

pub fn id_of(board: &Board) -> String {
    for id in board::builtin_ids() {
        if board::builtin_by_id(id).map(|b| b.name == board.name).unwrap_or(false) {
            return id.to_string();
        }
    }
    EN.to_string()
}

/// Every board the player can choose from, shipped ones first.
pub fn list() -> Vec<Entry> {
    let mut out = Vec::new();
    for id in board::builtin_ids() {
        if let Some(b) = board::builtin_by_id(id) {
            out.push(Entry {
                id: id.to_string(),
                name: b.name,
                language: b.language,
                builtin: true,
            });
        }
    }
    let mut custom: Vec<Entry> = Vec::new();
    if let Ok(read) = fs::read_dir(dir()) {
        for item in read.flatten() {
            let path = item.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if id.is_empty() {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(b) = Board::parse(&text) {
                custom.push(Entry {
                    id: id.to_string(),
                    name: b.name,
                    language: None,
                    builtin: false,
                });
            }
        }
    }
    custom.sort_by_key(|e| e.name.to_lowercase());
    out.extend(custom);
    out
}

pub fn load(id: &str) -> Result<Board, String> {
    if let Some(board) = board::builtin_by_id(id) {
        return Ok(board);
    }
    let path = dir().join(format!("{id}.json"));
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
    Board::parse(&text)
}

/// Copies a board file into the game's folder. Returns the new entry.
pub fn import(path: &Path) -> Result<Entry, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let parsed = Board::parse(&text)?;
    let dir = dir();
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {}", dir.display(), e))?;

    let base = slug(&parsed.name);
    let mut id = base.clone();
    let mut n = 2;
    while dir.join(format!("{id}.json")).exists() {
        id = format!("{base}-{n}");
        n += 1;
    }

    // Imported boards never follow the interface language: their names are
    // the player's own, whatever language the interface is in.
    let mut file = parsed.to_file();
    file.language = None;
    let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    let target = dir.join(format!("{id}.json"));
    fs::write(&target, json).map_err(|e| format!("{}: {}", target.display(), e))?;

    Ok(Entry {
        id,
        name: parsed.name,
        language: None,
        builtin: false,
    })
}

/// Removes an imported board. The shipped ones are refused.
pub fn delete(id: &str) -> Result<(), String> {
    if board::builtin_by_id(id).is_some() {
        return Err(crate::i18n::t(
            "The boards included with the game cannot be deleted.",
            "Les plateaux fournis avec le jeu ne peuvent pas être supprimés.",
        )
        .to_string());
    }
    let path = dir().join(format!("{id}.json"));
    if !path.exists() {
        return Err(crate::tr!(
            "This board is no longer there ({}).",
            "Ce plateau n'est plus là ({}).",
            path.display()
        ));
    }
    fs::remove_file(&path).map_err(|e| format!("{}: {}", path.display(), e))
}

/// A file name for a board: lower-case letters, digits and dashes.
fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
        if out.len() >= 40 {
            break;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "board".to_string()
    } else {
        out
    }
}

/// When the interface language changes, a shipped board gives way to the
/// shipped board of the new language. An imported board keeps its own names
/// and stays selected.
pub fn following_language(current: &Board) -> Option<Board> {
    // A themed or imported board keeps its own names whatever the language.
    if !current.classic {
        return None;
    }
    let current_language = current.language.as_deref()?;
    let wanted = crate::i18n::code().to_ascii_lowercase();
    if current_language == wanted {
        return None;
    }
    builtin_board(&wanted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_file_names() {
        assert_eq!(slug("My Board!"), "my-board");
        assert_eq!(slug("  "), "board");
        assert_eq!(slug("Plateau d'été 2026"), "plateau-d-t-2026");
    }

    #[test]
    fn the_shipped_boards_have_the_classic_layout() {
        let en = board::builtin("en").unwrap();
        let fr = board::builtin("fr").unwrap();
        assert_eq!(en.spaces.len(), board::SPACES);
        assert_eq!(fr.spaces.len(), board::SPACES);
        assert_eq!(en.groups.len(), 8);
        for (i, (a, b)) in en.spaces.iter().zip(&fr.spaces).enumerate() {
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.price, b.price);
            assert_eq!(a.group, b.group);
            assert_eq!(a.rent, b.rent);
            // "Chance" is spelled the same in both languages; the streets,
            // corners and taxes are renamed.
            if a.kind != crate::board::Kind::Chance {
                assert_ne!(a.name, b.name, "space {} keeps its English name", i);
            }
        }
    }

    #[test]
    fn a_board_being_imported_must_be_whole() {
        let text = std::fs::read_to_string("assets/boards/classic-en.json").unwrap();
        // Drop the last space: 39 spaces is not a board.
        let short = text.replacen(
            r#"{ "kind": "street", "name": "Boardwalk", "group": 7, "price": 400, "rent": [50, 200, 600, 1400, 1700, 2000], "house": 200 }"#,
            "",
            1,
        );
        assert!(Board::parse(&short).is_err());
        // A colour that is not #rrggbb.
        assert!(Board::parse(&text.replace("#954936", "brown")).is_err());
    }

    #[test]
    fn only_a_classic_board_follows_the_language() {
        let classic = board::builtin("fr").unwrap();
        assert!(following_language(&classic).is_none(), "already the right language");
        let themed = board::builtin_by_id("saint-etienne").unwrap();
        assert!(following_language(&themed).is_none(), "a themed board keeps its names");
        assert!(!themed.classic);
        assert!(classic.classic);
    }

    #[test]
    fn a_shipped_board_follows_the_language() {
        // An imported board has no language: it never follows.
        let mut imported = board::builtin("en").unwrap();
        imported.language = None;
        assert!(following_language(&imported).is_none());
    }
}

