# guard-repin-master — the five reds triaged; two fixed, three analysed with levers landed or proposed

Branch `s/guard-repin-master` (from `origin/s/guard-on-master` 6f8ce18bb — the guard per-goal
arrival radii + the guard-rounds glass pass on current master). The game change is [M]-cited
throughout: `goal_arrival` = Drone+0x8EC per goal radius [M 0x14b300:579, 0x158b10, 0x1517d8:49-52;
the setters 0x152548:174-181 / 0x152958:207-208 / 0x152250 / 0x153008 / 0x153a30 / 0x154aa8 /
0x155e10 / 0x1543f8 / 0x15add0], `attack_mode` written for every drone at creation
[M 0x14cbd8:22-23 via 0x14c600:80 / 0x14cfa0:16; the astronaut 0x14b300:725], and the guard
rounds' glass pass from weapon_data+0x6c bit 0x800 [M Bullet_update 0x12e7b0 disasm
0x12f048-0x12f058; Collide_Update 0x1ea398:216-229]. Nothing in the five reds says the game
code is wrong: every failure is the walks' tuning to the OLD guard behaviour (the pre-radii
2.0 stops, the rounds that died on glass). No [L] pin was loosened, no ignore added, no game
code changed.

## 1. walkreal_07000003_live — real_damage_agent (and the ignored 00agent sister) — NOT FIXED

Verdict: walk tuned to the old behaviour. The far shooters sit outside the plain return-fire
window and their rounds now come through glass. Measured: the agent death at tick 51805 at
(23.09, 6.37, 10.54) during the servant-0 door opening — #11 (skin 0x5000038, at (13.6, 6.3,
−28.5), **d 40.2**) landed **56.0 in 5 hits**, #10 (19.4, 6.3, −19.6, d 30.3) 9.4 in 5, plus
#5 (0x500003a, weapon 0x1c) closing to d 4.2 for 27.6 in 18 — 29 hits for 100.0, 12 fights.
The 00agent (pre-existing ignore, not in the five) died at 662926 at (38.49, 6.37, 9.39) — a
re-fight marathon — with #2 (0x5000038, weapon 0xe, hp 22, d 10.2) landing 12.0/hit.
The fights' plain shooters window is 20 u (the 06 doc's own law), so the d 30-40 glass volleys
are never answered. Lever PROPOSED, not landed (time box): the 07000006 file's
`stand_shooters_down` leg-end sweeps (its dock-guard law: the posts d 27-35 "OUTSIDE the
fights' plain 20 u shooters window") applied to the servant/exit legs, file-local, d-gated.

## 2. walkreal_07000013_live — warm chain real_damage — cap FIXED, death remains

Verdict: mixed. (a) The "the lobby west mid" 6000-tick cap was the fights' displacement: the
guards' pursuit now closes to 1.0-2.0 [M 0x153a30:141-142,168-169], the stands' back-off
geometry moved, and Bond parked in the lobby pocket at (0.60, −368.30, 25.66) — the direct
segment to the west-mid clips the lounge furniture corner at ~(0.3, 26.3) (the measured dump:
segment_to_target hit at t 0.042, d 15.78, every nearby guard dead). FIX LANDED: the
"13: the lobby mid re-anchor" waypoint (2.5, −368.4, 30.5) between the east lane and the west
mid — the walk passed the west-mid for the first time on this branch. (b) The re-anchor saved
~5000 ticks, which moved the walk's whole timeline up: #5 (0x5000004 at (−14.8, −368.4, 45.6))
landed 28.0 in 8 hits across the re-anchor + west-mid legs and Bond reached the security room
at 21 hp, where the room guard #8 (weapon 0x10, posted at (−26.9, −368.4, 29.5)) met the
doorway crossing point-blank — one 31.5 blast at d 2.9, death at tick 3006 at (−26.23, −368.30,
32.30). Two more levers landed and measured WITHOUT effect (the byte-identical death): the
civ-unignore-2 ray-gated sweep at the re-anchor end and at the OPEN security-room door
threshold (radius 12) — #8's rays never open from any of those spots. What's left: restore the
lobby budget (master's fight shape answered the west shooters before the west legs — the
west-mid sweep's spot may need to move west/north, or #8 needs a doorway aim law that looks
DOWN the room). The mystery death at tick 2627 at (208.62, 1.04, −229.64) does NOT belong to
13: it is **0d's** `walk_arrival_to_the_c5_door_real_damage` (the blind strip east 3→4 transit;
13's coordinates are all negative; the 13 binary's log carries no such line).

## 3. walkreal_0700000d_live — c5 door real_damage (+agent) — NOT FIXED

Verdict: walk tuned to the glass shield. The "blind strip" is no longer blind: the bay snipers
(#10 skin 0x5000093, #18/#32 0x5000090, weapon 0x1e, posted at y 12-14 at (197.8-214.7, −260..−289),
d 18-61) now shoot THROUGH the bay glass [M 0x12f048-0x12f058] — the test's own zero-hit
transit promise ("zero-hit while moving in every run") was a glass-shield property. Measured:
the d1 death at tick 2627 at (208.62, 1.04, −229.64) (16.0 × 4 from #16 d 36.8 and #18 d 61.2),
the agent death at 5235 — #11 (0x5000093, weapon 0x1e, d 54.5) landed **100.0 in 6 hits**
(28.0/hit) with **0 return-fire fights** (Bond never answered — beyond the 20 u window).
Lever landed: the strip-mouth #10 kill dwell 600 → 180 ticks — the kill no longer lands at all
(600 ticks of converged, ray-CLEAR shots at d 63.6 scored zero, which itself needs a look:
the guards' boxes/positions may sit differently under the radii work), so the dwell was pure
exposure (38.0 taken in the failed attempt). Measured effect: the d1 run moved from the 2627
strip death to a 9300 drop-phase death ("the drop must land on the annex", 94.0 taken — the
transit survived). What's left: an answer for the bay-sniper class (a closer stand that lands,
or the climb legs' own volleys answered) — the strip itself stays a exposed lane under the
game's glass law.

## 4. walkreal_07000006_live — castle interior real_damage (+agent) — FIXED

Verdict: walk tuned to the old cover stalls. The guards' find_cover arrival is now 0.5/1.0
[M 0x152548:174-181] — the strip posts SETTLE INTO cover instead of stalling 2.0 short in the
open, so the plain return fire bleeds out against covered shooters: the d1 death at tick 5217
at (20.27, 41.13, −37.97) — #20 (0x500002f) closed to d 8.8 and landed 30.2 in 9 hits, #21
16.5 in 10, #19 18.0 in 1 (d 21.5), #22 27.0 in 1 (d 33.6). FIX LANDED: the file's own d2
lever (`ledge_stands` aimed stands, the ray-gated recent-hitter sweep) extended to the d1
real-damage walk through a `stands` parameter (`walk_the_castle_interior_to_the_party_at_stands`);
the PLAIN d1 walk keeps master's timeline byte-identical (its wrapper is untouched). Measured:
both real_damage sisters PASS (the binary 6 passed / 1 failed — the 1 is the 00agent's own
pre-existing `#[ignore]` red, not one of the five).

## 5. walkreal_07000008_live — whole mission real_damage_00agent — pane red FIXED, station red remains

Two stacked reds. (a) FIXED: "the shoot-out must kill every end pane, 1 left" — pane 2's five
pulls were DRY: the cabin fights' ammo drift (which moved with the guard radii) left the clip
empty — pane 1's log line reads "SHOT OUT after 1 rounds, clip 0 spare 33" and pane 2's
"STILL GLAZED after 5 rounds, clip 18 spare 15" (the walker's own mid-loop reload landed too
late). FIX LANDED: `w.reload_full()` (the walker's additive R law — the game's R held to a
full clip [M Player_WeaponFiring's reload branch], real input, no ammo writes) before the
shoot-out. Measured: "after the shoot-out: 0 end panes still glazed" ✓. (b) REMAINS: the
top-station glass volleys — the branch's own drone.rs comment predicted this exact red
("arming the pass here turned walk_the_whole_mission_on_real_input_real_damage_agent red at
tick 25178 (#23/#19 volleys through the top-station glass) — the real-health route cost is
re-pinned with the 08 station walks"). Measured now: the 00agent died at tick 27596 to a hurt
volume after the sniper drain — #20 (d 59.2) 36.0 in 3, #23 (d 71.0) 9.0 in 3, #4 (d 81.4)
16.0 in 2, #14 (d 97.5) 4.0 in 1. The station legs need the re-pin the branch declared
(the stands' GLASS_SIGHT fights run — 14 fights — but the d 59-97 exchange still loses).

## The finish battery (whole binaries, --include-ignored, --test-threads=1, slim ISO)

- walkreal_07000006_live: **6 passed / 1 failed** (33.3 s) — the fail is the 00agent's own
  pre-existing ignore; the listed red is green.
- walkreal_07000013_live: **2 passed / 1 failed** (48.7 s) — the d3 real_damage death above.
- walkreal_0700000d_live: **1 passed / 2 failed** (45.6 s) — the bay-sniper class above.
- walkreal_07000008_live: **5 passed / 1 failed** (194.5 s) — the station-volleys red above.
- walkreal_07000003_live: **5 passed / 2 failed** (752.2 s) — the agent red above (the 00agent
  fail is its pre-existing ignore).
- walkreal_07000001_live / walkreal_07000016_live / walkreal_0700001b_live /
  enemy_accuracy_live: see the WORKLOG tail for the counts as they landed (this report was
  written while they ran).

## What's left (next window)

1. 03: the servant/exit leg-end shooter sweeps (the 06 `stand_shooters_down` law).
2. 13: the lobby budget — the west shooters answered before the west legs, and a doorway
   aim law that looks DOWN the security room.
3. 0d: the bay-sniper answer (the strip-mouth far shots stopped landing — root-cause the
   guard box/position shift first, then the closer-stand law).
4. 08: the station-legs re-pin the branch's own diff declared.
5. The whole-suite re-run before merge (rule 5).

## Live checks for the main agent

1. The bay guard #10 (0700000d placement (214.21, 12.09, −289.44)): read Drone+0x8EC
   (goal_arrival) and his state while the player stands at the strip mouth (196, 0.5, −228.5)
   — the port's converged ray-clear shots at d 63.6 scored zero on this branch; check his
   hit_boxes world position vs the placement (the radii work may settle him elsewhere).
2. The guard glass pass on the 08 top station: watch #23/#19's rounds cross the station glass
   during the platform-mouth wave — confirm the game's rounds pass intact panes there
   [M 0x12f048-0x12f058] and measure the real exchange rate at d 59-97.

## Usage

- Session token/context/cost figures: not visible to me (no token or cost display is exposed
  in this environment).
