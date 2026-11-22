# Drone cover

Sources: ACTION.ELF decomp and disassembly (research pass, WORKLOG 0021); placements on the disc. [M] = read in
the code or bytes, [S] = inferred. Two decompiler artefacts: `DroneAnim_CallAnim`'s real argument order is
(anim state, flag, next state, …); `NDrone2_FindCover` stores `CoverNodeOK`'s side (dropped by Ghidra; disassembly
0x152770). Port: `crates/nf-game/src/cover.rs`.

## Cover nodes [M]

Placements with flags word 0xE5 (`Drone_CoverCornerNode`) and 0xE6 (`Drone_CoverLowNode`), created at level load
into `CoverNodes` (0x283E10, 256 × 0xB0). Position = placement translation, yaw = rotation y.

Every map entry of the level archive contributes: the loader dispatches each type-0/1/2/11 entry to
`parsemap_parsemap` (0x1d4880 case 0/1/2/0xb — docs/formats/level-bin.md), whose block walk runs
`parsemap_block_map_data_dynamic` 0x1d1b68 per placement and `parsemap_create_dynamic_objects` 0x1d04f0 case
0xe5/0xe6 — in directory order, so the array's node order is maps in entry order × placements in record order.
On the disc every mission level's shared sub-map 010000f3 adds one corner + one low template pair ("Cover Corner/
Low Node", at the origin, no params — flags 0x1EC, angle 45°, idle 0x11/aim 0xA through the default arms
[M 0x136ff0:44-101, 0x137340:65-87]); 07000004/09/0a/1b/23/25/26/27/28/4a have NO main-map nodes, so the pair is
those levels' whole cover graph (66 sub-map nodes disc-wide, 902 main-map). Port: `cover::build` builds from
the last-parsed (main) map only — the every-map walk of 2026-10-17 was reverted by U-050's answer
(docs/systems/cover-nodes-load.md).

| Field | Corner (type 0) | Low (type 1) |
|---|---|---|
| +1 / +2 | switch channel that must be on / off (p4 / p5; 0 = none) | same |
| +3 / +4 | – | idle / aim anim state |
| +5 / +6 | side-1 depth, side-2 offset × 0.1 (p6 / p7, 0 → 8) | – |
| +8 | A = p3° (0 → 45°) | – |
| +0xC flags | p0: 1 → 8 stand only, 2 → 4 crouch only, else 0xC; p1: 1 → 0x20 step-out, 2 → 0x40 lean, else 0x60; p2: 1 → 0x80 side 2, 2 → 0x100 side 1, else 0x180; 0x10 occupied | p0 = 2 → 1 (+3 / +4 from u16 {0x11, 5, 0xA, 7, 0x1D}[p1 / p2]); else +3 = 0x11 and +4 = 5 (p0 = 1) or 0xA; p3 = 1 → 2 |
| +0x14 | owner | owner |
| +0x40 | emitter: per nav node a path-distance byte (`AINetwork_EmitPath`, 0xFF = unreachable) | same |
| +0x90 | per-target usability bits (8 targets) | same |

07000005: 18 corner nodes, all p = (0, 0, 0, 46, 0, 0, 8, 8) (both stances, both kinds, both sides, A = 46°).

## Usability [M]

`Drone_ProcessCoverNodes` runs every 5 frames (from the first): for each target slot (Bond in single player) it walks
the nodes from a shared round-robin cursor and stops after the first node that cast a line-of-sight ray.
`Drone_IsCoverNodeUsable(t, node)` clears the bits, then:
1. switch gates fail → unusable;
2. thresholds: low f14 = 45°, f13 = 30° (80° / 75° on 0x0700000D); corner f13 = 0.66·A, f14 = A + 5°; when occupied,
   low +5° each, corner 0.77·A and A + 10°;
3. target within 4 → bits 0x80, unusable (no ray);
4. diff = wrap(yaw to target − node yaw): low: |diff| ≤ f14 → 0x48, else 0x200. Corner side 1 (flag 0x100):
   π/2 − f13 < diff < π/2 + f14 → 2 (| 0x10 when diff < π/2); side 2 (flag 0x80): diff > π − f14 or diff < f13 − π →
   4 | 0x20; else 0x200 (no ray);
5. ray from the target's head to node + (0, 1.5, 0) (1.15 on 0x07000011); not visible → bits & ~0x70 | 0x400.

## Choosing cover [M]

- `NDrone2_CoverAvailable(needLos)`: prop 0x30 = 0 → no; already claimed (`+0x4F8 & 0x800`) → yes; opponent visible
  and closer than 4 → no; else `FindCover`.
- `NDrone2_FindCover`: the drone's nav node n (`AINetwork_NodeSearch`, limit 625); over unoccupied nodes on the same
  network with d = distance[n] ≠ 0xFF: the **smallest d** with d² ≤ (distance to the opponent)² and
  `CoverNodeOK(node, 0, needLos)` ≠ 0. Claims it: owner, flag 0x10, side, approach point =
  `GetPosForCornerCover(approach 1)` (goal radius 0.5), in-cover point (approach 0), `+0x4F8 |= 0x1800`.
- `CoverNodeOK(node, side, needLos)`: an opponent; bits & 0xE; with needLos bits & 0x70; distance ≥ 2.0 (and ≤ 1.5 ×
  combat range with prop 0x44); corner: one cover anim possible (crouch lean / crouch step / stand lean / stand step /
  as-low; crouch only when forced) and the side's bit (2 → side 1, 4 → side 2; 0 / 3 picks); low: prop 0xF and bit 8,
  side 3; no other drone within 1.5 of the node unless already claimed.
- Anim predicates (the anim set must have the state): crouch lean = (0x10 or 0xF) and flag 4, (0xA, 0xB or 0xD), flag
  0x40, anim 0x13; crouch step = (0x10 or 0xF) and 4, 0xC, 0x20, anim 0x15; stand lean = (0xA, 0xB or 0xD) and 8,
  0x40, anim 0x17; stand step = 0xC and 8, 0x20, anim 0x1B; forced crouch = prop 0x10 or (corner without 8); as-low
  = low node or crouch-only corner.
- `GetPosForCornerCover(node, side, approach)`: node + x·(cos y, −sin y) + z·(sin y, cos y): side 1 x = 0.1 (approach
  −0.7), z = −depth (−0.8); side 2 x = offset (0.8), z = −0.1 (approach +0.7); low x = 0, z = −0.3 (approach −1.1).
- `SetAngleToCover`: low node or as-low → face the opponent; side 1 → node yaw − π/2; side 2 → node yaw.

## States [M]

Entry: `DroneFunc_CombatState` → 0x8C when `CoverAvailable(1)`, from AimStandFire / AimCrouchFire (after the
reload, out-of-range and no-sight checks, before grenades and combat moves) and from 0x5A / 0x5C / 0x5F (unless
`+0x4F8 & 0x10` or prop 0x40). Timers C/D are cleared on every state change.

| State | Behaviour |
|---|---|
| RunForCover 0x8C | Update: node no longer OK, or opponent visible within 4 → 0x56. `MoveToGoalPosition(+0x704, approach point)` — the goal radius 0.5 that `FindCover`'s `SetupGoalPosition` stored [M disasm 0x169240/0x169250 `lwc1 $f12, 0x704`]: moving → combat-move anim, and `SlideToPosition(0.025, in-cover point, y = 1)` when the route's remaining distance (route+0x88) < 1.0 [M 0x169150:62-67]; arrived (route < route+0x8C = 0.5: `SetupRouteToPosition` stores the caller's radius there [M 0x158b10:104], and both `MoveToGoalPosition` [M 0x1517d8:49-52] and `AINetwork_FollowRoute` past the last node [M 0x15b110:20-22] compare against it) → 0x93; blocked → face, aim or 0x21; else 0x56. Exit (not to 0x93): disown. (The DefaultInit 2.0 at +0x8EC [M 0x14b300:579] is only the seed; the route build overwrites it. The old reading "arrival at < 2.0, slide unreachable" was wrong.) |
| UnderCoverInit 0x93 | `CoverAnim(6)` → 0x94; slide 8 %/tick (xz) to the in-cover point, face the cover angle. |
| UnderCoverIdle 0x94 | `CoverAnim(1)`; refill the clip; timer C in FRand(150) ticks (not with prop 0x10). Update: slide, face; node not OK or > 1.5 from the goal → 0x9C. Ambushers (prop 0x10) wait until the opponent is visible or within 3 → 0x9C. Timer C: Rand(5) = 0 and another cover → 0x8C; node OK with LOS → 0x95; other cover → 0x8C; else retry up to 3 times, then 0x56. |
| UnderCoverAim 0x95 | `CoverAnim(2)` (lean / step-out) → 0x96. |
| UnderCoverFire 0x96 | Fire. Update: node not OK (needLos) → 0x99 (leans) or 0x9C; aimed at and Rand(3) = 0 → 0x99; visible → fire. |
| UnderCoverReturn 0x99 | `CoverAnim(4)` → 0x9A (fail → 0x9B); node not OK or > 1.5 from the goal → leave after the anim. |
| UnderCoverTypeChange 0x9A | leave flag → 0x9C; node not OK and other cover → 0x8C; `CoverAnim(7)` → 0x94. |
| UnderCoverLeave 0x9B | → 0x9C. |
| UnderCoverLeaveNow 0x9C | disown; anim 0xA → 0x56. |
| CrouchCover 0x6E | not node cover: crouch (0x11) 2–3 s → 0x67 (drone type 0xC and crouch modes). |
| AmbushInit 0x37 / AmbushWait 0x38 | drone type 0x10: `FindAmbushCover` (nearest node within 2.0) → 0x93, else crouch and wait until seen / within 3 → 0x56. |

`DroneAnim_CoverAnim(mode, next)`: cover type (0 crouch lean, 1 crouch step, 2 stand lean, 3 stand step, 4 as low)
× column (0 enter, 1 idle, 2 aim, 4 return, 5 change) from `CCModes` (0x270768):

| Type | Columns 0..5 (side 1 / side 2 where they differ) |
|---|---|
| CrouchLean | 10 10 13/14 58 10 1A |
| CrouchStep | 10 10 15/16 58 10 1A |
| StandLean | 1A 1A 17/18 58 1A 1A |
| StandStep | 1A 1A 1B/1C 58 1A 1A |
| AsLowCover | 11 11 0A 58 11 0A |

Mode 6 picks the type: forced crouch → crouch lean / step by predicate, else stand lean / step; none → as-low for a
crouch-only corner, else fail; Rand(n) among the candidates. Mode 7 (type change): Rand(2) ≠ 0 → no change; else
candidates as mode 6 (distance > 8 also allows the crouch types); plays column 5 of the old type. Other modes: that
column. Low nodes: modes 0/6/1/4 play +3, 2 plays +4, 3 plays 0x58.

## Live check [L] (WORKLOG 0023)

work/chase_trace.tsv: the gate guard (61.34, 99.50), Bond 19 units away at (79.86, 22.92, 96.0): Combat (65) →
OutOfRange (66) → RunForCover (67; strafe 0xC, back-off 0x4, strafe 0xD around the gate pillar) → UnderCoverInit (265,
stand-cover loop 171) → UnderCoverIdle (296, clip refilled) → UnderCoverAim (394, lean-out 176 at speed 1.5, then loop
182) → UnderCoverFire (431) at corner node 1 (59.11, 98.69, yaw −90°), position (58.93, 99.45) — 0.28 from the
computed in-cover point (59.21, 99.49) (U-031). The port matches the sequence, node, UnderCoverInit / Aim durations and
the end position (0.2). RunForCover: live 198 ticks (strafe 0xC east of the gate pillar, back-off 0x4 west along its
south face, strafe 0xD north round its corner, UnderCoverInit at (58.60, 99.07)); the port now takes 199 ticks over
the same three legs and arrives at (58.57, 99.02) (`cover_run_rounds_the_gate_pillar_like_the_game`). The old 148-tick
port run stopped 1.5 short south of the pillar: every port goal route was built with a flat 2.0 arrival radius
instead of the goal's own. Fixed for RunForCover only (`handle_cover` passes `goal_arrival` to `move_to_goal_r`).
The game passes the goal radius from EVERY `MoveToGoalPosition` caller (e.g. `ReFindMissionPath` 0x153a30, disasm
0x153ce4/0x153e64 `lwc1 $f12, 0x704`); the port's other callers keep 2.0 (*deviation*, out of this row's scope).
**Open reds (2026-11-21):** the RunForCover change turns 17 scripted-walk tests red, 15 of them green on the old
code (walkreal 01 was not re-run on the old code): escort_follow_07000001 and fix_npc_doors (Bond's walker wedges at (3.4, 10.4) on the way to the 01 corridor
plate and the escort run stalls), walkreal 01 (2: the contact dies), 02 (geisha group trigger), 03, 06, 08, 0d (2),
11, 13 (1 more beside the 2 already red), 15 (2), 16 (3) — almost all real-damage deaths or tick caps. The guards'
own cover runs are the change; whether the reds are only the scripted walks tuned to the old short runs or a port
fault the longer runs expose is not settled. Widening the radius to every caller added two more (15).

## Sniper cover (0x97/0x98) [M] (WORKLOG 0079)

`UnderCoverFire` ENTER (0x16a3e0) branches to `UnderCoverSniperFire` (0x97) when the drone's anim set `+0xDA == 0x13`
(sniper) AND the level is 0x0700000C or 0x0700000D: the test is the unsigned `GameState+0xC + 0xf8fffff4 < 2`
[M 0x16a3e0 case 1]. On every other level (05's three sniper-set guards included) the sniper fires through
`UnderCoverFire` directly. Per level (`nf_view::load` over `dev::LEVELS`): sniper-set drones on 02 (1), 03 (1),
05 (3), 06 (7), 0c (20), 0d (19), none elsewhere.

State 0x97 [M 0x16a610]:
- ENTER: `+0x4F8 |= 0x10000`, AttackTalk, `+0x2E`, timer C at +1 s, timer D at +6 s, then the fire request
  `+0x3B = 0` and the shot flag `+0x40 = 0` (no round before timer C).
- UPDATE: a round fired (`+0x40`, set by `DroneWeap_FireWeapon` 0x178f68:260) with the clip empty → 0x98. Otherwise
  nothing unless the danim class `+0x568 == 2` and `+0x3B` stands. Then `CoverNodeOK(node, type & 3, 0)` (no sight
  need): failed → a corner node with a lean type returns (0x99), any other leaves now (0x9C). Passed, and only with
  the burst done (`+0x3C`): the armed opponent's aim latch `+0x2A` (`DroneWeap_OpponentIsAimingAtMe` 0x17a500; its
  7.5 is the MP-only facing angle, no distance) → 0x99; else `BondIsFacingMe(45)` and `Rand(3) == 0` → 0x99.
- Timer C: `DroneWeap_Fire(1)`. Timer D: 0x99.

State 0x98 [M 0x16a8b0]: ENTER `AlertStatusSet(1)`, `+0x3B = 0`, the reload anim 0x52 back to 0x97, the clip refill
`+0xBBC = +0xBBE`. Ported in `drone.rs::handle_cover`.

## Ambush (0x37/0x38) [M] (WORKLOG 0079)

Drone mode 0x10 (`NDrone2_init_DMODE_Ambush`) sets the start state `+0x5A2 = 0x37` (AmbushInit). `NDrone2_FindAmbushCover`
(0x152958): the unoccupied cover node within **2.0 units** of the drone whose anim predicates pass; claims it the same
way `FindCover` does → 0x93 (UnderCoverInit). Without one: `CallAnim(0, 0x11, 0, 0, …)` (crouch) → 0x38 (AmbushWait),
which waits until the opponent is visible (`+0x228 & 4`) or within 3.0 → 0x56 (Attack). Ported in `drone.rs::find_ambush_cover`
and `handle_cover`.)

## Scary objects (G-026, ported by fix-cover-remainder) [M]

`DroneFunc_HandleExplosives` (0x1435f0, per drone per frame, SP only): while the drone is not scared (`+0x2B == 0`) it
scans the explosive list `Drone_BuildDynamicAwarePoints` (0x138b10) rebuilds every frame from live objects — kind-2
entries: the grenade family (thrower weapon 0x2A/0x2C–0x2F) radius 12.0, the flashbang (0x37, not state 3) 8.0, the
0x34/0x6C and 0x3B–0x40 explosives 12.0, the `*` (0x2A) explosive object 3.0; gas (0x36) makes a kind-1 entry that
never triggers. A guard with fear prop 0x3A/0x3B within the radius that sees the object (ObjectVisibility ≥ 0.7 +
`CanSeeObject(5,0)`) sends itself msg 0x21. `DroneFunc_ConsiderExplosive` (0x1431c0): fear props 0x34/0x3A/0x3B and
the state gate; stores the position (+0x330), the run distance (+0x350: 4.0 grenade weapons with a 2 s deadline,
10.0 flashbang returning to the caller state, 14.0 the big explosives, 5.0 gas/0x2A-object), the return state
(+0x354: 0x87 Hide by default), the deadline (+0x324) and +0x2B, then `RunAwayFromObject` 0x86 [M 0x1431c0:194-195]
— the doc's older "0x86 = RecoverFromScaryObject" reading was wrong. 0x86 ENTER: `FindSafetyFromScaryObject`
(0x1559f8 → 0x152250): the nav-path emitter at dist·1.8, else the straight-line fallback goal = scary +
normalise(drone − scary)·dist, y = scary.y + 0.5; fail → 0x88. UPDATE: past the deadline → 0x88; moving → the run
anim 0x1E; blocked → crouch 0x11; arrived → the return state. `HideFromScaryObject` 0x87: crouch 0x11/0x19/0x21 and
wait the deadline out → 0x88. `RecoverFromScaryObject` 0x88: +0x2B = 0, +0x5A0 != 0x89 → 0x56 (Attack). The port
triggers once at the grenade's bounce (the game rescans every frame while the object exists) with the straight-line
goal (the emitter scan is [S]-approximated by the route build's clamp).

## 07000005 [M]

18 corner nodes (`nf-extract covers`), all reachable through NavPath A's emitter.

Cover prop 0x30 sits in the alert set (parameters 23–25), which governs after `FirstAttack`. 8 SG5 guards have {0xA,
0xC, 0xF, 0x30}; the snipers have 0x40 and no 0x30; two guards have an empty alert set. With default nodes they use
stand lean / stand step (0x17/0x18/0x1B/0x1C, idle 0x1A).

## ValidateCoverAnim (0x13bf40) [M] (ported by fix-cover-remainder)

`DroneAnim_CoverAnim` validates every LOW-node anim through `DroneAnim_ValidateCoverAnim(dcvars, animState)` before
playing it (mode 3's 0x58 excepted [M 0x13c078:297-300]): 0 passes; ≥ 0x77 passes unvalidated [M 0x13bf40:12,22];
1..0x77 is valid only when the drone's anim set (+0xDA) has the state's row, the base set's row also counting
[M 0x13bf40:14-19]; an invalid 5 substitutes 7 when the set has record 11's row, else 0xA [M 0x13bf40:25-38 — the
port approximates the record-11 gate with `can_do(7)`]; any other invalid → 0xFFFF and `CoverAnim` returns 0 — no
anim is queued, so UnderCoverInit/Aim linger and UnderCoverReturn/TypeChange take their fail routes [M
0x13c078:311-314]. The 05 level has no low nodes, so the behaviour is pinned at the function level
(`validate_cover_anim_matches_the_game_table`).

## Unknowns

`+0x2E`, `+0x38`, `+0x4C0`, the meaning of low-node flag 2, UnderCoverFire's clip handling, the
units of `OpponentIsAimingAtMe` / `BondIsFacingMe` (U-031); `+0x4F8 & 0x10000` = the transient flag the state
machine clears on every change (set by RunForCover/RunAway/Hide ENTERs [M 0x171df8:182-183]). The RunForCover
duration gap (live 198 vs port 148 ticks, earlier 114) is closed: the arrival radius, not the route (see Live check).

## Target slots [M] (checked 2026-11-21)

The usability bits are kept per NPCGlobals target slot (0x299110, 8 × 0x20). Only the player ever holds a slot in
single player: `DroneFunc_AllocateTargetID` 0x141f28 returns 0xff unless the drone's team (+0x44) is 1
[M 0x141f28:11-13], and a team-1 drone's opponent is always the player (`NDrone2_FindOpponent` 0x1424e0:101-118,
team != 2 → `glb_players`, or 0 when `PlrStat_OkToUpdate` fails or he is dead). `Drone_InitComms` 0x139520 refreshes
each claimed slot's position from its object (+0x30) [M :127-147], runs `ProcessCoverNodes`, then clears every slot
[M :150-160]; the drones' `SetOpponent` 0x1422d0 re-claims slot 0 later in the frame [M 0x1422d0:74-86]. So slot 0 = Bond at his
live position, as the port models. Port deviations, neither visible in play: (1) with no claimed slot (the first
frame of a level, a mission already won or lost) `IsCoverNodeUsable` clears a node's bits and casts no ray
[M 0x1386d8:60-63], so the round robin walks the whole list; the port always evaluates for Bond. (2) the walk stops
only when `DroneVision_CanSeePosition` 0x178438 really cast a ray (its cel pre-test can refuse first, the ray counter
0x299092 then unchanged [M 0x1386d8 return]); the port stops at every node that reached the ray step. (3) the
occupied-node gate: a node owned by a drone whose own slot (+0x174) is not the slot being evaluated is skipped
[M 0x1386d8:70-91]; every owner is a team-1 guard on slot 0, so it never fires. `iTimeToCalc.255` (0x30caa0) is a
function static that only `ProcessCoverNodes` touches [M: no other function in the decompile names it], so nothing
resets it per level and the game's 5-frame phase carries over between levels; the port starts each level at 0.
