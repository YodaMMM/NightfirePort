## aim-object-stage
Started 2026-11-05 ~23:10 UTC. Branch s/aim-object-stage (cut from origin/s/aim-snap-final
23975ae9f). Task: the aim ray still has no object stage, so the game's snap (shutter object
at (139.562, 50.792, -96.706) [L]) and the laser's cut of the 14 wire differ. Add the object
stage with honest glist posing, in the order Collide_Pick 0x1eb320 and Collide_Intersect
0x1ebaa8 walk it. Must end green: interactables_07000014_live, fusebox_geometry_07000014_live,
rocket_glass08_live, rocket_stop_plane08_live, rocket_origin_live, solar_panel_hits_live.
Then run walkreal_07000008, 0c, 14, 16, 1b and report each; the 1b console walk needs firing
spots above the panels. Cite the rule for the rocket's midpoint gate or leave it alone.
Strip the NF_AIMPROBE prints.

Progress log:
- Sandbox reused; branch pushed; both predecessor reports read.
- The object stage built in aim_snap: sp_bullet_targets candidates, the built-sphere broad
  phase (0x133070's pair), the 0x201 posed-glist refine; the 0x94 & 0x20 skip confirmed in
  the decomp (0x1eb320:141-144) and left unmodeled ([S]).
- Measured: the wire entities' collision glist poses ~0.1-0.3 u behind the 14 stat wall
  face and the models sphere poses 10+ u off — the wires ride the gfx-bounds frame end to
  end (the aim candidate and the flight contact); the panels keep the glist path (the
  walkway pin stays green).
- The doors-in-aim-ray experiment (world.rs dynamic walk, two runs) measured out and was
  reverted: the near-door snap breaks the 14 cut and the calibrated 08 lines. The 08
  shutter snap stays the tip's stat answer pending the obj+0xF0 live check.
- NF_AIMPROBE stripped everywhere (grep 0).
- Final: rocket_glass08 3/0, rocket_stop_plane08 4/0, rocket_origin 1/0,
  solar_panel_hits 1/0 GREEN; fusebox_geometry 0/1 and interactables 9/1 red (the measured
  chain: the near door leaf stops the laser round — the obj+0xF0 proxy + the glist pivot
  are the live checks). Walkreals quoted in the report: 08 5/1, 0c 2/1/1 (the real_damage
  death — my stage vs the tip's unrun sweep, one checkout A/Bs it), 14 6/3/1, 16 13/1/2,
  1b 15/2/2 (console 1 unchanged).
- Done 2026-11-06 ~00:15 UTC.


## beam-weapon-panels-1b
Started 2026-11-05 16:46 UTC. Branch s/beam-weapon-panels-1b (cut from origin/s/panel-mesh-wip 786875a4f;
origin/master fd8ee385b already an ancestor). Task: the start weapon (slot 51, the Beam) breaks panels
49/50 at tick 0 from the 0700001b walkway spot where the real game holds fire 245 frames and breaks
nothing (meter ~95). s/panel-mesh-wip's triangle stage in held_hit_volumes/sp_bullet_targets is right per
[M Collide_Intersect 0x1ebaa8] but leaves the bug and fails walkreal_0700001b at "console 1 latched".
Read the real fire path (bullet? press vs meter? damage/tick? class bits? obj+0xD0 contact?), compare with
the port's instant 20-damage round, fix the console-vs-panel difference, keep solar_panel_hits_live green,
run walkreal_0700001b_live + solar_panels_live. Lead: 0x33 Phoenix bolt 0x100000/0x200000 arms explode at
hit (0x60007c4 Explode_Create); two panels together = blast through the sphere sweep, check whether the
game's blast reaches panel SPs and where the port's first shot lands at tick 0.

## aim-snap-objects
Started 2026-11-05 19:34 UTC. Branch s/aim-snap-objects (cut from origin/s/panel-aim-snap 08ace8edf). Task:
finish the crosshair aim snap — port the OBJECT side of Player_WeaponInitBullet 0x1a4bf0:154-164's
Collide_RayIntersect aiming-point snap (mask 0x20a / 0x208 with +0x6C & 0x800) [L the PCSX2 breakpoints:
the beam round leaves the muzzle y -1.23, rises to -1.09 by x 3.47, clears the panels; rounds crossing a
panel's 4.77 sphere get no hit entry unless they touch its triangles]. Drones::aim_snap covers level+panels
only today; add drones, doors, wires/fuseboxes, breakables, script entities; read Collide_RayIntersect
0x1ec690 + Collide_Pick; the glass mask bit 2 semantics (aim_snap's glass argument may be inverted); check
+0x6C & 0x800 against the records. 13 red on the branch (suite 4386/13): fusebox_geometry_07000014,
interactables_07000014 (the laser cut), rocket_stop_plane08 x2, rocket_glass08 (live-proven — fix the ray),
rocket_origin (update to the snapped point), walkreal_07000014 x2, walkreal_0700000c real_damage,
walkreal_07000016 x2 (check for the same ray gap first), walkreal_0700001b x2 ("console 1 latched" — new
firing spots above the panels; order 3,5,6... stalls at console 8 from (44.77,-8.66,1.68)). Keep
firing_along_the_walkway_breaks_no_panel green; keep the panel-only triangle stage.

## panel-model-glist

Aim snap: finish on the corrected filter. Branch cut from origin/s/aim-snap-placement fc6e21313
(the aim ray's filter reads the placement word — object flags never carry 0x8000, so the old
filter skipped every static surface and the snap never landed). Green on this tip:
firing_along_the_walkway_breaks_no_panel, the ray stopping at x 6.958338 — the game's own point
[L PCSX2 breakpoint at 0x1a4d90: head (16.741, -1.137, 4.62), hit (6.958, -1.137, 4.62),
object 0]. Red on this tip, each to be fixed by matching the game's ray: rocket_glass08_live /
rocket_stop_plane08_live (the rocket must pass the 08 window plane -96.75), the 07000014 laser
cuts (the wire needs to be in the ray — the game's ray hits object glists [L the 0x1c object at
(139.562, 50.792, -96.706), pos (140.719, 49.5, -96.751), r 1.812]), rocket_origin_live against
the true snapped point. Then the walkreal consoles (0c/14/16/1b) from player spots and strip the
NF_AIMPROBE prints.

Progress (panel-model-glist): the flight's object-pass arm shipped — Bullet_CollisionHandler
0x12ce80's obj+0xF0 & 0x60 rule [M disasm 0x12cf58-0x12cf5c; Control_CreateObjEx 0x1343c8:36
writes obj+0xF0 = glist+0x44 | create param] ported as a per-door bullet_pass (the placement
word's 0x60 bits) applied in flush_world_impacts, with the world impact recorded at the
explosion point. rocket_glass08_live 3/0, rocket_stop_plane08_live 4/0, rocket_origin_live 1/0,
fusebox_geometry_07000014_live 1/0 (all whole, quoted); interactables_07000014_live 9/1 (the
east-wing laser cut needs the aim ray's object stage). Report: docs/reports/panel-model-glist.md.

## aim-object-flags
Started 2026-11-06 ~00:15 UTC. Branch s/aim-object-flags = origin/s/aim-object-stage
504ccdf75 + origin/master merged. Task: the 14 security room's near door leaf stops the
laser because Door.bullet_pass reads the placement flags as a stand-in for
obj+0xF0 = glist_record+0x44 | create_param [M Control_CreateObjEx 0x1343c8:36]. Parse
glist+0x44 from the disc; use the true word for the flight's pass rule (andi 0x60 at
0x12cf18) and Collide_PickObj's gate (obj+0xF0 & mask, 0x20 + 0x1020/0x40/0x10 by hittest
flags [M 0x1eae80]). Live anchors: panels f0 0x81, shutter pieces 0xc1, debris 0xa8a0 [L
0700001b audits]. Then fusebox_geometry_07000014_live, interactables_07000014_live, and
every walkreal the earlier reports list red (08, 0c, 14, 16, 1b with firing spots).

Progress log:
- Branch cut + master merged clean; pushed.
- The word sourced: celglist+0x44 = the entity-params payload +4 [M 0x1d1508
  pCurrCelList[0x11] = block+8] = the port's MapObject.flags; glist+8 = the collision
  surface-byte pointer (0 = no collision — the pick's glist requirement). Door.f0 plumbed
  (Door_Create's SetGList path, no create param); bullet_pass = f0 & 0x60. The aim stage
  gained the 0x1eae80 mask gate (0x60 for the aim/bullet hittest 0x20a — both call sites
  build it [M 0x1a4bf0:154, 0x12ef18 t3=522]) + the per-cel walk's f0 & 0x3000 pre-filter
  [M 0x1eb914]. Probe: the 1b whole panel word 0x81 and the ten pieces 0xc1 match the [L]
  audits; every 14 door placement's own word is the bare kind 0xdb (the stand-in read false
  for all of them); the 14 doors resolve the anonymous 'OBJECT_Door' (word 0x0, r 0.4937).
- The 14 blocker re-measured in the port: the near leaf (pl 384, cel 43 = the beam's own
  cel) stops the round on its POSED TRIANGLES at t 0.383; its BUILT SPHERE (centre
  (-89.338,-0.681,31.023), r 0.4937) also catches the beam (sqDist 0.0147 < r² 0.2437) —
  the game's own sphere arm would stop a closed-door shot too; the 05 boot snapshot reads
  f0 0x0 on all five doors [L]. The game-real flow opens the leaf (the walk-in proximity);
  the tests teleport in. Next: the tests walk the real route, or the live check settles
  the door's runtime pose/f0 on 14.
- The mask gate + the cel-linked door stage measured UNLANDABLE alone: either half re-snaps
  the 08 straight shot off the glass pane and the [L] -100 pierce breaks. The proxy's secret
  is out: p.flags & 0x60 reads the kind byte (0xdb & 0x60 = 0x40) — every door passed by
  accident; the true word (the shutters 0x0/0x201) stops the round the game passes, so the
  game's shutter pass rides the node-0x40 flag [M HITDATA+0x51 bit 1, 0x12cea0] — next
  stage. The 14 near leaf measured game-honest: its built sphere catches a closed-door
  beam (sqDist 0.0147 < r^2 0.2437), word 0x0, no door-path f0 writer; the tests teleport
  in with the leaf closed. Report: docs/reports/aim-object-flags.md. Runs: glass08 3/0,
  stop_plane08 4/0, origin 1/0, panel_hits 1/0, fusebox_geometry 0/1, interactables 9/1,
  walkreal_08 5/1 — all equal to the stage tip (the branch is sim-identical, f0 additive).

## aim-node-pass
Started 2026-11-06 ~01:10 UTC. Branch s/aim-node-pass = origin/s/aim-object-flags
af18e350a. Task: (1) verify the 08 shutter nodes' 0x40 flag in the port's shape data,
replace the kind-byte stand-in with Door.f0, let the node rule (HITDATA+0x51 bit 1 [M
0x12cea0-0x12cea8], penetration_roll_node) carry the shutters; (2) land the mask gate +
the cel-linked doors — the new [L]: the 08 rocket's aim ray hits the class-0x1c object at
(139.562, 50.792, -96.706) and the round flies (139.56, -95.472) to (139.568, -100.472)
with that object in its hit chain; (3) rewrite the two 14 fusebox tests to walk the real
route (the leaf open). Must end green: rocket_stop_plane08, rocket_glass08, rocket_origin,
solar_panel_hits, fusebox_geometry_07000014, interactables_07000014; then walkreals 08, 0c,
14, 16, 1b.

Progress log:
- Branch cut + WORKLOG; pushed.
- The node-rule hypothesis FALSIFIED [D]: the 08 shutter leaves' collision bytes are
  {3:52, 4:64, 10:146} (Shutter_1/3) and {16:18} (Shutter_2) — no byte carries 0x40; the
  glist words 0x0/0x201 have no 0x60; the walk's pass sources (the node word, the surface
  byte 0x40/0x80, the glass class, obj+0xF0 & 0x60) are ALL quiet on the shutters, yet [L]
  the round passes with the object in the chain. The stand-in stays; the live check (the
  shutter object's runtime obj+0xF0 + the walked node's +0x51 during a pass shot) goes to
  the report. The walk's loop decoded [M 0x12d800-0x12d810]: a passed node sets state 1
  and continues; the first non-passed node exits with its state-2 verdict.
- The mask gate + the cel-linked doors LANDED in the aim ray; glass08 3/0, stop_plane08
  4/0 green (last session's red was the f0-based bullet_pass, not the snap).
- The fusebox_geometry test walks the player's route: the door stance (the model test's
  dpos+(0,0.6,1.6) pattern), the E press via step_bond's 4th arg opens the leaf (state
  0x8 -> 0x4, standing open at the stance), the walk out to the pinned spot. The cut stays
  red: the beam bends west-up off the wire slab (the posed centre (-89.41, -0.62, 30.89),
  the round crossing at x -89.75) — the snap's posed-glist west shift, the stage report's
  documented pivot question. interactables' rewrite not started (the same blocker).

## aim-node-pass-2
Started 2026-11-06 05:28 UTC. Branch s/aim-node-pass-2 (origin/s/aim-node-pass 3ea60a711 merged with
origin/master — 4 files, clean). Task: finish the 14 cut — rewrite interactables_07000014's cut with the
walk-and-E pattern of the fusebox test, fix the snap per the report's live check, keep the assertions, re-quote
rocket_stop_plane08 / rocket_glass08 / rocket_origin / solar_panel_hits and the walkreal 08 0c 14 16 1b binaries.
No [L] pin loosened.
  Results (2026-11-06 06:57 UTC): the fusebox cut is GREEN — the west-up bend was the test's
  own aim, not the snap: the loop aimed from the EYE (the game's shot ray runs from the HEAD
  [M 0x1a4bf0:152-159], 0.45 lower — the crossing measured y -1.056 vs the slab -0.728..-0.503),
  the aim-mode steering fed the free-aim cursor (saturated [-0.296,+0.4] in 60 ticks, the
  crosshair ~0.228 rad west-up — the measured bend), and the laser's raise script held the view
  pitch through 900 aim-mode ticks. Fix: the head-anchored direct view + hip fire (the 06
  burn_wire pattern); assertions untouched. fusebox_geometry_07000014_live 1/0 (1.76 s). The
  gfx-frame snap (live check 2's fix) is already in aim_snap's wire branch. interactables
  07000014: 9/1 unchanged (the cut rewrite not started — the window went to the diagnosis);
  the fix's three findings are the rewrite's recipe.

## aim-finish
Started 2026-11-06 07:22 UTC. Branch s/aim-finish (from origin/s/aim-node-pass-2 cb62c58e; origin/master c2c4a6e already contained — the merge is a no-op). Task: finish the 14 cut re-quote — rewrite interactables_07000014_live's cut loop the fusebox_geometry way (aim from bond.head(), set yaw/pitch directly, hip-fire plain input; keep every assertion), re-run the aim-stack binaries on the merged tree (fusebox_geometry 14, interactables 14, rocket_stop_plane08, rocket_glass08, rocket_origin, solar_panel_hits, walkreal 08/0c/14/16/1b), and check the two aim filters read inst.placement.flags (not the object flags) with a measured filtered-instance count per level.
- 07:30-08:15: the interactables_07000014 cut rewritten the fusebox_geometry way (the head-anchored direct view, the asin pitch — the old helper's atan(dy/len) under-angled the steep stance shot, the hip fire, the equip waited to weapon==0x4e): 10/0 on the merged tree. The re-quote: the six aim binaries green (fusebox 1/0, interactables 10/0, stop_plane 4/0, glass 3/0, origin 1/0, solar 1/0); the five walkreals against master baselines measured same-session — the aim stack's src REGRESSES 8 tests across 5 levels (08 5/1 vs 6/0, 0c 2/1 vs 3/0, 14 6/3/1 vs 9/0/1, 16 13/1 vs 14/0, 1b 15/2 vs 17/0). The 1b evidence: consoles 1/2 never latch (master: all 8) — the aim snap (absent on master) redirects the console shots; the snap's object stage excludes the script entities per the live-proven 08 lines, so a shot AT an entity snaps past it. The filter check: filter A reads inst.placement.flags (verified + history), filter B reads the object's glist word ([M] obj+0xF0 = glist+0x44 — the game's own gate); per-level counts printed (aim_filter_census). Lesson 9 found live: the four #[cfg(test)] CollisionWorld literals lacked glist_flags — the lib suite did not compile on the aim stack; fixed, lib 24/0 + formats 92/0 + audio 683/0.

## aim-candidates
Started 2026-11-06 08:28 UTC. Branch s/aim-candidates (from origin/s/aim-finish 81616ca, merged origin/master c2c4a6e — clean). Task: the aim snap's object stage stops on objects the game does not have — fix the pre-filter constants (lui s1,0x3000 = 0x30000000, the f0 word glist+0x44|old&0x30008000), add the Collide_PickObj gates (class 5 skip on 0x200, obj+0xfa&8 or glist+8!=0), make the candidate set the per-cel chains + ForcedList, find why sp_bullet_targets returns stream entities, and re-check Door.bullet_pass against the f0 gate under the [L] 08 pins. Quote the six aim binaries + walkreal 08/0c/14/16/1b.
- 09:00-09:45: the hittest settled from the disasm [M ACTION.ELF]: RayIntersect is called with
  the FLAGS word 0x20a/0x208 in the o7 slot (0x1a4d5c-0x1a4d88: addiu t3,0x20a; movn t3,0x208 on
  weapon+0x6c & 0x800; sw $zero,($sp) = the refine arg 0) and the type word +0x94 = 0x201
  [M 0x1ec690's store] — so the PickObj mask is 0x60 (the port's value was right), the flags'
  0x200 class-5 skip is real but vacuous for the port's candidate set, and the obj+0xfa&8 /
  glist+8!=0 gate is active (script entities pass via Script_EntityStart's 0xe4 [M 0x1d8d78:109];
  the shape() requirement is the glist+8 half [M 0x1d1810:23-40]). FIXED: both f0 pre-filter
  sites 0x3000 → 0x30000000 [M Collide_Pick disasm 0x1eb910: lui s1,0x3000]. The chain-membership
  gate (find_cel on the candidate sphere) built and measured a NO-OP on 1b (every candidate
  in-cel — the probe) and was NOT shipped (unverified against the 14 wire legs). Reproduced 1b
  pre-fix (15/2, console 1 unlatched at line 644); post-fix battery: the six aim binaries GREEN
  (fusebox 1/0, interactables_14 10/0, glass 3/0, origin 1/0, stop_plane 4/0, solar 1/0), lib
  683/0, walkreals 08 5/1 / 0c 2/1 / 14 6/3/1 / 16 13/1 / 1b 15/2 — the aim-finish profile
  unchanged: the constants are safe but the 8-walk regression lives in the candidate set
  (step 3), whose discriminator (creation state vs pose vs a kept-apart list) is still open —
  the report carries the evidence and the live checks.

## aim-console-phase
Started 2026-11-06 10:44 UTC. Branch s/aim-console-phase (from origin/s/aim-candidates ceb1ddae8, merged origin/master d6245281a — clean). Task: the 10-aim-candidates premise "the port stops on objects the game does not have" is WITHDRAWN (the PCSX2 rays ran with Bond idle, pre-launch-order, rigs down). Now measured: the port's rig pose matches the game (t 0 (24.44,-8.80,-11.39) both; game frame 1782 (26.25,-3.02,-11.39) vs port t 1620 (26.28,-3.04)); idle only one rig rises; the rest come with the console phase (channel 19, raised by the first wave's death); shoot_consoles starts frame 2746; writing channel 19 alone in PCSX2 raised nothing. No evidence the game's ray skips raised rigs — do NOT remove candidates to make walks pass. Plan: (1) log what the snap stops on at each failing shot of consoles 1/2 (entity, launch-script role, the console's own hit sphere behind it — is a raised rig really in the line?); (2) if so, the snap may be right and the walk's stance wrong — level_station plans from sphere_shot_ok which does not see these entities; make it use the same candidates as the snap, try on 1b; (3) the other six reds (08, 0c, 14 x3, 16) one at a time — aim-stack route levers vs raw-aim tuning, say which is which; (4) for PCSX2 later: what the first wave's death does besides channel 19, in order, with citations. Keep the [L] 08 pins and the 0x30000000 fix. Quote the six aim binaries and walkreal 08/0c/14/16/1b.

## aim-console-phase
Window 2 (2026-11-06 10:44-12:00 UTC). Branch s/aim-console-phase (from origin/s/aim-candidates ceb1ddae8, merged origin/master d6245281a — clean; first entry 10:44).
- 10:50-11:10: the traced snap — aim_snap's body moved into aim_snap_inner with an optional trace sink, aim_snap unchanged, new aim_snap_traced returns (point, Vec<SnapHit>) with one push per winning stop (level/object/wire/door/drone; SnapHit{stage,id,pi,t} in drone.rs). One implementation, zero behavior change — proven by the battery below.
- 11:00-11:25: the 1b probe (walkreal_0700001b_live.rs: the [cands] dump, the [ring] per-bearing scan, the [gate] traces, the [snap] per-shot winners). FINDINGS: (1) console 8's thief is NOT the snap — 120/120 ring bearings snap-clean at t2207 and t4000; the rig pieces (pi30 gate191 script 6000928, hashes 20007e6 r1.80 / 20007f6 r2.70 at z -11.39, ~2 u behind the screens) are candidates only while the rise script streams them (they appear at t1296 con4's winners, gone by t2207) and never beat the screens' own t 0.044; (2) the real failure: the float WEDGES at (44.7,-8.66,1.68) ~2.2 u short of console 8's station (46,-8.12,0) — the bank steps down along x (screens y -0.60/-2.91/-5.21/-7.51 at x 12.7/25.7/38.7/51.8) and the descent overshot onto the room floor where walk-mode pins it, thrusters dead — from there sphere_shot_ok is FALSE (the world clips the bank) and the aim misses 0.54; (3) the pre-merge master delta (770a74927's alert fold) is the only game-code change on the leg — the 1b test file is byte-identical pre/post merge.
- 11:25-11:30: the snap-aware planner wired anyway (the task's step 2, kept as correct-by-evidence): snap_shot_ok (the traced twin on the stance's crosshair ray, the same candidates the snap sees) gates level_station's ring, the default station, and fire; the stall now measures GOAL progress (a wedged crawl moved 0.3 every ~110 ticks and kept moved_at fresh — the stall never tripped); the replan anchors on the console's default station when the wedged spot's straight lines are all bank-clipped; the float grew climb/creep sag guards. Consoles 8/7/1/2 still red (the landing itself not yet prevented — the crossing-waypoint attempt went pathologically slow and was reverted).
- 11:35-11:46: the battery, every number the aim-candidates profile: fusebox_geometry_07000014_live 1/0, interactables_07000014_live 10/0, rocket_stop_plane08_live 4/0, rocket_glass08_live 3/0, rocket_origin_live 1/0, solar_panel_hits_live 1/0; walkreal 08 5/1, 0c 2/1, 14 6/3/1 (single-threaded — 2 threads OOM-killed), 16 13/1, 1b 15/2. The src change is behavior-preserving; the seven reds (08 0c 14x3 16) are the pre-existing aim-finish ones, untouched.
- LEFT: the six reds' per-level triage (the task's step 3) not started beyond 1b; the first wave's death ledger besides channel 19 (step 4) not compiled; 1b's console 8 landing fix (the zero-g volume boundary vs the low stations) open; the full suite not run this window (the battery above + 1b's 15/2 only).

## 1b-zero-g-floor
Started 2026-11-06 12:02 UTC. Branch s/1b-zero-g-floor (from origin/s/aim-console-phase, already
up to date with origin/master). Task: the float wedge on 1b is the WALK's, not the game's (the
driver's correction: with no input PCSX2 and the port settle identically; the pocket under the
bank never ends zero-g by itself [M Player_ZeroG 0x192ab0 has no ChangeSubState]); fix the
comment, find why the walk's thrust cannot leave the pocket (the bank-face thrust or float_to's
lift input), re-run the 1b walks until consoles 7 and 8 latch, and run the [ring]/[gate] probe
on 08, 0c, 14 x3 and 16 to classify snap faults vs route/raw-aim tuning (14's three are the nav
stick at "the corridor south to the leg-2 spot"). No zero-g or collision code changes. Keep the
[L] 08 pins and the 0x30000000 fix.
1b-zero-g-floor OUTCOME (2026-11-06 ~13:15 UTC): the wedge is the walk's (the driver's correction
confirmed) — the [pocket] ray dump maps the enclosure (ceiling 0.83 up = the bank's sloped
underside, riser 4.73 at +x, −x/±z/down clear) and the port's radial push cancels slope-parallel
thrust (frozen ±0.05 through 3000+ ticks of full climb+back; the game's own push tail is dead
[M Intersect_CylGeom's +0x50 guard] and clips instead). Fixed by the two-phase goal (the report's
own next lever): low-station goals cross at the column top goal.y+5 and descend the station's own
column. The wrong "walk-mode pins it, thrusters dead" comment fixed with the [M] citations (the
lift IS actions 7−8 [M 0x192ab0:136-137]). Consoles 7 AND 8 latch — all eight latch
(8@f5511, 7@f5943); the newly-reachable MissileDeploy gate is the next lever (the deadline math
in the report). The probe on 08/0c/14×3/16: ALL route reds, none snap (08 the station hurt
volume; 0c the d 61-90 sniper drain, killers full hp; 14 the leg-2 spot nav stick — the straight
line clips the wall 0.56 u west of the OPEN door (32,2.1,35.8); 16 the point-blank chaser swarm).
Battery: 1b 15/2/2, 08 5/1, 0c 2/1, 14 6/3/1, 16 13/1/2 (all pre-existing counts; the 1b reds now
run 300+ s deeper to the MissileDeploy assert). Report: docs/reports/1b-zero-g-floor.md.

## aim-route-finish
Started 2026-11-06 13:25 UTC. Branch s/aim-route-finish (from origin/s/1b-zero-g-floor, merged
origin/master, clean). Task: finish the aim routes from the 1b-zero-g-floor report — (1) the 1b
missile deadline (consoles 5/8/1/2 land 30-438 ticks late: pre-charge the clip, lower the burst
gate to 36, the 7->1 flight gets the two-phase treatment), (2) the 14 x3 doorway waypoint
(~x 31.8, z 36.3) before the leg-2 spot, (3) the 08/0c/16 route re-pins per the report's
classification. No collision/zero-g/snap changes; the [L] 08 pins and the 0x30000000 fix stay;
zero-g-slide has not landed, so the pocket-avoidance levers stay.
aim-route-finish OUTCOME (2026-11-06 ~14:50 UTC): the 14 x3 route reds LATCH — the leg-2
approach re-anchored on the probe's west lane (29.7, 0.8, 36.2) at both sites; the report's
doorway waypoint (31.8, 36.3) measured WEDGED (the walker's E toggles vs use_door's 0x8
early-return, the leaf's sliding live sphere keeps the kind-2 door cycling — stuck dump 0x8
pos01 0.77, d 1.55, 6000 ticks); whole walkreal_07000014_live 9/0/1 (was 6/3/1). The 1b missile
deadline MEASURED across three drakes runs: the burst gate kept at 48 (36 regressed 5 by +126
and 8 by +348 — the hold-chain recharge churn); the 7→1 departure-side two-phase kept (console
1 36 ticks early, 2 by 24); the anticipation arm added but measured dead (the console scripts
WAIT at holds — best is Some — so the next console's flight runs inside its own hold window;
the next-window lever: count answered holds (3/script [D]) and pre-fly on the last one).
MissileDeploy still [3,3,2,2,3]. 08/0c/16 untouched (the classification carried). The stream
batch: do NOT join yet (the 1b drakes walks must go green first — one stable timeline).
Suite: nf-game/nf-formats/nf-audio --no-fail-fast 162 binaries, 160 ok, disc_golden 11/3 (the
allowed slim-ISO 3), identity_disc = the onfoot sector-count artifact (volume/serial/NTSC/1.00
match); the door-test 4-tuple compile debt (aim-object-flags' missed files) fixed. The rest of
nf-view not run (test-file-only changes: 14 and 1b; no src/, no common/ helper). Report:
docs/reports/aim-route-finish.md.


## aim-route-3
Started 2026-11-06 ~15:36 UTC. Branch s/aim-route-3 (cut from origin/s/aim-route-finish
85ce433f4, merged origin/master a6665d115). Task: (1) 1b: build the aim-route-finish recipe
— count answered holds per console, pre-fly to the next station on the last hold's answer;
measure the estimated 150-tick lateness of 8; goal MissileDeploy clear and both drakes walks
green. (2) 08 x1, 0c x1, 16 x1: the report's levers; 0c finishers sit at d 61-90 — check the
weapon Bond holds before any stand (pistol range 50 vs rifle 150+ [weapon_data +0x1c]).
Test-side only; the [L] 08 pins and the 0x30000000 fix stay. Finish with the aim binaries and
walkreal 08 / 0c / 14 / 16 / 1b whole.

Progress log:
- Sandbox reset; fresh clone, branch cut, master merged, pushed.

Progress log (continued):
- The recipe built + the deadline instrumentation: `answered[pi]` counts SP_UnPause
  activations (the damage tail's one-tick flag [M 0x1dc2f8-0x1dc318]) = the answered holds;
  the pre-fly to the next un-latched station fires on the 3rd answer; every latch logs
  `deadline = missile_start[k-1] + P0_LEN[k-1] - 1` [M 0x1aa218/0x1aa23c-258; D P0_LEN].
- MEASURED run 1 (the recipe, the 48-burst kept): ch19 = 2747, all 8 missile starts 2747;
  latches 3@4017 (−529) 4@4425 (−121) 5@4629 (+83) 6@5043 (−103) 8@5505 (+359) 7@5961 (−85)
  1@6388 (+342) 2@6531 (+485); MissileDeploy [3,3,2,2,3,2,2,3]. The pre-fly fires on all 8
  consoles and gains ~6 ticks — the flights were already inside the end-plays.
- The chain decomposition (run 1): twin taps 36 t, height re-floats ~78, clip dry gaps 289
  (k5→k6) and 360 (k7 h1→h2), the real flights 336 (6→8) and 353 (7→1); all 8 UI scripts hold
  in parallel from ~S+1009 — the serial flow is the wave cost.
- CHALLENGER 1 (the steady cadence, clip ≥ 16 replaces the 48-burst): MEASURED WORSE
  ([3,3,2,2,3,3,3,3] — k4 −126 but k5 +108, k8 +414, k7 arms): the early chains consumed the
  clip ration the 48-threshold saved for the late ones. REVERTED.
- CHALLENGER 2 (the interleaved fire pick): bit-identical to challenger 1 through k7 — the
  level look pins a tap to the eye's own height band, so the only free targets are the height
  twins, which the serial flow already answers in ~36 ticks. REVERTED. The 48-burst survives.
- Both drakes walks still red at the MissileDeploy gate; the next levers (deadline-rationed
  burst gate, the fight-exit clip top-up, the mid-bank dual stance) are in the report.
- 08 x1 / 0c x1 / 16 x1 NOT started (the 1b item consumed the window) — the report carries
  the classification and the 0c weapon-range hint.
- Suite not run (time): one test file changed (walkreal_0700001b_live.rs), no src/, no
  common/; the drakes walks ran 3× whole.

## aim-route-4
Task: fix the missile countdown (SpaceMissile_Update state 0 loads 0x30d0dc = REC_FRAME_RATE = 1/60 [M 0x1aa1f0-0x1aa1f4], so COUNTDOWN_STEP 512.0 in space_missile.rs is wrong and the seeds are seconds), then re-pace the two drake walks in 1b from the true deadlines (launch + seed x 60 + P0 length) and the console order they imply, then 08 / 0c / 16 with the levers in 1b-zero-g-floor.md section 3. Test-side only; the [L] 08 pins and the 0x30000000 fix stay.

Progress log:
- Branch cut from origin/s/aim-route-3 (f49ded65a); origin/master already an ancestor, merge a no-op. Pushed.

Progress log (aim-route-4 continued):
- The countdown fix shipped (085b88ca7): COUNTDOWN_STEP = REC_FRAME_RATE 1/60 [M 0x1aa1f0-0x1aa208
  disassembled], the p3 seeds are seconds — and the seed map CORRECTED to 105/105/40/20/0/60/105/60
  by index [D placement re-dump; the old list was mis-ordered against p4].
- ch 19 pinned as the AND[193,194] mux's out (the mux writes it every tick — the raw ch-19 poke
  in PCSX2 held one tick, hence "started nothing in 118 s"); the drive tests hold the mux's inputs.
- The drakes walks re-ordered onto the true deadlines [5,4,3,6,8,7,1,2] (k5 +1799 … k1/k2/k7 +9599)
  with a 12000-tick phase cap: MEASURED — all eight latched early (k5 3993 −553 … k2 10803 −1542),
  MissileDeploy all-2 — the gate that killed the last three windows PASSED without the clip levers.
- interactables 9/9, drake_fight 3/3, space_laser 4/4, nf-game space_missile 6/6;
  walkreal_0700001b 15/2/2 — the two drakes walks now red LATER: Drake's fight (run 1) and the
  real-health console starvation on 7/1/2 (run 2). Report: docs/reports/aim-route-4.md.

## aim-route-5

Branch `s/aim-route-5` from `origin/s/aim-route-4` (b991dc45d), merged `origin/master`
(ab80ac6ed — the one-way doors + the escort VIP fail, clean). Scope per the driver: the
console phase is green (aim-route-4 §2); the next reds are Drake's fight and the real-health
clip (§4), then 08/0c/16 with the levers in docs/reports/1b-zero-g-floor.md §3 — for 0c check
what Bond holds first (pistol-class range 50 vs the d 61-90 snipers). Test-side only; the
[L] 08 pins and the 0x30000000 fix stay. Carry-over from the missile-countdown-master window:
on master's RAW aim both drakes walks went green (17/0/2 whole) with the tail [1,2,7], the
tail-only wave defense (the first-wave fight), and the bounded ch-0x09 monitor-read wait —
Drake's fight is green on raw aim, so the aim-side fight red is the snap stack's own shape.
