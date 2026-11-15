# Crazyopoly

A pixel-faithful, top-down recreation of the classic Monopoly board game in Rust,
built with [`macroquad`](https://github.com/not-fl3/macroquad). The board and the
pieces are drawn with primitives at runtime, so the whole game is a handful of
`src` files; the only assets are two bundled fonts, [Lato](https://fonts.google.com/specimen/Lato)
for body text and [Playfair Display](https://fonts.google.com/specimen/Playfair+Display)
for the titles (both SIL Open Font License), plus the soundtrack and the sound
effects, all synthesised from scratch for this game (see **Sound** below).

Base repository: <https://github.com/MonsieurM2/CrazyPoly>

A property is offered: its title deed floats over the board, the decision sits in
the right-hand column, and the log follows every move in the players' colours:

![Crazyopoly in play](docs/buy.png)

Start it, press **NEW GAME**, pick your token, and the table is yours - or let
every seat be a bot and watch it play itself:

![The main menu](docs/menu.png)

## Features

- **1 to 8 players** on one shared screen (hot-seat), with an **AI bot** toggle for
  every seat: solo against bots, a mixed table, or a game with **no human at all**
  (set every seat to BOT and watch it play itself).
- The complete **40-space official board**: all streets, colour groups, prices,
  rents, houses/hotels, four railroads and two utilities, Income Tax, Luxury Tax,
  Jail / Just Visiting, Free Parking, Go To Jail, Chance and Community Chest,
  plus the dice at the centre of the board.
- An **animated main menu**: gold dust drifts up over the felt, the rings behind
  the card breathe, the title carries a pulsing halo, the eight tokens float in a
  row and a sheen runs across **NEW GAME**. It offers **NEW GAME**, **CONTINUE**
  (the saved game) and the rules of the game; the language and sound switches sit
  in the corner. In game, the **MENU** button at the top of the log asks once and
  takes you back to it.
- **A real rulebook**: **RULES OF THE GAME / RÈGLES DU JEU** - on the menu, and in
  the pause menu during play - opens a four-chapter booklet on cream paper, set
  in two balanced columns: the table, the board, building, and the cards and the
  end of the game. Each rule is laid out whole, so no section is ever cut by a
  column break or a page turn; the pages turn with the arrows, the keyboard or
  the dots, and the table stays frozen until the book is closed.
- Full rules engine:
  - dice rolling, doubles (and the third-doubles jail rule), GO salary
  - property purchase, rent (including the monopoly double-rent rule),
    railroad rent by count, utilities at 4x / 10x the dice
  - **houses and hotels** with even-build/even-sell rules - and **no limit**:
    the bank always has another house, another hotel and more money
  - **mortgaging** and lifting mortgages (with 10% interest)
  - **jail**: roll for doubles, pay the $50 fine, use a card, or serve three turns
  - **Chance and Community Chest**: all 16 classic cards in each deck
    (advance cards, nearest utility/railroad with the proper rent modifier,
    repairs, payments to/from every player, Get Out of Jail Free)
  - **auctions** when a player declines to buy
  - **trading**: properties plus cash, offered to a human or a bot, which
    evaluates the deal; bots open trades **between themselves** too
  - **debt and bankruptcy**: sell buildings and mortgage lots to cover a debt,
    or declare bankruptcy; assets pass to the creditor (or the bank) and the
    last solvent player wins
  - **ending a game early**: anyone can call the vote from **END THE GAME** in the
    pause menu, at any moment, even in the middle of a bot's turn. Every human has to agree (bots
    always do); the ranking is by **net worth** -
    cash plus the price of every deed plus its buildings - so the player who
    would have won the long game wins here too
- **French and English** interface, switchable at any time with the **FR / EN**
  button under the player list and on the setup screen. French is the default; street
  names stay in English to match the classic board.
- Bots buy, bid, build, raise cash and answer trade offers on their own. They also
  **start trades**: when a bot holds two lots of a colour group it offers cash for
  the third. A human on that seat gets the usual trade window; another bot prices
  the deal, and when the two agree the lots and the cash change hands on the spot,
  with the exchange printed in the log.
- **Save and load** a game in progress (one slot, plain JSON) plus a prebuilt
  Windows executable in `dist/`.
- **Physical dice**: every roll is a real rigid-body simulation run with the
  generic [`rapier3d`](https://rapier.rs) engine - a floor, four walls (the tray)
  and two cubes with mass, friction, restitution and a random shove and spin.
  The cubes tumble, collide, bounce and finally come to rest somewhere in the
  tray; the recorded motion is then replayed on screen. Because the board is seen
  from straight above through an orthographic camera, a die at rest shows exactly
  one face - the upper one - and that face, read back from the simulation, *is*
  the result of the throw. Nothing is drawn to force a value. Once both dice have
  stopped, they hop back to their home spots on either side of the panel centre,
  keeping the face they landed on, ready for the next roll.
- **Eight token designs** (car, top hat, dog, boot, ship, cat, plane, diamond)
  drawn with primitives - pick one for each seat on the setup screen.
- **Music and sound effects**, all original and royalty-free: a laid-back
  salon loop (electric piano, bass, brushed ride, small room reverb) under the
  whole game, plus dice tumbling, a step per space, coins, a purchase chord, a
  card swoosh, the jail clank, a hammer for houses, a low buzz for debts and a
  short fanfare when someone wins. Two icon buttons - an equaliser for the
  effects and a note for the music - turn either channel off at any time, on the
  setup screen and in the left column of the board.
- **Every space shows its own deed**: hover any tile and its title deed opens
  under the cursor - the rent table for a street, the four rates for a railroad,
  the 4x / 10x dice multipliers for a utility. The owner - a colour dot and a
  name, or *Unowned* in grey - sits in a cream plate at the foot of the card,
  drawn in the same cream-and-gold style as the rest of it.
- **Your deeds on demand**: in **MES BIENS / ASSETS**, click a lot to read its
  card without leaving the window, even mid-turn.
- **Fast forward**: the `>>>` button at the bottom of the left column resolves
  the dice, the walk and every bot decision instantly, so a table of bots plays
  itself in seconds instead of minutes. Press it again to get the animations back.
- **Renamable seats**: click a name on the setup screen and type - every player,
  human or bot, can be called whatever you like.
- Animated token movement, hover tooltips on every tile, live game log.
- A **pause menu** on **Escape**: the whole table freezes - dice, animations,
  bots and all - and offers **RESUME**, **SAVE**, the rulebook and **END THE GAME**.
  It can be opened at any moment, including while a bot is playing, and the same
  command sits as a button in the bottom-left bar.
- A **three-column layout** that fills the window: the square board in the middle,
  the players on the left, the current instruction, the action buttons and the
  game log on the right. Buying is decided right there - the
  prompt reads *"Vermont Avenue is free"*, the buttons offer **BUY** or **PASS**
  (which opens the auction), and the property's **title deed** floats over the
  board with its full rent table, so the price is visible before you commit.

## Screenshots

Every capture below is taken in English, cropped to the window itself.

The setup screen - 2 to 8 seats, each one HUMAN or BOT, a token design per seat
and the FR / EN switch:

![Setup screen](docs/setup.png)

A throw in progress: the cubes tumble, bounce and land, and the upper face of
each one is the result. Once they have stopped they hop back to their home spots,
still showing the face they landed on:

![The dice tumbling](docs/dice.gif)

The rulebook - four chapters, two columns, every rule laid out whole - one click
from the menu or from the pause menu:

![The rules](docs/rules.png)

Everything the board cannot show has its own window: the pause menu, buildings,
selling and mortgaging, trading properties for cash, auctions, debts, the
end-of-game vote, the final ranking and the token picker.

| The pause menu (Escape) | Build, sell, mortgage |
| --- | --- |
| ![The pause menu](docs/pause.png) | ![The assets window](docs/assets.png) |

| Read a deed you own | Trade lots for cash |
| --- | --- |
| ![A deed opened from the assets window](docs/card.png) | ![The trade window](docs/trade.png) |

| Auctions | Debt and bankruptcy |
| --- | --- |
| ![An auction](docs/auction.png) | ![A debt to settle](docs/debt.png) |

| Ending a game by agreement | The final ranking |
| --- | --- |
| ![The vote to end the game](docs/vote.png) | ![Game over, ranked by net worth](docs/over.png) |

| Hover any tile for its deed | Eight token designs |
| --- | --- |
| ![A title deed on hover](docs/hover.png) | ![Choosing a token](docs/tokens.png) |

## Sound

The music and the eleven effects are **original**: `make_audio.py` renders them
from sines, filtered noise and a small Schroeder reverb, so nothing here is
borrowed and there is no licence to check. The WAVs are compiled into the
executable with `include_bytes!`, so the game still ships as a single file.

- The loop starts on the setup screen and runs through the whole game.
- The note button mutes the music, the equaliser button mutes the effects; both
  icons are struck through when their channel is off.
- No audio device (or a failed load) simply means a silent game - nothing else
  depends on it.
- To tweak the sound, edit `make_audio.py` and run `python make_audio.py`, then
  rebuild.

## Requirements

- Rust 1.75 or newer (`rustup` recommended): <https://rustup.rs>
- A desktop with OpenGL/Metal/Direct3D support. Linux additionally needs the
  usual X11/Wayland development packages that macroquad/miniquad requires
  (`libx11-dev libxi-dev libgl1-mesa-dev` on Debian/Ubuntu).
- No extra system library for the dice: the physics engine (`rapier3d`) is pure
  Rust and is compiled in with the rest of the project.

## Run

```bash
cargo run --release
```

The first build downloads `macroquad` and compiles it, which takes a minute or two.
Later launches are instant. A 1440x920 window opens; it is resizable.

To build only:

```bash
cargo build --release
```

The binary is at `target/release/crazyopoly` (`crazyopoly.exe` on Windows).

## Prebuilt executable

`dist/Crazyopoly.exe` is a ready-to-run Windows build: double-click it, no Rust
toolchain needed. Rebuild and refresh it with:

```bash
cargo build --release
cp target/release/crazyopoly.exe dist/Crazyopoly.exe
```

## Saving and loading

- **SAVE / SAUVEGARDER**, in the pause menu, writes the current game at any
  point where nothing is moving (it is disabled while dice are tumbling, a card
  is being resolved, or an auction/debt dialog is open).
- On the setup screen, **CONTINUE GAME** resumes the saved game. It is disabled
  when no save exists.
- The slot lives at `%APPDATA%\Crazyopoly\savegame.json` on Windows and
  `~/.local/share/Crazyopoly/savegame.json` elsewhere. Set the `CRAZYOPOLY_SAVE`
  environment variable to any file path to use a different location.
- The save holds players, cash, deeds, buildings, mortgages, the bank supply,
  turn order, dice and both card decks in their current order.

## How to play

1. Press **NEW GAME** on the main menu (or **CONTINUE** to resume a save), then on
   the setup screen choose **PLAYERS** (2-8 seats). Each seat can be
   **HUMAN** or **BOT**; click a name to type a new one, and click the token on
   the left to pick another design. With one human and one bot you have a solo
   game - bots fill any seat you leave to them. Set **every** seat to BOT and the
   game plays itself, which is handy to watch the AI (and the dice) at work.
2. Press **START GAME**.
3. On your turn press **ROLL THE DICE**. Your token walks around the board; the
   `>>>` button below the player list fast-forwards the animations (and the
   bots) whenever you want to speed things up.
4. When you land on an unowned property, choose **BUY** or **PASS** (which puts
   the lot up for auction); the deed floating over the board shows every rent.
5. Use **ASSETS** to add houses and hotels, or to raise cash; **TRADE** opens the
   trade window; **END TURN** passes play on. **Escape** opens the pause menu.
6. If you owe more cash than you hold, the debt dialog appears: sell buildings,
   mortgage lots, then pay - or declare bankruptcy.
7. The last solvent player wins.

Hover over any tile for its price, rent and owner. The log in the right-hand
column records everything that happens, and **END THE GAME** in the pause menu
starts the vote to stop early: the richest player - cash, deeds and buildings -
takes the table.

## Language

French is the default. Press the **FR / EN** button under the player list, or the
one on the setup screen, to switch instantly - labels, tooltips, cards,
buttons and every log message follow the active language. All strings live in
two places: `src/i18n.rs` (`t("English", "Français")` and the `tr!` macro for
formatted messages) and the bilingual card texts in `src/cards.rs`, so adding a
language only touches those two modules.

## Project layout

```
Cargo.toml
assets/
  Lato-Regular.ttf, Lato-Bold.ttf   bundled UI fonts (SIL OFL, see OFL.txt)
  PlayfairDisplay.ttf               titles (SIL OFL, see OFL-Playfair.txt)
  audio/                            music + effects (WAV, generated, see Sound)
src/
  audio.rs   music and effects: embedded WAVs, one-shot playback, the two toggles
  main.rs    window setup, app state, frame loop
  board.rs   static board data (names, groups, prices, rent tables)
  cards.rs   bilingual Chance / Community Chest decks
  game.rs    rules engine and phase state machine
  i18n.rs    active language, `t()` and the `tr!` macro
  ai.rs      computer players, including the trade offers they make
  dice.rs           replaying the recorded throw under a vertical camera
  dice_physics.rs   the throw itself: rapier3d world, recorded poses, face read-out
  tokens.rs  the eight player token designs, drawn with primitives
  persist.rs save/load (JSON, one slot)
  ui.rs      green table look: felt, wood-framed board, gold accents, three columns
```

`make_audio.py` at the top of the repository regenerates every file in
`assets/audio/` (it needs nothing but plain Python 3).

## Rule notes

- Free Parking collects nothing (as in the official rules; no house-rule pot).
- Auction bids are limited to the bidder's cash on hand.
- When a player goes bankrupt to the bank, their lots return to the bank rather
  than being re-auctioned.
- A hotel counts as five buildings for repair cards, and sells back for 2.5x the
  house price (half of the total invested).
