# level-11-start-fight-real-health-3 — report (2026-11-03)

**DONE.** `walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage` is
**un-ignored and green on real input** (23.8 s): the whole walked chain — 07000011 spawn → the
stairwell descent → the Load Point exit → the real 07000012 load (the difficulty-3 front-end
path) → tower2's route → the shaft exit → the real 0700004A load — completes on the port's own
damage with no health/armour/ammo writes and no teleports.

Branch `s/level-11-start-fight-real-health-3` (from origin/master 01917076, merged
origin/s/level-11-start-fight-real-health-2; origin/s/player-upward-shot is already fully in
master — `git log origin/master..origin/s/player-upward-shot` is empty).

## The task's questions, answered

1. **"First check whether the stands fire from the hip."** YES — every return-fire stand sends
   `Input { aiming: self.dmg.aimed_fights }` (default false) and taps at the plain 0.03 rad yaw /
   0.04 pitch gate [M damage_log.rs stand_and_fire].
2. **"Play the fights aimed."** Two measured halves:
   - **The tap gate, tightened (shipped).** New opt-in `DamageLog::precise_fights`: the tap fires
     only with the crosshair inside 0.01 rad of the exposed part and the line clear. The loose
     gate spent rounds the budget did not have — the 2026-11-03 repro (repro10.out) fired all 7
     by tick ~1020 (two of them from stands whose aim error printed −2.3/−2.5 rad at t0) and went
     DRY at the keypad with 24.5 health still to lose to #5 and 54.5 to #10. With the precise
     gate every fight dropped its target (repro11d: the stairwell fights all killed, 100.0 →
     56.2 through the descent).
   - **The aim button, deliberately excluded.** With `Input.aiming = true` the port's free-aim
     path reinterprets the fight's `turn`/`look` as CURSOR moves (scene.rs free-aim:
     `want = cursor − turn/0.76966`, clamped ±0.296) and the fired round's direction is the view
     PLUS the cursor offset — the 07000011 top-floor fight tapped 7 rounds at err 0.0000 into a
     ~4 u stationary guard for ZERO boxes (repro11c: clip 7→0, target hp 24.0 unchanged, then
     every stairwell fight dry). The fight machinery drives the VIEW; it cannot drive the cursor
     window. The 0c snipers keep the button (`aimed_fights`) — their zero spread needs the bit
     [M Bullet_init 0x12c5a0:139-170]; the mission-11 pistol has no 0x400000 record flag [L
     ee_start_05.bin 0x2bf150 + 0x10·0x10C + 0x68 = 0x1000], so the bit buys it nothing anyway.
3. **The kill order / the ammo math (why the head-hunt).** The fresh equip for BOTH legs is
   weapon 0x10 with SEVEN rounds [M Player_InitWeapon 0x1a7ef8:275-281 — case 0x7000011 equips
   0x10/7; case 0x7000012/13/4a RamLoad and, BL+0x27a clear, re-runs the same equip]. The
   pistol's vs-drone damage is **10.0 a torso round** [L ee_start_05.bin 0x2bf150 +
   0x10·0x10C + 0x0c = 10.0; the port's player-bullet damage reads record +0x0c [M
   fire_player → PlayerBullet.damage]] and one head hit is 20.0 (the observed −20.0 hits). So
   torso fights need 2-3 rounds a guard: the precise-gate run spent 6 of the 7 on the lobby
   (fight#9's hp-30 guard ate 2 torso rounds for 30→10, survived, and MELEED 14 later) and Bond
   crossed the south corridor with 1 round — not enough for the shotgun #10 (hp 12, needs 2
   torso rounds, 1 head hit). The fights now run the **07000012 walk's own head-hunt +
   unseen-fights law** (`with_real_damage_head_hunting` + `with_real_damage_unseen_fights`, the
   wrappers walkreal_07000012_live's real-damage sister runs on the same route, whose own doc
   cites [M NDrone2_HitDamage 0x145558 × DroneDamage_Head]): one head hit drops a 12-health
   guard.

## The result (repro11f.out, the passing run)

- 07000011: the fights killed #4, #0, #8, #9, #10, #12, #17, #14, #13 down the stairwell
  (every one "killed true"); Bond reached the Load Point and the exit fired.
- 07000012 (the d3 front-end load, 7 rounds again): the lobby fights killed #9, #8, #5, #4
  (including the previously unreachable east volleyer #5), resupply passes fetched the guard
  drops (clip 14 spare 0 at the corridor), and **#10 was killed at the south-east bend**
  (fight#10 t0: err 0.167, aim Some, blocked false — the exact spot the blocked reports named;
  killed true at tick 8894 at 79.0 health). The corridor crossing walked clean; the console,
  the lift, the shaft entrance and the gated exit fired; 0700004A's spawn drop completed
  0x04000038. No death anywhere on the chain.

## What shipped

- `crates/nf-view/tests/common/damage_log.rs`: new opt-in field `precise_fights` (the 0.01 rad
  gate WITHOUT the aim button; read in exactly one place — the gate line; default off, every
  other walk bit-identical) + the NF_RF_TAPS diagnostic trace (env-gated, per-tap aim state).
- `crates/nf-view/tests/walkreal_07000011_live.rs`: `precise_fights` set on both legs' walkers
  under `if !w.tops_up()` (inert under the top-up), the real-damage test wrapped in
  `with_real_damage_head_hunting` + `with_real_damage_unseen_fights` + `with_real_damage`, the
  `#[ignore]` REMOVED.

## Port divergences found (not fixed here — they are not this walk's blockers)

1. **The free-aim cursor vs the fight machinery** (above): a walker-fights-vs-aim-mode
   disagreement, the same class as the 0c report's "the predictor uses the flat trace, the flight
   the cel-scoped walk". A human player moves the stick and the reticle together; the port's
   fight loop only drives the view. No game address contradicts the port's own aim mode — the
   divergence is between the port's fight harness and the port's aim mode, so the walk routes
   around it (gate only). If a later task wants the fights to hold the aim button, the fight
   must drive `cursor` (the free-aim window) instead of `turn`/`look`.
2. **The resupply stall**: with the unseen-fights fallback the top floor ran a body-block →
   resupply loop whose nav attempt hit the 6000-tick cap (repro11f, "resupply 0 nav 3: gave up
   at the 6000-tick cap"); the walk still finished (23.8 s wall, ~100k sim ticks). The shared
   caps bounded it; no test failure. [S] the loop could budget tighter.

## What was run (the sandbox disk cannot hold the full 4-crate test build)

- `walkreal_07000011_live`: **2 passed / 0 failed** (34.9 s — the topped-up sister + the
  real-damage walk).
- `start_corridor_guard_07000011_live`: **8 passed / 0 failed** (13.7 s).
- nf-game + nf-formats + nf-audio (`--test-threads=1`): **136 passed, 3 failed** — exactly the 3
  allowed slim-ISO `disc_golden` (bigf header/tables/directory).
- nf-view lib tests: **100 passed / 0 failed**.
- **EVERY `walkreal_*_live` binary** (rule 7): the 19 per-level `walkreal_0700*_live` binaries
  (01, 02, 03, 04, 05, 06, 07, 08, 09, 0a, 0b, 0c, 0d, 11, 12, 13, 14, 15, 16, 1b, 41, 4a —
  14/15 passed this time; the prior sessions' SIGKILLs did not recur) plus barrelrow, eastroute,
  11_route, 4a_satchel, celprobe, dump — **25 binaries, 0 failures**.
- NOT run: the nf-view `fix_*`/`probe_*`/`gate_*` test binaries (~800 of them) and
  `tools/test/suite.py`'s all-at-once build — the sandbox disk (9.9 GB total, ~5 GB free with
  the ISO + work/ in place) cannot hold their ~12 GB of artifacts; the build OOMed the disk
  three times this session. The suite's required chunks that DO fit all ran green above. The
  helper change is opt-in and read in one place, and the whole walkreal sweep re-ran green.

## Live checks for the main agent

1. The 0x10 vs-drone damage: fresh boot, 07000011, shoot a stairwell guard in the torso with the
   starting pistol — expect 10.0 per torso round and 20.0 per head round [L ee_start_05.bin
   0x2bf150 + 0x10·0x10C + 0x0c = 10.0; M NDrone2_HitDamage 0x145558 × DroneDamage_Head].
2. The fresh-equip ammo: `goto_level 07000012` from the front end (no save) — expect exactly 7
   rounds of weapon 0x10 [M Player_InitWeapon 0x1a7ef8:275-288].
3. The aim-mode round direction: hold the aim button and fire with the reticle parked off-centre
   — the game's round should follow the reticle window (the port's free-aim), and gdbwatch on
   Bullet_init 0x12c5a0's param direction vs the view would settle whether the game's aim-mode
   bullet direction includes the cursor offset the way the port's does (the port: view + cursor
   offset — scene.rs free-aim branch). This is the divergence the walk routes around.

## What's left

Nothing for this task. The walk is green on real input and the sweep is green. The two port
notes above are candidate follow-up tasks, not blockers.

## Usage

not visible to me (no token/credit/context figures are exposed to this session; nothing in the
sandbox reports them).
