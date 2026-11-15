# Crazyopoly — instructions du projet

## Publication sur GitHub

- Dépôt de référence : <https://github.com/MonsieurM2/CrazyPoly> (`origin`, branche `main`).
- **Toute nouvelle version doit être commitée et poussée sur ce dépôt.**
- Avant de pousser : `cargo build --release`, puis
  `cp target/release/crazyopoly.exe dist/Crazyopoly.exe` pour garder l'exécutable
  de `dist/` à jour, et mettre à jour `README.md` si le comportement change.

## Captures du README

- Les images de `docs/` sont prises **en anglais** (le README est en anglais) et
  **recadrées sur la zone client** de la fenêtre, jamais sur le bureau Windows.
- On ne refait que les images dont le rendu a changé ; les autres sont conservées.

## Vérifications

- `cargo build --release` doit passer sans warning.
- Les dés sont simulés par `rapier3d` dans `src/dice_physics.rs` ; la valeur d'un
  lancer vient de la face du dessus de la pose finale, jamais d'un tirage direct.
