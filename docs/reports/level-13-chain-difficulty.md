# level-13-chain-difficulty — the 12 → 13 chain walk plays one difficulty end to end

## What was wrong

`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out` loaded 07000012
with `load_with_carry_difficulty(..., 3)` (00 Agent — the route pins the Agent+ roster and the
keypad's no-create-unlock burn), but the two warm loads after it (0700004A, 07000013) went
through `load_with_carry`, which hard-coded difficulty 1. The chain therefore played 12 at 3
and 4A/13 at 1 — no player experience does that.

## The game's rule ([M])

The difficulty is the global `GameState+0x28` (0x2a3790 — the decomp's `GameState._40_4_`,
Ghidra DECIMAL 40 = hex 0x28). Whole-decompile grep over `work/decomp/ACTION.ELF`:

- writers, exactly five: `bootup` 0x1c7e78:25 (`GameState._40_4_ = 2`, the boot default),
  `C_SBNFDFCTY_Handler` 0x216ca8:15 (the difficulty page's pick), `P_MPCONFIRM_Handler`
  0x20ec00:150 (= 1), `Menu_RunMiniMission` 0x1ffb90:22 (= 1), `Boot_LoadPTPData` 0x1c8560:36
  (the driving-return PTP restore);
- every other referencing file reads it only (13 reader-only files: `Drone_Create` 0x136be0:49's
  min-difficulty tag, `NDrone2_DefaultInit` 0x14b300:505-526, `Lock_Create` 0x1ae1f8's
  difficulty-1 auto-unlock, `Player_RamLoad` 0x1957a8:23's boost pick, `Player_HandlePain`
  0x1902c8, `DroneWeap_DoBulletAccuracy` 0x179740, `Hint_Create` 0x1b3248,
  `SoundTrigger_Create` 0x1b2628, `PlrStat_GetScore` 0x1984b0, `Check_AutoAim` 0x1a73f8,
  `NDrone2_HitDamage` 0x145558, `Boot_GetPTPData` 0x1c8198).

None of the five writers runs in an on-foot level change, so the game carries one difficulty
through a mission by simply never touching the global. The port re-creates the Scene per load
and seeds the difficulty at build, so the value must ride the warm-carry vehicle — the RamSave
(the same treatment as the mission globals, see the `mission` field's doc).

## The port fix

- `RamSave.difficulty: u32` (nf-game/src/player.rs), 0 = not carried (the plain `capture` keeps
  every existing call site at its old difficulty-1 replay parity), plus the `with_difficulty`
  builder. The field's doc carries the writer/reader survey above.
- `load_with_carry` (nf-view/src/lib.rs) seeds the load's difficulty from the save when the save
  carries one (1..=4), else 1.
- `warm_exit_save` attaches the sim's mirror (`Drones::difficulty`) — the front-end wrappers take
  the pick as a parameter and ignore it, so the App is unchanged.
- The bugreport entry.sav bumps to v3: `encode_ram_save` appends the difficulty u32,
  `decode_ram_save` reads it for v3 and leaves 0 for v1/v2 (still decodes).

## The tests

- Both chain captures in `walkreal_07000012_live` and `walkreal_07000013_live` attach
  `.with_difficulty(sim.drones.difficulty)`; the 13 load pins
  `sim.drones.difficulty == 3` with the [M] citation.
- The 13 test's body is factored into `walk_the_warm_chain(iso, real_damage)`; two wrappers run
  it: the existing top-up test (unchanged legs, now one difficulty end to end) and a NEW
  real-damage variant (`walker::with_real_damage`) that also takes the BodyArmour #414 detour
  the route passes inside the security room ([D] the placement at (−27.66, −368.51, 25.57);
  [M] `Pickup_Handler` 0x18e650:220-225 writes the armour float to 50 on the walk-over).

## Walker fixes (test helpers only — no game code changed)

Four real-damage runs died in the 13 lobby; the logs drove four tactic fixes in
`common/damage_log.rs` / `common/walker_b2.rs`:

1. **`switch_to_armed` accepted a gadget as armed.** The Q-Decryptor's beam charge (clip 500)
   satisfied `clip + spare > 0`, so after the keypad burn every fight "fired" the gadget at the
   guard and Bond died trading (the first run: fight#3 on 12 stood at clip 500 while #3/#10
   dealt 18.5). The armed test is now return_fire's in-hand gate, `(2..=0x47).contains(&weapon)`
   — the gadget ids sit above the gun range.
2. **The shooters windows.** The vertical 6.0 excluded the tower lobby's mezzanine sniper (deck
   y −362.36 vs floor −368.43, dy 6.03 [D the drone dump]) — widened to 7.5. The horizontal 20
   excluded the same sniper working the north lane from d ≈ 43 — widened to 48 (the fight is a
   stand; `exposed_part`'s world trace still decides reachability).
3. **Dry fights.** The fight at the security-room door started on clip 2 spare 0 and the 24 hp
   sniper outlived it. `return_fire` now resupplies (the walker's existing `resupply` — the
   guards' floor drops) before giving up when unarmed, before a fight when the in-hand total is
   under 10, and after a fight that emptied the gun; a `resupplying` re-entrancy guard stops the
   resupply nav's own return-fire gate from recursing (the first attempt overflowed the stack).
4. **Target pick + spacing.** The fight target is the damage-weighted shooter over the same
   240-frame window (nearest first on ties) — nearest-first spent the crossing on 0-damage desk
   guards while the rifles chewed health — and the real-damage chain opts into the stand's
   `back_off = Some(3.0)` (under 3.0 u every drone round hits [M `DroneWeap_DoBulletAccuracy`
   0x179740:113-115]; the spawn-lobby fights ran at 0.9 u).

Effect: the 12 legs went from arriving at 13 with 18.5 health to 78.0, and the desk fights are
free; the crossing death moved from the first leg to the glass band.

## What is left

- **The real-damage variant is `#[ignore]`d** with the reason in its doc: the north-lane
  crossing at z ≈ 39 cannot be survived yet. Two rifle guards (0x5000006, weapon 0x18, 7.0 a
  hit) — the mezzanine sniper #4 at (15.4, −362.4, 42.3), d 38.6, and spawner #6 at
  (4.1, −368.4, 37.2), d 26.8 — hit Bond from cover the walker's `exposed_part` never exposes
  (their rounds reach him; the ray to their hit boxes does not), so no stand can answer them.
  Next steps: find the cover lane the game's route uses (the desk block at x −14.6..−12.9 /
  z 31.3..32.9 is the candidate), or decide the port's guard LOS is stricter than the game's —
  that would be a game-code divergence and needs the live check below first.
- The ignored variant still pins everything it passes: 12 completes on real damage at 78.0
  health with the chain's difficulty, the 13 desk guards die free, the armour take works.

## Live checks for the main agent

1. **Can the player see the mezzanine sniper from the lobby floor?** Fresh boot on 07000013,
   00 Agent (the difficulty wheel's third row), walk Bond to the north lane (x −12, y −368.4,
   z 39.5) facing east: the sniper stands on the mezzanine at about (15.5, −362.4, 41.6).
   Question: does the game's crosshair pick him (the aim dot turns hostile / the name shows)?
   If the game CAN target him from the lane, the port's `exposed_part` world trace is stricter
   than the game's pick — that is a port gap (cite `Collide_Pick`/`Collide_Jointy` 0x1ec180 vs
   the auto-aim `Check_AutoAim` 0x1a73f8). If he is NOT targetable, the lane is wrong and the
   route needs the cover lane.
2. **The spawner rifle guard #6 (4.1, −368.4, 37.2):** same question from the same lane, and
   which channel spawns him (his template's start channel — the dump showed present=false at
   load; the disc census says spawner start channels 0x29/0x2a have no writer, so he spawns as
   placed — confirm he is visible/targetable when he opens fire).

## Verification (what was run)

- `cargo test --offline --release -j4 -p nf-view --test walkreal_07000012_live -- --test-threads=1`
  → `test result: ok. 1 passed; 0 failed` (7.64 s).
- `... --test walkreal_07000013_live -- --test-threads=1` → `test result: ok. 1 passed; 0
  failed; 1 ignored` (10.31 s).
- `cargo test --offline --release -j4 -p nf-game` → 100/100 test binaries `test result: ok`.
- The full suite (`tools/test/suite.py`, NF_ISO set) was started on the final tree; see the
  total line the driver quoted from `suite.out` — the sandbox's 2-hour window ended before the
  full run printed its total.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
