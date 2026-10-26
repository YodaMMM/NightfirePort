## pickup-medal-gate

- (16:15 UTC, 10-26) Branch created from origin/master (82362142), pushed. Task: port the
  kind-6 ('007' bonus) pickups' gold-medal gate — PlrStats_HasGoldMedal 0x199a40 [M] and the
  per-level medal record the save keeps (PlrStats) — so the pickups appear/take exactly when
  the game allows. Tests: no medal for the level -> the game's no-medal arm at create/take
  [M]; medal recorded -> the other arm [M]. Suite runner starts first.
