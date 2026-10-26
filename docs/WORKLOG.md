## pickup-medal-gate

- (16:15 UTC, 10-26) Branch created from origin/master (82362142), pushed. Task: port the
  kind-6 ('007' bonus) pickups' gold-medal gate — PlrStats_HasGoldMedal 0x199a40 [M] and the
  per-level medal record the save keeps (PlrStats) — so the pickups appear/take exactly when
  the game allows. Tests: no medal for the level -> the game's no-medal arm at create/take
  [M]; medal recorded -> the other arm [M]. Suite runner starts first.

## pickup-medal-gate (2026-10-26, this window)

Done:
- the gate: `Pickup::create_placed` refuses kind 6 with no gold medal, in the game's order
  (after the id −1 check, before the 7..=11 set intercept) [M Pickup_Create 0x18dda0:93-95].
- the medal state: `progress::has_gold_medal(bits, hash)` / `medal_set` — the
  `PlrStats_HasGoldMedal` 0x199a40 → `Menu_HasMedal` 0x202580 → `PlrStarts_ProcessRewardCounter`
  0x199210 chain over the explicit `bonus[0]` word; `Progress::has_medal` delegates to it. The
  counter guard fixed to the game's `param_3 < 5` [M 0x199210:24] (the old `> 5` let counter 5
  index desc[5] out of bounds).
- the BaseMap: `Mission_BaseMapHCode` 0x1830a0 returns the MISSION table's map, not the archive
  id — the decompiler renders Mission_Init 0x182270's table walk with an off-by-one; the DISASM
  pairs the mission id at 0x2a4350+k·0x28 with the map at +4 [L ee_start_05 0x2a4350]:
  0700000a plays under 0x07000009, 07000011 under 0x07000011. The port reads
  `crate::mission::LEVELS` (same rows) via `DroneContext::has_gold_medal(level_id)`.
- the carrier: `DroneContext.bonus` (the game's `bonus[0]` 0x2e1b88) rides the load; the front
  end passes the active profile's word on every launch (`load_with_carry_medals`, `FrontEnd::
  bonus_bits`); the plain `load*` wrappers keep 0 (the fresh-boot BSS state [L ee_start_05]).
- tests: `gold_medal_gate.rs` (the medal math headless: counter-3 bit, the 0x0700005/9/11
  kind-4 gold group's per-level bits, the mission-table BaseMaps, the no-row/kind-5 arms; the
  create gate over the disc's weapon_data: no-medal → None, medal → Some+spin, kind 0/0xFFFF
  unaffected). Eight kind-6 live tests re-pinned to medaled loads, each with a no-medal sister
  (the old plain loads pinned the port's ungated create — the decompile contradicts them; fixed
  with the address, per the rules).
- ran: `cargo test --offline --release -j2 -p nf-game --test gold_medal_gate` blocked on the
  suite's build lock; the full suite (`tools/test/suite.py`) re-started on the final tree at
  ~17:38 UTC — see the report for its state.
