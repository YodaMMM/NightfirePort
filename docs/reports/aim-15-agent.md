# aim-15-agent — the 15 real-damage Agent walk goes green: the opt-in pre-throw kill stand (the 0c `sniper_stands` pattern), the drawn 0x1a, and the great-hall throw race won

Branch `s/aim-15-agent`, cut from `origin/s/aim-on-master-2` (baseline 4433 passed / 1 failed /
354 ignored; the one red `walkreal_07000015_live walk_spawn_to_the_omega_sector_exit_real_damage_agent`
— the d2 Agent walk that died at tick 9422 taking exactly 100.0, 13 hits, 18 return-fire fights
[L the aim-last-seven §3 ledger]).

## 1. What was built (test-side only; no `crates/*/src` file changed)

**`pre_throw_stands`** — a new opt-in flag on the shared `DamageLog`
(`crates/nf-view/tests/common/damage_log.rs`), set by the 15 d2 Agent run only:

- **The release detector** (`alive_seen`): the first walker poll that sees a drone
  live-hostile (present && enabled && !corpse && hp > 0 && team 1) IS its release, within the
  10-tick poll cadence. This catches BOTH release kinds: the spawner templates (the spawner
  arm writes `present` and the game's own `Drone::spawn_frame` stamp [M nf-game drone.rs]) and
  the **channel-held placements** (enabled flips at the channel, no spawn stamp — the
  spawn-stamp-only scan of the first run never saw the west-room pair). A guard's first-seen
  tick inside `PRE_THROW_WINDOW_TICKS` (1200) pre-empts the plain return-fire pick — nearest
  first, VISIBLE only, within the plain shooters' 20 u reach — and the fight runs
  `stand_and_fire` with the 0c pattern minus the aim bit: the settle (a tap into a live
  switch/fire anim is a wasted round [M 0x1a24c8:113-152's pressed-edge law]) and the tap
  gated on the exposed part at the 0.01-rad gate (the P2K/0x1a are not 0x400000 records, and
  the aim-bit hold would hand the drive to the cursor law [M scene.rs free-aim — the
  `precise_fights` measurement]).
- **`switch_dry` rides with it** (the existing 02/03 00agent opt-in): the run's fights ran
  dry mid-fight (the master give-up left #10 at full hp).
- **The fights' gun**: the fence dance holsters the fights' weapon for the pinned P2K graze
  geometry and nothing switches back — the walk drew the dropped 0x1a back out at the rim
  (the walk-over take armed it at the crossing; the take equips what Bond LACKS and takes
  rounds otherwise [M Pickup_Handler 0x18e650; Player_IsBetterWeapon 0x1a6eb0]). The 0x1a
  round is also the cheap kill: a direct hit detonates its blast AT the guard (80 raw, the
  falloff at d 0) — one round drops any hp-20 grenadier where the P2K's 3.5 needs six [L the
  run-3 fight taps; M the 0x1a record damage 3.5]. The draw needed pressed-edge retries
  (`draw_persist`): the request follows `shoot_mine`'s live fire script and a single press is
  dropped [M 0x1a24c8:113-152].
- **The great-hall throw race** (the walk body, d2-gated): the door[50] settle 40 → 12 ticks
  (the leaf clears the shot line in ~25 ticks of its swing [M door.rs update's open arm]) and
  the placement-733 pre-shoot runs `stand_and_fire` (no flanks — the flank machinery's
  sidesteps walk Bond off the threshold the race is run from). The run-6 ledger lost this
  race by ~20-30 ticks (#5's grenade was airborne at the kill, −27.7, the killing hit); the
  12-tick settle + the stand won it — #5 dies pre-throw.
- **The #447 fence-shot assert** now reads the muzzle-aware cause table (the range assert
  `d > 8.0` and the no-body-contact assert unchanged): the P2K round takes the #447 beam
  first (the neighbours go on its case-0x4B blast [M 0x17c518]); the fights' 0x1a GRENADE
  round's direct mirrored contact took the #448 diagonal first [L the run-7 shot: #447 cause
  3 blast, #448 cause 2 bullet] and the rest went on the chain. Either way the proof stands:
  one round from 11.1 u, the whole fence gone, no body contact.

The flag, the `switch_dry` rides, the rim draw, and the door[50] settle are ALL behind
`difficulty == 2 && !w.tops_up()` — the d1 real-damage run, the d3 run and the top-up walk
keep their pinned timelines byte-for-byte.

## 2. The measured path to green (each run's ledger, NF_RF_TRACE)

1. **Run 1** (spawn-stamp scan, window 120): died 7758 (19 hits) — the west-room pair never
   caught (released ~28 u away BEHIND the west wall; the window expired before first sight),
   the jog sweep's flank machinery ate 21.0 on #11, #10 never engaged (hp 20 at the death).
2. **Run 2** (window 1200): the PRE-THROW fights caught #0 (born +134 frames) and #1 (+29)
   BEFORE their first hits — both killed at zero health cost; 48.1 health at the rim. The
   `draw(0x14)` assert failed: the AR was never owned (the take equips only what Bond lacks).
3. **Run 3** (the first-seen-alive detector): EVERY release caught (15 pre-throw fights); the
   kills nearly free; 29.8 health at tick 8866 — then the walk died of THIRST: `clip 0
   spare 0` at the #447 fence shot (the P2K's 48-round pool spent on the crossing and plank
   kills; the 0x1a's leftover ~17 rounds unused in the wheel).
4. **Run 4** (the resupply backtrack): the shared resupply's nav walk WEDGED at the
   6000-tick cap — reverted.
5. **Run 5/6** (the drawn 0x1a + `draw_persist`): the one-shot kills (#10/#18/#19/#4); Bond
   died at 8580 to #5's airborne grenade (−27.7) — the great-hall race lost by ~20-30 ticks.
6. **Run 7** (the 12-tick settle + the stand): #5 killed pre-throw; 27.7 health at the fence;
   the #447 cause order flipped (the 0x1a round took the #448 diagonal first).
7. **Run 8 — GREEN**: the walk completes at **27.7 health** (10 hits for 72.3), every
   corridor and exit-hall guard killed on release (the final fights: #19, #4, #12, #14, #13 —
   all `killed true` at zero health cost).

## 3. Verification — every touched binary whole, per-binary counts (single-threaded)

The diff touches only `crates/nf-view/tests/` (the shared walker helpers are opt-in per the
field doc), so the nf-game/nf-formats/nf-audio suites are untouched by construction; every
`walkreal_*_live` level binary ran whole, plus the top-up 15 walk:

| binary | result |
| --- | --- |
| `walkreal_07000015_live` | **6 passed / 0 failed / 1 ignored** — was 5 / 1 / 1; the agent red is fixed, the d1 real-damage and top-up sisters green |
| `walkreal_0700000c_live` (the same pattern's home) | **3 / 0 / 1** |
| `walkreal_0700000c_barrelrow_live` | 1 / 0 |
| `walkreal_0700000c_eastroute_live` | 1 / 0 |
| `walkreal_07000001_live` | 7 / 0 / 1 |
| `walkreal_07000002_live` | 3 / 0 / 1 |
| `walkreal_07000003_live` | 6 / 0 / 1 |
| `walkreal_07000004_live` | 10 / 0 |
| `walkreal_07000005_live` | 14 / 0 |
| `walkreal_07000006_live` | 6 / 0 / 1 |
| `walkreal_07000007_live` | 6 / 0 |
| `walkreal_07000008_live` | 5 / 0 / 1 |
| `walkreal_07000009_live` | 8 / 0 |
| `walkreal_0700000a_live` | 13 / 0 / 2 |
| `walkreal_0700000b_live` | 7 / 0 |
| `walkreal_0700000d_live` | 2 / 0 / 1 |
| `walkreal_07000011_live` | 5 / 0 |
| `walkreal_07000011_route_live` | 1 / 0 |
| `walkreal_07000012_live` | 3 / 0 |
| `walkreal_07000013_live` | 3 / 0 |
| `walkreal_07000014_live` | 9 / 0 / 1 |
| `walkreal_07000016_live` | 14 / 0 / 2 |
| `walkreal_0700001b_live` | 17 / 0 / 2 |
| `walkreal_07000041_live` | 15 / 0 |
| `walkreal_0700004a_live` | 4 / 0 |
| `walkreal_0700004a_satchel_live` | 2 / 0 |
| `walk_07000015_live` (the top-up spot check, flags off) | 2 / 0 |

The quoted whole-binary lines verbatim:

```
=== walkreal_07000015_live (whole) ===
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 31.71s
=== walkreal_0700000c_live (whole) ===
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 33.42s
```

The `walkreal_probe*` diagnostic binaries were not run: none sets the new flag, and with the
flag off the diff's only live path is the two gated lines in `stand_and_fire` and the
flag-checked newborn scan (both no-ops at the default `false`).

## 4. What's left

1. The full 4-crate suite run on a longer window (the walkreal family above is the touched
   surface; the other three crates are untouched by construction).
2. The 08 00agent: the forecast gate + the spawn-direction pin (carried from aim-last-seven
   §6; its walk is ignored with the measured self-blast reason).
3. The 01 00agent: the hide-cycle live check / the lure stand (carried).
4. The d3 (00 Agent) 15 walk stays ignored with its measured 132.3 budget — the pre-throw
   lever is available to it next round by widening the d2 gate.

## Live checks

1. **The 15 west-room throw cycle** (carried from aim-last-seven §3, still the ground truth
   behind the window): the d2 west-room grenadier pair — the tick of the release (the
   channel/spawner write), the first launcher shot, the first grenade throw. The port now
   kills on the first-seen-alive tick; confirm the game's release precedes the first shot by
   ~50 ticks and the first throw by ≥ 400 (the measured budget the 1200 window buys).
2. **The channel-held placements' release stamp**: does the game write any observable
   release marker on a channel-released placement drone (the port's `spawn_frame` stays 0
   there [M drone.rs — only the spawner arm writes it])? The detector keys on the first
   enabled tick; a game-side stamp would tighten it.
3. **The 0x1a's grenade mode** (carried from aim-last-seven §3, unchanged).
4. **The 08 rocket's spawn direction** (carried from aim-route-6 §3, unchanged).
5. **The m8 timeline gap** (carried from aim-route-5, unchanged).

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
