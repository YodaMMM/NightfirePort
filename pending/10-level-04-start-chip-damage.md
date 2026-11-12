## Your task: `level-04-start-chip-damage`, the 07000004 start of play hurts Bond as fast as the game does

Bond stands still where play starts on 07000004 (after the intro: (-3.87, 5.46, 30.30) yaw 1.658 in both). At difficulty 1 (GameState+0x28 = 0x2a3790 = 1) the two lose health very differently [L PCSX2, health = BLData+0x894 sampled every 30 frames from the first play frame, no input]:

| s into play | game | port (load_with_carry_difficulty(.., 1), step_bond idle) |
|---|---|---|
| 0.0-0.5 | 100 | 100 |
| 1.0 | 97.5 | 98.5 |
| 2.0 | 97.5 | 86.5 |
| 3.0 | 97.5 | 76.0 |
| 4.5 | 95.0 | 65.5 |
| 8.0 | 70.0 (one 25 hit; Bond turned to yaw 2.558) | 64.0 |
| 12.8 | 68.75 | 35.25 (one 25 hit, Bond lifted to y 5.91) |

The port's chip damage in the first 5 s is about 7x the game's. Find which drones hurt Bond in that window in the port (shooter index, weapon, hit chance roll, range) and in the game (breakpoint the damage path, e.g. the hit-chance 0x179740 / damage 0x1902c8 callers, with Bond's health held only AFTER logging each event), name the arm that differs, fix it against the decompile, and pin it with a test at difficulty 1 that checks the first-5-s damage against these [L] numbers (tolerances stated). The 25-point hit at ~8 s in the game (Bond also turns) vs ~12.8 s in the port is the second item: what lands it, and why the port's comes later and lifts Bond.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-04-start-chip-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
## Main agent addendum (2026-11-12): two more start-of-play differences on 07000004 [L PCSX2 frames audits-side, same direct load, d1]
1. **The assassin's position.** At ~frame 1440 after the load (play starts at port tick 1330) the game's ninja stands ahead of Bond in the courtyard, in view, red goggles lit, while the port's ninja (drone #0, skin 0x0500001e, state 0xae) has run to (-5.23, 5.34, 28.32) - right beside Bond, out of view - and is the one landing the early hits. Trace the ninja's opening states in both (Drone+0x10C each 30 frames in the game) - this is most likely the same root as the damage gap above.
2. **The objective banner.** SETTLED - not a bug: on the real 03 -> 04 hand-off the game shows OBJECTIVE COMPLETE then NEW OBJECTIVE / Defeat the assassin, exactly the port (03 obj 2 completes from 04's story script ch 0x53). Skip it. On the first play tick the port posts two objective events: record 6 objective 2 (state 3 -> 4, "OBJECTIVE COMPLETE") and record 7 objective 0 (state 1 -> 3, "Defeat the assassin"); the HUD shows "OBJECTIVE COMPLETE" for 3 s. The game shows "NEW OBJECTIVE / Defeat the assassin" at the same moment. Find which is wrong: does record 6's objective 2 complete here in the game (read the LEVELS record states live), and if both post, which message `HUD_UpdateStatusPane` 0x19c310 shows first (Text_AddMsg 0x1b0c30 insertion order). Pin it with a test.

---
## Main agent addendum 2 (2026-11-12): the ninja's state trace, game vs port (d1, frames from the load, no input)
Game [L PCSX2, drone 0x1b45420 Drone+0x10C, Bond BLData+0x894]: 0xa9 (intro) -> 0xae @1327 -> **0xaa @1461 at (1.76, 5.31, 30.09)** (a jump ~4.4 u EAST of its start (-2.64, 5.34, 30.08)) -> hits of **1.25** at 1474, 1477, 1538, 1605, 1607, 1661 (~one per 60 frames) -> 0xab/0xad @1666 -> 0xac @1779 moving onto Bond -> **-25 @1823** (melee) -> 0xae @1976 -> 0xa9 @2092 back east at (1.89, 5.32, 30.97).
Port [load_with_carry_difficulty(.., 1)]: 0xae @1332 -> **0xaa @1464 at (-5.50, 5.34, 28.15)** (WEST/south, not east) -> hits of **1.5 every 6 ticks** in bursts of 6 (1465-1495, 1516-1546, 1567-1597, 1618-1642), ~20-tick gaps = 24 hits in 180 ticks -> 0xae @1644 -> 0xaa again @1757 at (-6.54, 5.41, 25.82), hits of 0.75 -> 0xad @1937 -> 0xac @2048 -> -25 @2090.
So three things differ in state 0xaa: (a) the jump target (east in the game, west in the port), (b) the attack cadence (game ~1 hit / 60 frames, port 1 hit / 6 ticks in bursts - the throw/fire timer or the burst count), (c) the per-hit damage at d1 (1.25 vs 1.5, then 0.75). Read NDrone2's 0xaa state handler (the ninja's throw state) and its fire timer; pin the cadence and damage against this trace.

---
(main agent claim released 2026-11-12 - see addendum 3)

---
## Main agent addendum 3 (2026-11-12): ROOT CAUSE found; released back (claim withdrawn)
[L PCSX2 07000004 d1, the ninja drone 0x1b45420]: through the intro the drone already holds state **0xa9** (from the load) with Drone+0x1a0 (distance) **11.88** - Bond's intro-time spot. At frame 1326 +0x4F8 gains 0x100 (0x80010042 -> 0x800901e2, the cut-scene's Drone_EnableAll(1)); at **1327** the distance reads **1.25** (Bond at the hand-back spot (-3.87, 5.46, 30.30)) and the state goes **straight to 0xae**, obj+0x54 yaw staying **-1.572** for the whole flip -> she flips EAST (-2.64 -> +1.76 x) away from Bond.
Port (master a275e4a32): the drone is state **0** through the intro (GLOBAL ENTER deferred until enabled), and on the release tick (port frame 1332) the walk runs GLOBAL -> 0xa9 ENTER+UPDATE with the **stale 11.88 / bearing 0.389**, because Bond's hand-back position is applied AFTER the drones' pass that tick (the `player` snapshot the drones tick reads is pre-hand-back). 0xa9's MoveToObject aims at the old spot (desired yaw 0.955), the backflip 0xae (whose UPDATE sets no angle [M 0x16d430]) keeps turning toward 0.955, and the flip lands WEST at (-5.50, 5.34, 28.15) 2.6 u from Bond -> the close-range 0xaa bursts.
Proven: deferring that drone's first update by one tick (skip the walk's control for a drone re-enabled this tick by Drone_EnableAll(1) when the pre-pass gave it no opponent info) reproduces the game exactly - 0xae with dist 1.25, yaw -1.572, the flip to (1.96, 5.34, 30.08) (game 1.76), then 0xad / 0xac sword -25 / back east 0xa9 / 0xaa chip hits - and assassin_07000004_live passes once its `settle` ticks one more time after the release. BUT the blanket skip turned 3 walkreal_07000001_live walks red (walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent, the_plate_refuses_bond_alone, following_mayhew_down_completes_the_level) - 01 also releases its drones from an intro. The attempt's diff is at the main agent's audits (ask if needed); not committed anywhere.
The faithful fix is likely ORDER, not a skip: in the game the script that ends the cut-scene moves Bond (hand-back) and re-enables the drones in one Script_Free, and each drone reads Drone_GetOpponentInfo inside its own control func at its walk position - so whatever runs after the release sees the hand-back Bond. Make the port's hand-back position visible to the drones' pass on the release tick (or refresh opponent info from the post-hand-back Bond at each drone's walk turn), check the 04 trace above, and re-tune/verify the 01 walks against PCSX2 if they move. Residual after the order fix: 0xaa chip hits 0.75 (port) vs 1.25 (game) at d1 - check Drone_ModBulletDamage 0x139f50's distance term against the game at the same range.

---
claimed-by: main agent 2026-11-12 (the order fix is in test: Bond's hand-back position reaches each drone at its walk turn; full suite running)
