# level-06-agent-guardroom-descent — report

Branch `s/level-06-agent-guardroom-descent`, cut from origin/master 554952ac (the difficulty
branch s/real-health-agent-difficulty-01-07 was already an ancestor — no merge needed). Its
report (docs/reports/real-health-agent-difficulty-01-07.md, read from 737515ff) framed the
failure: the 06 Agent sister wedges on the "guardroom's upper threshold", (28.55, ~41.5,
−19.16), floor_below 40.0, grounded=false, the door #59 legs at the 6000-tick cap.

## What's done

**The sister restored.** Master's 2879088f had reverted the 01/06 Agent walks (red on the
batch). 06's parametrized body was restored byte-for-byte from the difficulty branch's
e356a0ca version of crates/nf-view/tests/walkreal_07000006_live.rs: the walk body at
`difficulty` (the Operative call sites unchanged), the Agent load via the front-end's launch
path `load_with_carry_difficulty(..., 2)` [M C_SBNFDFCTY 0x216ca8:15 → GameState+0x28] with
the `Drones::difficulty == 2` assert [M GameState+0x28 0x2a3790], the door #59 mouth step,
and `walk_the_castle_interior_to_the_party_real_damage_agent`. The wedge reproduced exactly
(the first run panicked at the 6000-tick cap on "to the doorway mouth", bond (28.57, 41.56,
−19.16), floor_below Some(40.000416)).

**The wedge root-caused — there is NO upper level and NO floating bug.** A scratch coll-scan
probe (tris_near + segment grids; deleted before commit) showed:

- The guardroom is ONE room at floor 40.0: the floor grid over x 26..32, z −24..−16 reads
  40.0 throughout; nothing standable exists at the wedge spot's xz [D tris_near: the
  threshold's static tris are the floor plane and the wall faces].
- The "floating" stuck samples (41.56/41.83 ungrounded) are the walker's own stuck-mitigation
  jump pulses [M walker.rs stuck 3/4/5 arms: `jump: true, forward: 1.0`]; the grounded sample
  reads 41.13 = the 40.0 floor + 1.1 (the game's own stand height [M
  Player_StandAtNewPosition: the floor point + 1.1]). The port's floor/fall path
  (grounded = walkable || band [M Player_FeetOnPoint 0x194188:106 + Player_Collision
  0x194df0:104]) needs no fix — Bond falls normally the moment nothing is beside him.
- The real blocker is a **pilaster on the guardroom's west wall**: its east face is the plane
  x 28.61 over z −19.71..−21.36 (vertical face strips at x 28.61 with the ledge bands y
  40.00..40.22..40.40), its solid top ≈42.2 (the floor grid's 42.2 hits at x 28.5,
  z −20..−21) [D]. The leg target (27.9, 40.0, −21.0) sits inside the wall/pilaster, and the
  direct steer's line grazes the pilaster's north-east corner (28.61, −19.71) — Bond
  corner-catches exactly at corner + body radius 0.55 = (28.55, −19.16), d 1.95, and the
  ladder's sidesteps decay before clearing the 1.65-long pilaster while its E-mitigation
  opens the hatch door #748 (28.19, 41.54, −18.43 [D audit #748]) whose open leaf parks at
  z −18.53..−18.42 — sealing the pocket north. At Agent the min-difficulty spawn drone #3
  fights from (29.9, 41.0, −19.5) — 1.4 from the catch point [D Drone_Create 0x136be0:49's
  gate] — and delivers Bond into the catch; at Operative the same steer happened to slide
  around (the master stayed green).

**The fix (walk tactics, real input, opt-in).** `if difficulty >= 2` adds two legs before the
door #59 approach — (29.7, 40.6, −20.9) "the pilaster's east lane" (clearance 1.09 off the
x 28.61 face, > the 0.55 radius) then (29.4, 40.6, −22.3) "south of the pilaster" (south of
its −21.36 end) — and keeps the Operative `walk_nav_to` arm byte-identical in the else. The
sister then walks the whole route: the balcony drop through plate #664, the courtyard, the
ledge route, the dressing room, level_exit. No port code changed; nothing in
crates/nf-view/tests/common/ changed (a temporary nav-route debug print was added to
walker.rs during the diagnosis and removed — net zero, verified with git diff).

**The second failure — the ch-55 alarm assert was a path/phase coincidence, now pinned
honestly.** After the wedge fix the sister failed "a lit head spots the walked Bond (ch 55)".
Head-cone scan [D searchlight parts + the sweep model [M sweep(), Searchlight_Update
0x1cb900]]: head6 (5.50, 55.67, −50.15; yaw cone ±0.218 around 0, pitch cone 1.004±0.218)
reaches Bond's sphere only where its pitch edge meets body height — the band z ≈ −38..−35 ×
x ≈ 2.4..8.6. The Operative run's alarm measured exactly there ((4.55, 42.18, −36.30) at
t 15528); the Agent descent runs a fight-displaced west lane (x ≈ 0.75 at z −36) — 0.6 u
outside head6's yaw band — and the game's own deterministic sweep misses it too (the cones
are identical code at both difficulties; the Operative catch verifies the port's model).
The alarm is NOT a fail channel on this level [L record 1; M 0x1827d8:98-107], and dodging
the beams is the player's own best line at Agent real health (no ch-55 → no reinforcement
spawn #735). The ch-55 pin moved to Operative-only (`if difficulty == 1`); the sister pins
the lit heads (state != 3 && on) and the route. This is a TEST correction against measured
evidence, not a weakening: the event it pinned was never the game's law.

**The damage log against the per-difficulty pipeline — no port divergence.** Operative:
5 hits for 13.2 (end health 86.8 + 50 armour). Agent: 21 hits for 53.6 (end health 46.4).
Agent hits are ≥ the Operative hits for the same drones (drone #3: 1.65/hit at Operative vs
3.83/hit at Agent, closer range; Plr_DMod_Normal 0.7 vs Easy 0.6 [M Player_HandlePain
0x1902c8; tuning.rs 05-08 group], the guards' accuracy 1.1 vs 0.8 [M DroneWeap_DoBulletAccuracy
0x179740:74-88]); the extra damage comes from the Agent-only spawns (drone #23, weapon 0x24,
one hit 15.1 from d 27.5; drone #24, ten hits 15.4). Every hit is the game's own pipeline.

## Verification (what was run)

- walkreal_07000006_live: ALL 6 tests green (36 s) — both Operative masters, the mouth and
  gallery tests, and `walk_the_castle_interior_to_the_party_real_damage_agent` (the whole
  castle in 15216 ticks, 46.4 health, level_exit fired, falls []).
- The EVERY-walkreal sweep: all 26 level-walk binaries green, **137 tests passed, 0 failed**
  (01: 6, 02: 2, 03: 5, 04: 8, 05: 12, 06: 6, 07: 3, 08: 4, 09: 6, 0a: 10+3 ign, 0b: 5,
  0c: 2+1 ign, 0c_barrelrow: 1, 0c_eastroute: 1, 0d: 1+1 ign, 11: 1+1 ign, 11_route: 1,
  12: 2, 13: 2, 14: 8, 15: 5, 16: 14, 1b: 17, 41: 11, 4a: 2, 4a_satchel: 2).
- The 06-adjacent probes: fix_06_alarm_live 2 ✓, fix_06_driver_live 2 ✓, fix_06_fire_live
  3 ✓, fusebox_06_live 4 ✓, difficulty_wiring_live (nf-view) 2 ✓, difficulty_spawn_gate_live
  (nf-game) 1 ✓.
- nf-game: all binaries **1041 passed, 0 failed** (the lib alone 655/0 + 5 ign).
- nf-formats + nf-audio: **125 passed, 0 failed** except the 3 disc_golden tests — the
  allowed slim-ISO exceptions.
- The suite.py build step verified **1095 test binaries compile with 0 errors** on the final
  tree. NOT run: the ~900 remaining nf-view probe/diagnostic binaries (hours of wall time;
  this sandbox kills background jobs between tool calls, so the fast runner cannot outlive a
  call — the sweeps above ran in the foreground, one binary group per call). My branch
  touches only walkreal_07000006_live.rs and docs, so the blast radius is the 06 binary plus
  the shared-crate suites, all green.

## What's left

Nothing for this task's scope. A natural follow-up: the 00 Agent (difficulty 3) sister for
06 — the load path already supports it and the pilaster legs + the beam dodge apply verbatim;
only the damage budget (Plr_DMod and accuracy at value 3) would need the same log-vs-pipeline
read.

## Live checks for the main agent

1. The pilaster corner-catch (confirms the port's floor path and the catch geometry): with
   the disc, from the fusebox nook (29.9, 40.5, −17.5) hold only forward toward (27.9, 40.0,
   −21.0). Expect Bond pinned at ≈(28.55, −19.16) with his feet ON the 40.0 floor; a
   gdbwatch on the player object +0x30 y during the pin should show ~1.1-amplitude jump arcs
   (the mitigations), never a steady hover above 41.1 — confirming nothing holds him and the
   40.0 floor is read all the way to the pilaster face x 28.61.
2. The head6 catch band (confirms the relaxed assert's geometry): with the four heads lit,
   place Bond at (4.5, floor 41.0, −36.3) for one sweep period, then at (0.75, 41.0, −36.3).
   Expect the alarm channel 55 (the heads' data+0x36 byte) to raise at the first spot and NOT
   at the second (the ~0.6 u yaw-band miss).

## Usage

not visible to me
