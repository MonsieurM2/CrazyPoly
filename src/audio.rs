//! Music and sound effects.
//!
//! The soundtrack and every effect are synthesised from scratch by
//! `make_audio.py` (sines, filtered noise, a small Schroeder reverb) and
//! embedded in the binary with `include_bytes!`, so the game stays one
//! self-contained executable with no external files and nothing to license.
//!
//! The handles live in a thread-local: macroquad's audio engine is global and
//! single-threaded, and loading is async, so `load()` is awaited once at start
//! up and everything afterwards is fire-and-forget.

use std::cell::RefCell;

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

const MUSIC: &[u8] = include_bytes!("../assets/audio/music.wav");
const CLICK: &[u8] = include_bytes!("../assets/audio/click.wav");
const DICE: &[u8] = include_bytes!("../assets/audio/dice.wav");
const STEP: &[u8] = include_bytes!("../assets/audio/step.wav");
const CASH: &[u8] = include_bytes!("../assets/audio/cash.wav");
const BUY: &[u8] = include_bytes!("../assets/audio/buy.wav");
const CARD: &[u8] = include_bytes!("../assets/audio/card.wav");
const JAIL: &[u8] = include_bytes!("../assets/audio/jail.wav");
const BUILD: &[u8] = include_bytes!("../assets/audio/build.wav");
const BAD: &[u8] = include_bytes!("../assets/audio/bad.wav");
const WIN: &[u8] = include_bytes!("../assets/audio/win.wav");

const MUSIC_VOLUME: f32 = 0.32;
const SFX_VOLUME: f32 = 0.75;

/// One sound of the set.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    /// A button was pressed.
    Click,
    /// The dice are thrown.
    Dice,
    /// A token walks one space.
    Step,
    /// Money changes hands.
    Cash,
    /// A lot is bought.
    Buy,
    /// A Chance or Community Chest card is drawn.
    Card,
    /// A token is sent to jail.
    Jail,
    /// A house or a hotel goes up.
    Build,
    /// Something is refused, or a debt opens.
    Bad,
    /// The game is won.
    Win,
}

struct Bank {
    music: Sound,
    click: Sound,
    dice: Sound,
    step: Sound,
    cash: Sound,
    buy: Sound,
    card: Sound,
    jail: Sound,
    build: Sound,
    bad: Sound,
    win: Sound,
    music_on: bool,
    sfx_on: bool,
}

thread_local! {
    static BANK: RefCell<Option<Bank>> = const { RefCell::new(None) };
}

/// Load every sound once. Silently gives up when the machine has no audio
/// device, so a silent computer can still play the game.
pub async fn load() {
    macro_rules! snd {
        ($bytes:expr) => {
            match load_sound_from_bytes($bytes).await {
                Ok(sound) => sound,
                Err(_) => return,
            }
        };
    }

    let mut bank = Bank {
        music: snd!(MUSIC),
        click: snd!(CLICK),
        dice: snd!(DICE),
        step: snd!(STEP),
        cash: snd!(CASH),
        buy: snd!(BUY),
        card: snd!(CARD),
        jail: snd!(JAIL),
        build: snd!(BUILD),
        bad: snd!(BAD),
        win: snd!(WIN),
        music_on: true,
        sfx_on: true,
    };
    start_music(&mut bank);
    BANK.with(|cell| *cell.borrow_mut() = Some(bank));
}

fn start_music(bank: &Bank) {
    play_sound(
        &bank.music,
        PlaySoundParams {
            looped: true,
            volume: MUSIC_VOLUME,
        },
    );
}

/// Play one effect, unless audio is unavailable or the effects are muted.
pub fn sfx(kind: Sfx) {
    BANK.with(|cell| {
        let cell = cell.borrow();
        let Some(bank) = cell.as_ref() else {
            return;
        };
        if !bank.sfx_on {
            return;
        }
        let sound = match kind {
            Sfx::Click => &bank.click,
            Sfx::Dice => &bank.dice,
            Sfx::Step => &bank.step,
            Sfx::Cash => &bank.cash,
            Sfx::Buy => &bank.buy,
            Sfx::Card => &bank.card,
            Sfx::Jail => &bank.jail,
            Sfx::Build => &bank.build,
            Sfx::Bad => &bank.bad,
            Sfx::Win => &bank.win,
        };
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: SFX_VOLUME,
            },
        );
    });
}

/// Flip the music and return its new state.
pub fn toggle_music() -> bool {
    BANK.with(|cell| {
        let mut cell = cell.borrow_mut();
        let Some(bank) = cell.as_mut() else {
            return false;
        };
        bank.music_on = !bank.music_on;
        if bank.music_on {
            start_music(bank);
        } else {
            stop_sound(&bank.music);
        }
        bank.music_on
    })
}

/// Flip the sound effects and return their new state.
pub fn toggle_sfx() -> bool {
    BANK.with(|cell| {
        let mut cell = cell.borrow_mut();
        let Some(bank) = cell.as_mut() else {
            return false;
        };
        bank.sfx_on = !bank.sfx_on;
        bank.sfx_on
    })
}

pub fn music_on() -> bool {
    BANK.with(|cell| cell.borrow().as_ref().map(|b| b.music_on).unwrap_or(false))
}

pub fn sfx_on() -> bool {
    BANK.with(|cell| cell.borrow().as_ref().map(|b| b.sfx_on).unwrap_or(false))
}
