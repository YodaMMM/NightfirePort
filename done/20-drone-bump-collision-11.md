## Your task: `drone-bump-collision-11`, land the bot-capsule arming without breaking the other levels

`origin/s/drone-bump-collision-10` (its report is on the branch) merged with master makes these tests RED, reproducibly (all green on master without it): walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit, walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit, escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens, fix_party_mission_live::the_07000008_safe_chain_works_in_play, playtest_spots_live::spots_07000008_safe_bolts_in_reach, interactables_0700000a_live::armour_pickup_taken_by_walking_over_it, fix_drone3_live::the_truck_driver_reaches_his_idle_state_and_never_twists, pickups08_live (4 tests), fix_safe08_live::the_safe_pickups_take_after_the_safe_opens, speech_facial_fallback_live::a_present_script_talker_plays_through_the_facial_and_advances, fix_06_driver_live::the_scared_cower_keeps_him_in_place_on_the_floor, rook_door_06_walk_live::driver_walks_his_mission_route_after_the_sit_script. Continue from -10 (merge `origin/master` first). For each red, find which of -10's changes moves it (the route-walk boundary clamp replacing probe_move, the push in every stance, the kind-word arming, the door-zone opponent gate) and what the game does at that step [M/D/L]. A test calibrated on old port behaviour is re-derived only with evidence; a real regression is fixed in the port. Run the FULL suite on the final tree and quote its total line - it must have no new reds.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-29T12:46Z

---
## Report (done, 2026-10-29T13:14Z)

# drone-bump-collision-11 — the boundary clamp was dead code; three reds healed

## What was done

**Root cause found for the driver-family reds: `-6..-10`'s route-walk boundary clamp never
runs in the game.** The clamp gate is `+0x4F8 & 1` [M `DroneAnim_PreTransCallback`
0x13b490:120; `DroneMove_SetBoundryFlags` 0x14f140 computes the +800 flag word under the same
gate], and **no instruction in ACTION.ELF sets that bit**:
- `NDrone2_DefaultInit` 0x14b300:60 spawns every drone with it CLEAR —
  `+0x4F8 = (+0x4F8 & 0xffea05e2) | 0x801e2`, bit 0 clear on both masks;
- a disassembly sweep of **all 319 `sw .., 0x4F8(..)` sites** (work/elf/ACTION.ELF, mipsel,
  symtab-resolved via work/action-symtab.tsv, scripts/scan_4f8b4.py) shows every
  read-modify-write ORs only bits ≥ 2: the vision bit 0x10000
  (`DroneVision_ConsiderAlerted`+0x448 — the `ori .., 0x7`-shaped hits there are LEVEL-HASH
  constants 0x070000XX feeding a `bne`, not the stored register), 0x2000
  (`AstronautLaunch`+0x494), bit 2 (`AbseilDeath`+0x69c; its other write ANDs 0xffffffdf =
  a bit-5 clear);
- no SB/SH targets the word (scripts/scan_4f8b4.py scans 0x28/0x29/0x2B).

The port had conflated the clamp gate with `bounded` (the game's +0x4F8 **bit 9**, set by
DefaultInit's OR 0x200) and clamped every route walk from spawn. The game's own probe
geometry proves the driver is unclamped: his park spot is walled on three sides within 0.8 of
his feet [D the parsed nav boundary walls; probe_driver11.rs], so a bit-0-set bot there would
be pinned by the game's own four-probe code [M 0x14f140] — yet the driver walks his whole
mission route [L 06-firstsight.trace 148014-148247].

**Fix (drone.rs)**: the route-walk arm applies the root-motion step unclamped [M
`AnimFrameResolve` 0x1c35f8:425-432 adds the callback's vector to obj+0x30 unconditionally];
the post-tick nav-bounds push (0x157bd0) remains the horizontal constraint, and `probe_move`
stays gone (the -6 wedge fix stands). Healed, all green on this tree:
- fix_drone3_live::the_truck_driver_reaches_his_idle_state_and_never_twists (was: pinned at
  route idx 0 forever, blocked_reach 0.02/tick, never 0x34)
- fix_06_driver_live::the_scared_cower_keeps_him_in_place_on_the_floor
- rook_door_06_walk_live::driver_walks_his_mission_route_after_the_sit_script
- the clamp's own beneficiaries stay green: probe_07_route 1/0, walkreal_07000007_live 2/0
  (the -10 wedge fix never needed the clamp — the unclamped step plus the post-tick push
  carries the 07 hall legs).

**Second fix (player.rs): the bot push's y is zeroed for the grounded stance.** The Jointy
push (now stance-free per -10's [M Collide_Jointy 0x1ec180:229-266] reading) lands after the
floor refine, and the port has no next-tick floor re-resolve: with the guests wandering into
07000008's GuardRoom, a guest capsule beside Bond projected him THROUGH the floor at the
safe-bolt stand (y 42.15 → 7.44 in 160 ticks [L probe_safe11.rs]). The bot push accumulates
in its own variable now; the static head-bump/floor push keeps its y (the first draft zeroed
the shared accumulator and the lib step-climb pin `walks_at_the_game_speed_...` caught it —
fixed before push). Airborne keeps the full push (the sleeper drop). nf-game lib: **585
passed / 0 failed**.

## What ran (`--test-threads=1`, NF_ISO set)

- `cargo test --release -p nf-game --lib`: **585 passed / 0 failed**.
- fix_drone3_live: 5/0 (the named red green; the file's other 4 pass).
- fix_06_driver_live 2/0, rook_door_06_walk_live 1/0.
- probe_07_route 1/0, walkreal_07000007_live 2/0.
- pickups08_live: still 1/4 — see below.

## What's left (the honest list)

- **pickups08_live ×4 + fix_safe08_live + playtest_spots_live + fix_party_mission_live**: the
  burn/pickup stands on 07000008 still fail — with the grounded-y fix in, Bond is now shoved
  HORIZONTALLY off the safe-bolt aim lines (probe_safe11: the first post-poke tick moves him
  +1.1 x; the guests at (161.9, −116.1) overlap his capsule). The open question the next
  window must settle with evidence: do the disc's kind-8 guest routes actually reach the
  GuardRoom on 07000008 [D the placements' route params], or does the port's unclamped route
  walk overshoot the game's wander extent (the post-tick bounds push 0x157bd0's port shape)?
  If the guests' reach is game-true, the tests' stand-spot calibration is re-derived (the
  game's player would be shoved too); if not, the route walk's constraint is the fix.
- **walkreal_07000014_live, walkreal_07000002_live, escort_follow_07000001_live,
  interactables_0700000a_live, speech_facial_fallback_live**: not yet reproduced this window
  (the driver-family + 08 families were prioritised). The clamp removal likely moves the
  escort lead's park (he walks further unclamped — the test's gallery-wedge special cases are
  calibrated to -10's clamped parks); re-derive against the route data with the same
  [L trace / D route] method the driver reds used.
- The full-suite total line: not produced in this window (the background runner dies between
  tool calls in this sandbox — three launches, three kills; suite.out stayed empty). The
  per-crate runs above are the quoted evidence; the next window runs
  `tools/test/suite.py` (or the chunk pattern -10 used) on this tree.

## Live checks for the main agent

1. 07000006 fresh boot: confirm the truck driver walks his full mission route to the truck's
   far side and idles (the port now does; the -10 tree pinned him at his park).
2. 07000008 fresh boot at the safe: after the Q-Laser opens the safe, do the wandering party
   guests ever enter the GuardRoom and shove the player off the bolts? (Settles whether the
   pickups08 red is a port overshoot or a test recalibration.)

## Usage

not visible to me
