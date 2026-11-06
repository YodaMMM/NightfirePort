# gas-draws-8 — report (2026-11-06)

**The task's core is DONE: the 1b Drake pair is green** (both `walk_arrival_to_drakes_defeat`
and its real-health sister, on real input from the load), and the `s/guard-lines-finish` merge
took the 16 chasers pair green with it (`walkreal_07000016_live` whole: 14/0/2i). The branch is
`origin/s/gas-rng-draws-7` d4c0933b6 + `origin/master` 2ea70f9d2 + `origin/s/guard-lines-finish`
merged, plus three fix commits. Pushed as `s/gas-draws-8` and `s/gas-rng-draws-8` (the task text
named both; they are the same commits).

## 1. Drake's hover: the port's band is RIGHT (the task's step 1, answered from the game code)

The merged-stream hover (rise from (−7.3, −1.4, 0.1) to x 9-17.8, y 4.2-7.6, states 0xc0/0xc1
[L the gas-rng-draws-7 probe]) is the game's own zero-G astronaut combat, and the port matches it
line for line:

- 0xc0/0xc1 are `NDrone2_DSTATE_AstronautCombat` (0x174458) / `AstronautCombatMove` (0x174550):
  the hold sets speed 0 + thrust off + hover anim 0x70 [M 0x174458:12-18]; the move sets
  fly_speed 2.0 (`+0x50 = 0x40000000`), the 5 s timer (`FRAME_RATE_INT·5`), the anim pick
  [M 0x174550:15-23].
- **What drives the height**: `DroneMove_AstronautCombat` (0x154618) — on the re-arm tick
  (`+0xcd8 + +0xcdc ≤ GameState`, the pair re-armed `FRAME_RATE_INT·2 + Rand_Rand(FRAME_RATE_INT·5)`
  [M :101-104]) it picks a destination **= the opponent's position + a random 3-D offset**
  (`Rand_FRand(2.0)−1` per axis, normalised, scaled `Rand_FRand(10.0)+10.0` → length 10-20 u,
  full Y included) [M :80-93], accepts it only inside a type-3 AI volume
  (`Drone_PointInAnyAIBox(pos, 3)` [M :94]), thrusts on, → 0xc1 [M :96-105]; arrival < 3.0 →
  back to 0xc0 [M :108-113]. The port's `astronaut_move_pick` (drone.rs) implements exactly
  this, draw for draw, and `move_flying` integrates `vel += normalize(goal−pos)·(1/60·fly_speed)`
  [M NDrone2_Move 0x14eff8:52-68].
- So the band y 4.2-7.6 is Bond-wedge-relative: destinations 10-20 u off Bond in random 3-D,
  filtered by 1b's kind-3 volumes, flown at 2.0. **No game-code fix** — the fix belongs to the
  fight, as the task anticipated.

## 2. The Drake pair fix: the pitched zero-g fight (test-side, opt-in)

`fight_opts` gains a `pitched` flag (only the Drake fight passes it):

- The game's zero-g view pitch is the player's own input composed body-frame into the object
  matrix every tick [M Player_ZeroG 0x192ab0:76-90 — the pitch action's −1.2566371·REC_FRAME_RATE
  rate at BL+0x30/0x34 — through RotMatrix 0x1e2368 + Concat 0x1e4a90 at :160-162], and the port
  does the same (`rot_matrix(-look·π/2, turn, 0)` then m·R, player.rs physics_tick);
  `Bond::look` returns the matrix's forward row — the same row `ray_on` checks and the laser
  round flies along (scene.rs `shot_from` aims the round at `camera + look·256`). **A player aims
  up**; the walk now does too: the pitched arm yaws AND pitches to `exposed_aim`'s point and the
  `view_level` gate is lifted for it.
- The hold never chases the target's height in this mode — the (12.7, −3.0, −1.6) under-deck
  wedge WAS that chase (Bond thrusting up into the slab). The cover re-pick runs every 30 ticks
  out to 15 u (the gas-rng-draws-7 scan's clear-ray holds sit 10-15 u out), scored with the
  same prio-blind +50 so the hold keeps a clear 3-D ray to Drake.
- The stale module note ("only the level shot lands where the view looks") is amended in the
  file — it predates the current m·R composition.
- Result: **walkreal_0700001b_live whole: 17 passed / 0 failed / 2 ignored** (128.45 s;
  was 15/2+2i). The d2/d3 real-health sisters keep their measured ignores.

## 3. The guard-lines-finish merge (the 16 chasers pair + the 01 levers, not redone)

Merged `origin/s/guard-lines-finish`. Three conflicts, resolved by hand:

- `crates/nf-game/tests/ai_live.rs`: kept HEAD's seed 4 (the gas branch's documented re-pick —
  every `Gas_Init` puff consumes the game's own draws, the stream shifts on every shot).
- `crates/nf-view/tests/walkreal_07000001_live.rs`: the two sides had evolved the same doorway
  region differently — the gas side's `far_field_guards`/`far_field_sweep` + the inlined
  far-start-aware lip loop (all difficulties), the guard side's `door_lip_scan`/`door_lip_fight`
  extraction + the d3-gated escort-crossing re-scan poll. **Both kept**: the first lip loop is
  the gas side's (far-start break + mid-lip sweep), the escort re-scan poll is the guard side's,
  driving the extracted helpers. The guard side's d3 pre-hall resupply stock merged clean.
- `docs/WORKLOG.md` (add/add — the file does not exist on master): both sides' entries kept in
  full, nothing reordered.

16's punch test was still red after the merge with a NEW gas-stream ledger: the plain route
bled **86.8 of 100** through the corridor before the window (the gallery/ring pair's plinks at
66-90 u the reactive fights never answer) and the counter-fire loop shot the chaser out of her
own window (`hold` lets the taps run below 55 health at 8-12 u — she was dead at the 1843
assert). Fix: the punch test rides the defeat walk's full route economy
(`walk_to_the_chaser_fight(w, true, false)` — the ring legs' level clears and the pit stand;
`station_armour` stays false so the punch pin's armour-0 window holds). **walkreal_07000016_live
whole: 14 passed / 0 failed / 2 ignored** (79.31 s) — both chasers tests green in the same run;
the d2/d3 sisters keep their measured ignores.

## 4. The 01 d3 walk: still red, a new death point, ledgered

`walkreal_07000001_live` whole: **7 passed / 1 failed** (42.38 s). The guard branch's two d3
levers (the pre-hall resupply stock, the lip re-scan polls) are all in and armed — their tree
went 8/0 — but on the gas stream the d3 walk dies EARLIER, at tick **5974** at (−23.37, 6.47,
15.18), crossing the ch-0xF slab (the deliberate ambush pop): drone #3 (skin 0x5000038, weapon
0x12, hp 24 still) at (−30.9, 6.3, 13.5), 7.7 u, poured 29.2 over 8 hits; Bond arrived at the
crossing with ~10 health (18 hits for 100.0 total: the doorway fight's single −24.0 from #15,
the far pair's 2 hits for 24.0, the threshold fights' 21.0). The lever is upstream fight
economy on the new rolls (the doorway trade and an earlier far-pair sweep), d3-gated — not
fitted in this window.

## 5. The master merge's wave × the gas stream: 14 reds on 11 levels, classified not fixed

Master advanced past gas-7's merge point (402e4f57c → 2ea70f9d2) with a walk-un-ignore wave
(06 +104, 13 +190, 0d +370, 07 +122, 03's keep-off, 01/16/1b edits) and two src changes
(drone.rs +13 "Prop-0x52 cover-lost fail on first attack"; player.rs/scene.rs jump-landing
sounds). All of it is green on master and was green on gas-7's own tree; **on the gas stream
the intersection reds 14 tests on 11 levels** (my branch changes no src and only the 1b/16 test
files — by elimination the interaction is master's wave with the gas draws; [S] for the
per-test attribution). Every walkreal binary ran WHOLE on the final tree
(`--test-threads=1`, NF_ISO, slim ISO):

| binary | gas-7's report | this tree |
|---|---|---|
| walkreal_07000001_live | 7/1 | 7/1 (the d3 walk, §4) |
| walkreal_07000002_live | 3/0 + 1i | 3/0 + 1i (71.65 s) |
| walkreal_07000003_live | 6/0 + 1i | 4/2 + 2i — `walk_the_full_route_and_complete_every_objective` ("servant 1's captor #11 died to the chase") and `_real_damage_agent` (Bond died tick 31789 at (26.0, 6.2, 6.75)) |
| walkreal_07000004_live | 10/0 | 10/0 (10.86 s) |
| walkreal_07000005_live | 14/0 | 13/1 — `the_door_band_exit_chains_to_07000006_walked_real_damage_00agent` |
| walkreal_07000006_live | 5/0 + 2i | 4/2 + 1i (master un-ignored walks; 2 red on this stream) |
| walkreal_07000007_live | 5/0 | 6/0 (20.02 s, master's +122 rides green) |
| walkreal_07000008_live | 6/0 | 5/1 |
| walkreal_07000009_live | 8/0 | 8/0 (15.59 s) |
| walkreal_0700000a_live | 13/0 + 2i | 13/0 + 2i (52.26 s) |
| walkreal_0700000b_live | 7/0 | 7/0 (42.45 s) |
| walkreal_0700000c_live | 3/0 + 1i | 2/1 + 1i |
| walkreal_0700000c_barrelrow_live | 1/0 | 1/0 (7.47 s) |
| walkreal_0700000c_eastroute_live | 1/0 | 1/0 (10.93 s) |
| walkreal_0700000d_live | 1/0 + 2i | 1/1 + 1i (master's +370 rides; 1 red on this stream) |
| walkreal_07000011_live | 3/0 + 1i | 2/1 + 1i |
| walkreal_07000011_route_live | 1/0 | 1/0 (6.38 s) |
| walkreal_07000012_live | 3/0 | 2/1 |
| walkreal_07000013_live | 2/0 + 1i | 1/2 (master's +190 rides; 2 red on this stream) |
| walkreal_07000014_live | 9/0 + 1i | 8/1 + 1i |
| walkreal_07000015_live | 6/0 + 1i | 5/1 + 1i |
| walkreal_07000041_live | 15/0 | 15/0 (25.75 s) |
| walkreal_0700004a_live | 4/0 | 4/0 (23.84 s) |
| walkreal_0700004a_satchel_live | 2/0 | 2/0 (9.10 s) |
| **walkreal_0700001b_live** | 15/2 + 2i | **17/0 + 2i — the Drake pair, fixed** |
| **walkreal_07000016_live** | 12/2 + 2i | **14/0 + 2i — the chasers pair, fixed** |

`nf-game -- --test-threads=1`: every binary green except **grenade_live**
(`a_guard_in_aim_stand_fire_0x68_flees_his_own_landed_frag`, 3/1): the merged-stream frag
lands 11.52 u from the thrower (the 12 u scan's edge) and the 0x68 guard does not leave his
state within the 10-tick window. **Inherited, not this branch's regression**: the identical
failure reproduces on a pristine `origin/s/gas-rng-draws-7` worktree (run this session,
3/1) — the gas series' streams shift the frag's scatter; draws-7's session never ran the
nf-game test binaries. nf-formats/nf-audio untouched by this branch; the full four-crate
suite did not fit the window again (the standing ~3 h measurement) — every walkreal binary
ran whole instead, plus all of nf-game.

## What's left

1. **The master-wave × gas-stream reds (§5)** — 14 tests on 11 levels, each needing measured
   iteration on the merged stream (the 03 pair and 05/08/0c/0d/11/12/13/14/15's singles; 06's
   two include master's freshly un-ignored walks). Next session's first duty.
2. **The 01 d3 walk** (§4's ledger) — upstream economy on the new rolls, d3-gated.
3. **grenade_live's 0x68 flee** — the gas-stream frag scatter vs the 12 u scan; the gas
   series' follow-up.
4. The d2/d3 1b and 16 sisters keep their measured ignores (untouched).

## Live checks for the main agent

1. 1b's Drake float on the merged stream (carried from gas-rng-draws-7): his obj position every
   second from the fight gate — does the real game's Drake also take the high hover band
   (y 4-7.6 over the deck)? The port's band now matches the game's own strafe code [M
   0x154618:80-105]; a real-game low float would mean the port's float physics (not the strafe
   pick) drifts.
2. The same fight: does the real game's Drake descend toward Bond's height to fire, or hold the
   band? The port fires from the band (the fight's `astronaut_move_pick` runs in both states).

## Verification notes

- No `crates/*/src/` file changed on this branch (the merges' src is master's own and the gas
  series' own, already landed); the fixes are test-side only, in the two files the task owns.
- The pitched fight is opt-in (a `fight_opts` flag only the Drake fight passes); the 16 punch
  economy is the defeat walk's own path with `station_armour` false; no common/ helper changed.
- No test was weakened: the two 16 punch asserts that matter (armour-0 at the window, the
  pinned 20.0/star value set) are untouched; the 1b module-note amendment is documented above.
- The token never left the clone URL; `git status --short` checked before every commit.

## Usage

not visible to me (no token, credit or context-window figures are exposed to this session;
nothing in the sandbox reports them).
