# level-0c-route-teleports — the report

Branch `s/level-0c-route-teleports` (from origin/master e79794da). Work commit 41ac55f9 + this report.

## What the task was

`walkreal_0700000c_live`'s full walk teleports Bond three times: out of a shaft he can drop into
but not climb out of, onto a mezzanine reached (as far as the tests show) only by the Q-Claw from
the warehouse, and onto an east deck no collision connects to. For each: find the game's route
[D collision and placements: ladders, grapple points (Q-Claw targets), lifts, doors; M the
classes the route uses], fix any port gap with the address cited and walk it on real input;
where the route is a grapple, drive the real Q-Claw. Delete each replaced teleport; keep the
rest with a sister assert and name the live check.

## What was done

The task's placement checklist (ladders, grapple points, lifts, doors) was run against the
disc's own data for all three sites, and the Q-Claw route was IMPLEMENTED and driven end-to-end
for the mezzanine before the disc's collision closed it (the walk to the claw spot arrived at
(13.78, 7.40, −142.10) on real input; the pick ray was blocked at t = 0.037, hit (15.56, 8.25,
−142.21); the follow-up segment_hit scans mapped the wall). The findings, per site:

1. **The trench/shaft exit** (common/walkreal0c.rs, the stand-in at [5.0, 12.35, −112.9]) — KEPT.
   The prior seals stand ([D] the south wall slab z −112.34..−112.363 with the crouch capsule
   stalling at −111.813 [M 0x194df0 case 3]; [L 2026-10-26 hatch-3] the game holds the body too;
   no 0x20 Break placement on 0c [D census]; the vent hole 3.2 m over the floor, unjumpable [M
   the 0.78 apex]). THIS session added the grapple check: the level's five grapple points
   [D #308/#346/#500/#508/#511, the sim dump] are gp0 (61.13, 12.28, −143.48), gp1 (77.45,
   12.12, −143.95), gp2 (−4.60, 13.16, −189.71), gp3 (39.42, 23.06, −221.77), gp4 (54.27,
   15.35, −237.02) — the nearest to the shaft is gp2 at ~80 m, out of the Q-Claw 0x50's 50.0
   range [L ee_start_05 weapon_data+0x1c; M Bullet_update 0x12e7b0:158]; no gp hangs at the
   vent-hole lip. Sisters inline (the pocket wall, the pocket ceiling, the ray-blindness pin,
   the deterministic stall). **Live check (named): the real post-drop flow** — whether the game
   ever moves the body out of the shaft (a script, a checkpoint nudge, an exit the static data
   hides), and whether its capsule stops at −111.813 like the port's.
2. **The mezzanine reach** (common/walkreal0c.rs, the stand-in at [54.0, 5.8, −158.0]) — KEPT,
   reason hardened. The walking closures stand ([L 2026-10-31] fresh-boot runs; the port pins
   the same stops — walkreal_probe0c_barrelrow_live, walkreal_probe0c_lip_live; the deck floods
   as a separate statics component). THIS session drove the Q-Claw route as far as the data
   allows: the claw IS in the boot loadout (0x50 [D the loadout dump]; the Q-Claw records
   0x50/0x51 [L bank "Grapple"; M Bullet_Delete 0x12e660:33]); the fire-time pick needs the aim
   ray through the point's sphere (r 0.64043313 [D #308] + the bolt radius 1.0 [M Bullet_init
   0x12c5a0:92]) within 50.0 [L ee_start_05; M Bullet_update 0x12e7b0:158]; gp0 is ~48 m from
   the display ground's east edge — IN range — but the east wall (x 14.8..15.6) is SOLID from
   y 6.5 up past 15.5 with no z gap in −136..−150 (segment_hit scans, y 3.0/5.0 clear only
   under the floor complex) — the game's bolt dies on it exactly like the driven ray; the hall's
   upper east lane is 55.5 m from gp0 (out of range); the corridor's north end (the one spot
   that would see gp0 over the barrel row) is circular — the corridor is entered only through
   the annex vent FROM this mezzanine, or the secret door from the yard (the route's later leg).
   The Q-Claw's own classes are all ported and pinned [M Player_SetGrapplePoint 0x195da0 →
   ChangeSubState(2); Player_SSGrapple 0x192860 the reel 0.05/tick to 1.5, the phase-2 hang
   drift committing the stick's wish with the rope block pinning y — LAB_00192a24].
   **Live check (named): how the game enters the warehouse interior** — the one link that would
   put Bond under gp0/gp1 with the claw; gdbwatch `bond.pos` through a fresh-boot play from the
   display ground (carried from level-0c-remaining-teleports-2's check 1, now with the claw
   candidate noted).
3. **The east climb** (walkreal_0700000c_live.rs, the stand-in at [95.0, 11.3, −209.5]) — KEPT.
   The collision-island pins stand (sisters 5a/5b/5c: the chasm gap rays, the north-strip break,
   the crates4 air gap; no lift kind on 0c [D kind census]; the 8 SSs/2 doors/2 ScriptPlayers
   west-side [D audit §6]; the nav patrols G3/G4 unlinked [D the 0c nav dump]). THIS session
   added the grapple range check: gp3 is ~44 m from the crate-field claw spots (76, 1.2, −210)
   — inside the 50.0 range — but the reel hangs Bond AT the point (y ~22) and the ~49 m
   horizontal drift east to the deck with the rope block pinning y [M 0x192860 tail] is no game
   mechanism (the drift is the walk-speed stick wish [M Player_Move 0x191dd0:111-141]) — and the
   release drop would be ~11.7 m; gp4 hangs the same ~49 m short. **Live check (named): any
   dynamic that lifts Bond onto the band deck** — gdbwatch `bond.pos` at the gate ground spot
   (carried from the previous report's check 2).

No port gap was found to fix: every closure the scans found is the disc's own collision or
range data, not a port defect — the three stand-ins stay with their reasons, sisters and named
live checks, exactly the task's "keep the rest" path. Nothing was weakened: all three sites
gained evidence this session.

## How it was verified

- The task's four binaries, fresh runs on the final tree: `walkreal_0700000c_live` **2 passed /
  0 failed**, `walkreal_0700000c_eastroute_live` **1 passed / 0 failed**,
  `interactables_0700000c_live` **5 passed / 0 failed**, `nf_game` **594 passed / 0 failed /
  5 ignored**.
- The full suite (tools/test/chunk_par_tmp.py — this sandbox kills true background jobs, so the
  chunked foreground 2-worker runner with a fresh state file; priorities: the 0700000c family +
  nf_game): **367/996 binaries recorded by the 01:26 UTC stop line = 1704 passed / 1 failed /
  103 ignored**. The one failure is `probe_pylon_render` (rc 101): the wgpu adapter-enumeration
  panic — this sandbox has no GPU (the same environment limit previous tasks reported; the
  branch touches only test documentation). The 0700000c family inside the suite: walkreal 2/0/0,
  eastroute 1/0/0, interactables 5/0/0, interact 9/0/0, walk 4/0/0, barrelrow 1/0/0, census
  0/0/2 (ignored-by-design), nf_game 594/0/5 — all green.

## What's left

- The suite tail (~630 binaries) — the sandbox clock, not a test result; zero real failures in
  the recorded set.
- The three named live checks above (main agent, PCSX2 fresh boot).

## Usage

not visible to me
