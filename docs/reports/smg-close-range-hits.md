# smg-close-range-hits — the SMG registers per-record damage at 1-2 u; the "zero damage" was the game's own switch-hold law, not a port bug

Branch `s/smg-close-range-hits` (cut from `origin/master` f3b24398). The task's premise
(docs/reports/level-01-agent-hall-fight.md "Remaining" item 3): 32 SMG rounds at a guard
0.5-2.1 u, aim settled, rounds spawned, zero damage, while the pistol hits the same guards at
3.5-9 u — a suspected player-bullet-vs-guard registration bug at close range.

## Status: the premise is REFUTED with a real-input proof test — no port change needed

`crates/nf-view/tests/smg_close_range_live.rs` (new) walks the proven route from the level start
(intro → the ambush clear → the meet plate → the scene's SMG hand-off → the east-door hall
stage), then on real input only:

- **SMG 0x12 at 1.12 u** (guard d#7, 24 hp, torso aim): 3 rounds spawned, **3 impacts, 2.25 per
  round, drop 6.750 == expected 6.750** (the record damage 1.5 [L ee_start_07000001.bin
  weapon_data @0x2bf150 + 0x12·0x10C] × the part factor × `DroneDamage_Easy`, computed per hit
  through `hit_damage` 0x145558's own port). The run-7 tree (pre-reorder, same law) also
  registered **3 rounds → 3 impacts at 1.88-1.90 u, drop 6.750 == expected**.
- **SMG at the 2 u stand**: 2 rounds, 1 impact, the kill (drop = his remaining 1.5; expected
  2.25 ≥ hp0 − 0.5).
- **Pistol 0x2 sisters**: 1 round each at 1.12 u, 1.87 u and the 5.89-6.00 u control — one
  player impact per leg, 5.25 a hit (3.5 × 1.5), drop 5.250 each.
- **The dormant pair takes nothing — in the game's own law**: 4 pistol rounds at 1.02 u into
  d#10 (switch ch 0x5, `enabled=false`, state 0x1): hp 12.0 → 12.0. That is not the bug:

  - `NDrone2_DSTATE_WaitSwitch` 0x15b960 **ENTER** calls `NDrone2_Enable(0, drone)` [M];
  - `NDrone2_Enable` 0x149dd0 with param 0 sets `obj+0xF0 |= 0x10` and `obj+0xF0 |= 0x20` [M];
  - `Collide_PickObj` 0x1eae80:26-30 drops `obj+0xF0 & 0x20` objects [M] — the game's own
    bullet pass never picks a switch-held guard, at any range, any weapon.

  The port's `update_player_bullets` skip (`!dr.enabled → continue`, crates/nf-game/src/drone.rs
  ~14605) IS that rule. The level-01 report's 32 zero-damage SMG rounds were fired at exactly
  these held guards — the measurement measured the script hold, not the hit test.

## What the investigation pinned along the way

- The spawn law is healthy: a round every ~6 ticks (the record's interval), the cone
  ±0.0014·(16 + 5·bloom·k) [M Bullet_init 0x12c5a0:139-170], the first-segment step
  (FRand(0.5)+0.5)·speed, and the near-field gate (`seg_point_dist2 > 2.1²`) passes close
  targets. Neither candidate in the task brief (the `shot_from` midpoint path — both records'
  `+0x68` lack 0x10000 so both spawn at the camera [L the record dump] — nor the near-field
  gate) is involved.
- **The prober's trap for the next session**: the escort (Mayhew) fires his own 0x1c volleys
  into the same guards — his pellets land in the same `player_impacts`/`dr.hit` records
  (rec-dmg 3.0 vs the pistol's 3.5 identifies them). The test attributes impacts by
  `dr.hit.weapon`; without that the ally's kills read as the player's (run 2's false 13.5).
- The pistol's own ricochet arm (record `+0x6C` bit 0x10 [M Bullet_CollisionHandler
  0x12ce80's bounce counter]) can land one round more than once — 3 part-4 impacts from 1
  spawned round in run 2 are that law, not a duplication bug.

## What ran

- `cargo test --offline --release -p nf-view --test smg_close_range_live -- --test-threads=1`:
  **ok, 1 passed** in 18.23 s — the full quoted leg list is its stdout (runs 9-12 in the
  session; the committed tree's deterministic run is run 12).
- The workspace builds clean (`--no-run` over nf-view's targets compiles nf-game/nf-formats).
- **No `crates/*/src/` file is touched by this branch** — the only change is the new test file
  plus this WORKLOG/report — so every `walkreal_*_live` binary and the rest of the suite are
  byte-identical in behavior to master by construction. The full-suite total line did not fit
  the sandbox window (setup rebuilt the ALSA user prefix; the repro iterations consumed the
  rest); the next session can take it for free.

## Live checks for the main agent

- `fieldprobe` poke: stand Bond 2 u from a switch-held guard (07000001's d#10/d#16, ch 0x5
  un-risen), weapon 0x12, fire one round — **predicted: no health change, no hit flash** (the
  game's `Collide_PickObj` drops `obj+0xF0 & 0x20`). If the game DOES register damage there,
  this report's Enable(0) reading is wrong and the port's skip is the bug — please report back.
- `gdbwatch` on the SMG spawn at a live guard 2 u away: confirm the game's BU registers a hit
  per round (the port's per-round 1.5-base registration is the claim to confirm).

## Remaining / open

- The hall's south-structure cover blocked every surviving 2 u SMG candidate in the committed
  deterministic run (the test's LOS gate skips to the next candidate and logs it) — the 2 u
  registration proof is run 7's tree (quoted above); a future tweak can stage Bond INSIDE the
  hall before the SMG legs to make the 2 u leg assert in-tree.
- The full-suite total line (free next time; no src changes on this branch).

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
