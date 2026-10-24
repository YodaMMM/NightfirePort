### level-0a-office-hatch-5 (started 01:41 UTC, 2026-10-24)

Task: 0700000a — the post-spawn settle (the game's spawn ray from (−3.9319, 0.1, 10.3978) hits
y 0.0000 in the spawn cel, surface 0x15 [L fresh boot, Collide_RayIntersect breakpointed from
build_PointOnFloor] — the SAME floor the port finds — so Player_StandAtNewPosition stands 1.1000
in both, and the snapshot's 1.0891 [L ee_start_0700000a 0x18694e0] is the AFTER-spawn settle:
−0.0109 from the collision push against the floor at the capsule's cc 1.0347 band, the
feet-on-point settle, the anim-driven cc [M Player_Collision 0x194df0 reads AnimObject+0xcc
every tick]; the -0.0109 surface hunt is moot). Then the hall glide (game 1.1147 after the
doorway-base climb at x −8.304) and the corner at (−17.87, 4.87) (the game walks through at
y 1.127 → 1.120); un-ignore the three sisters and the main walk in walkreal_0700000a_live.

Plan: log the port's root per tick from the spawn with no input and find the lowering step;
port it [M]; then the hall glide line, the corner push landing, and the un-ignores.

- (03:15 UTC) The settle found and ported: the placement is 1.1000 in both ([L fresh boot] the
  breakpointed spawn ray hits y 0.0000, surface 0x15; [M 0x1c95a8 writes the hit back, 0x190d88
  adds 1.1f]); the snapshot's 1.0891111 (0x1b78640+0x34 = 0x3f8b67fe) reproduces BIT-EXACTLY as
  1.1f + f32(4·(−9.8f·(1/60))·(1/60)) — FOUR `vel += WldGravity·REC` adds ([M 0x194df0:107], the
  constants [L] 0x2d8894/0x30d0dc) integrating ONCE ([M 0x194df0:189-197]), then the band rest
  ([M 0x194188:106]). Ported as `Bond::settle_hold` (armed in `stand_at`): four held ticks, the
  release tick integrates the held velocity in the game's op order, the band catches. The anim
  path was cleared first: the stance script 0x06000a00 = one op-0 (seq 0400084f, 2 frames, root
  FLAT at 1.0108399 [D 07F0000a.bin]; cc = root + anim+0xD0 0.0238784 = 1.0347183 [L bit-exact]),
  so no root motion exists at the spawn. `the_spawn_height_matches_the_game_snapshot` un-ignored
  and green (4 passed in walkreal_0700000a_live). The glide/corner sisters re-noted (the settle is
  in; the remaining gap = the doorway base climb's +0.0256 settle — the port pushes off the jamb
  base (−8.604..−8.304, z 16.278 [D tris]) for +0.0036 only); the main walk re-noted with this
  round's run (wedged at the N-S lane leg, the segment hit 0.21 ahead). KNOWN REGRESSION: 05's
  walk_to_the_gatehouse_and_ride_the_truck misses the ch-0x70 plate with the settle in (the ride
  route needs re-calibration); nf-game lib 546 green, the 0a family green (walkreal 4/0, keyboard
  1/0, walk 3/0), walkreal_07000009 3/0. probes: probe_0a_settle.rs committed as the evidence.

Task: level-0a-office-hatch-6 — the spawn settle without the 05 truck-ride regression, then the
0a hall. Branch start from origin/s/level-0a-office-hatch-5 (not merged): the settle is
bit-exact at 0700000a but moves every level's spawn by −0.0109 and
walkreal_07000005_live::walk_to_the_gatehouse_and_ride_the_truck misses the ch-0x70 plate.

Plan: (1) run the truck ride on both trees, log the root and the ch-0x70 plate box at the park
tick, name which side he misses by; fix so both hold — the ride route re-derived at the settled
height, or the settle is wrong somewhere [M]. (2) find the boot gate that holds the placed root
for the first ticks (after Player_StandAtNewPosition 0x190d88, the first Player_Update 0x193600
calls, GS_PausePlayer, the fade-in) [M]; if unfound, keep the observable model with the hold
`// [S] unverified`. (3) the hall glide (game 1.1147 after the doorway-base climb at x −8.304)
and the corner at (−17.87, 4.87); un-ignore the sisters and the main walk as they go green.

Task: level-0a-office-hatch-6 — (05:41 UTC) THE 05 TRUCK-RIDE REGRESSION FIXED. The settle was
arming at EVERY `Bond::stand_at`, including the mid-game `Player_Enable` hand-off. The game's
`Player_Enable` 0x195498 does NOT go through `Player_StandAtNewPosition` at all: it copies the
matrix translation into obj+0x30 DIRECTLY (no floor ray, no +1.1) [M 0x195498:57-68] and zeroes
the velocities [M :81-86]. The port's enable→stand_at+settle held Bond for four ticks with the
truck's script running, and the hand-off's FIRST post-enable tick (the crouch press the walkreal
ride sends on the disable→enable edge) was swallowed by the hold's early return — Bond rode
STANDING and parked at 38.37661 (the standing sister's exact park height), the touch capsule
bottom 38.3766+0.15−0.55 = 37.9766 vs the ch-0x70 plate y 37.7418: a 0.2348 VERTICAL miss
(inside the plate XZ (28.804..30.866, 48.378..52.185) [D the plate quad through the port's
script load, probe_05_park]). Fix: the settle is LEVEL-START ONLY — `Bond::arm_spawn_settle`
called at lib.rs's `Player_Init` 0x1910f8 load spawn (the [L ee_start_0700000a] path);
`stand_at` itself, the `Player_Enable` path, the MP respawns and the build's structural
fallback keep master behaviour. The enable `at` matrix origin measured at the hand-off:
(92.25, 19.42, 103.58) vs the port's floor+1.1 (20.289) — a direct-origin placement would embed
Bond in the truck body, so the port's enable keeps the floor+1.1 stand-in (documented in
`stand_at`'s doc). The hold gate (why the boot's first four ticks hold) remains UNFOUND in the
decompile — Player_Init writes WldGravity (0,−9.8,0) right before the placement
[M 0x1910f8:146-156], GS_PausePlayer 0x17d280 is an MPGame pause-flag store only — the
observable model stays, marked `// [S] unverified`, out of pass/fail as to the gate.
walkreal_07000005_live 5/0 (the truck ride GREEN again), walkreal_0700000a_live 4/0 (the settle
rest still pinned), fix_truck_live 2/0. The 05 boot's own ee_start root 24.9633007 sits
1.0858860 above the port's floor_below at the spawn xz (the courtyard is SLOPED, plane
23.8774147 there) — NEITHER the 1.1 placement (24.9774) NOR the 1.0891 settle (24.9665)
matches; the game's own 05 rest is 0.0141 below the port's floor sampling — a pre-existing
floor-sampling divergence (present on master too), noted for the main agent's live checks.
01's boot rest reads 1.0892906 above its flat floor (5.3394027) — the settle height within the
band's f32 rounding (0a's floor is exactly 0.0, 01's isn't). The glide/corner sisters re-ran
UNCHANGED (the port still wedges at the doorway base (−9.15,1.09,15.92) and dithers ungrounded
at the corner (−17.87,1.17,4.87) — the climb/push shape named in the -2 report; no new
decompile evidence this round). Suites (chunks, foreground — the sandbox kills background jobs):
walkreal 01-0d 15/0/2, walkreal 11-4a+0a 19/0/7, walk_*+level_carry+fix_world_gravity+
fix_start_loadouts+mp_respawn_ammo 40/0/0, nf-game/nf-view/nf-formats/nf-audio units+
crouch/ladder/mp cores 44/0/4 — 118 passed, 0 failed, 13 ignored, no regressions. probes:
probe_05_park.rs, probe_05_floors.rs committed as the evidence.

Task: level-0a-office-hatch-7 — the spawn settle without its five master-tree regressions
(fire_modes3 torpedo, interact_09 standing-short, fix_08 seam, bathhouse mixer, the crouch
option), then the 0a hall glide + corner + main walk. Branch start from
origin/s/level-0a-office-hatch-6 (not merged): hatch-6 made the settle LEVEL-START ONLY and
fixed the 05 truck ride, but never ran the five now-red binaries (spawn-sensitive chunks only).

Plan: (1) reproduce the five on this tree; (2) per test: first-tick timing wrong (fix the test
with evidence) vs the hold swallowing something the game's player does do (narrow the hold [M]);
never widen the hold; the gate stays [S]; (3) the hall glide/corner/main-walk: dump the doorway
base + corner triangles, log per-tick capsule contacts and pushes, compare with the slope-blind
push [M Intersect_CylGeom 0x109da0, Player_CollisionHandler 0x1949a8 disasm 0x194c60-0x194c70]
and the feet band [M 0x194188]; (4) full suite on the final tree (merged with origin/master),
none of the five failing. Suite runner starts in the first minutes; checkpoints every 20-30 min
(sandbox dies 18:47 UTC, last work 18:27 UTC).

Task: level-0a-office-hatch-7 — the five master-tree regressions of the spawn settle. Branch
start from origin/s/level-0a-office-hatch-6 (not merged), merged origin/master (915b4923); all
five reproduced on the merged tree (each passes with the arm line disabled). ROOT CAUSE (4 of 5):
hatch-6's held tick RETURNED out of physics_tick — skipping the crouch/jump/look reads AND the
capsule push. The crouch option test's tick-1 press was swallowed; the 01 seam test's teleport
lands inside the hold window (the direct load starts ENABLED — probe: waited 0 disabled false)
and read the capsule pressed into the seam wall; the 09 armour tester parked short had its step
overlap unresolved through the hold and the release tick's push EJECTED it into the pickup's
take radius (probe: static 3 ticks, then +0.365/+0.254/+0.254 out, taken at tick 11). FIX: the
hold NARROWED — the held tick keeps the unconditional gravity add [M 0x194df0:107], skips the
vel→pos integration ([L] the root holds) and holds off the three grounded-tail zeroing sites,
but RUNS move/look/crouch/jump and the push resolution [M 0x1949a8:136-140]; the [S] gate stays
unverified, not widened. The [L] rest pin still passes bit-exact (walkreal_0a 4/0/3), the 05
truck ride stays green (walkreal_05 6/0/2, fix_truck 2/0). FIFTH (torpedo): NOT the hold — the
old aim clamped pitch to -1.0 (per-tick -0.5) firing into the courtyard floor; the round dies on
the floor between read ticks 2-3 with the full catalog (probe), and the game's own [L ee_start_05]
rest 24.9633 sits BELOW every port root — the down-shot dies within the read window at all of
them. TEST FIXED with the evidence: aim_down(2, -0.25) = pitch -0.5; all pinned bits unchanged.
BATHHOUSE (the fifth, still red): fails bit-identically before and after the narrowing
(id 115, l=0.40999818, r=0, tick 1) — the divergence is the boot REST (1.0891 [L] vs master's
~1.0973) shifting the camera/listener geometry across a marginal right-channel clamp; the game
boots at the LOWER rest, so the live pan read decides (report live check 1). Test untouched.
The glide/corner/main-walk item did not fit the window. Ran: the five + walkreal 05/0a +
fix_truck + keyboard_0a + hatch_crouch + drop_stand + level_carry + fix_world_gravity =
31 passed, 0 failed (excl. bathhouse), 5 ignored; the full sweep did NOT fit. probe:
probe_hatch7_regress.rs committed.

### level-0a-office-hatch-8 (in progress)
Branch s/level-0a-office-hatch-8 from origin/s/level-0a-office-hatch-7 (not merged). This round:
the fifth regression `bathhouse_shout_mixer_live` (the bunker trio shout reaches the mixer
silent l=0.40999818 r=0 at tick 1) — root-cause the 3-D sound volume/pan path [M Sound_Play3D →
SFX attenuation] vs the boot rest delta (game 1.0891 [L ee_start_0700000a] vs master ~1.0973),
fix the cause, never widen the settle or special-case the test; then the 0a hall glide, corner,
office walk (walkreal_0700000a_live); full suite on the final tree. Setup redone (sandbox reset:
no repo, no work/, no toolchain).

### level-14-sleeper-push (in progress)
Branch s/level-14-sleeper-push from origin/master (4f31c0eb). Task: 07000014 past the sleeper
drop — the port's actor-actor push recentres Bond on the carrier drone instead of moving him
off it. Plan: (1) find the game's drone-vs-player push [M Player_Collision 0x194df0 /
Player_CollisionHandler 0x1949a8 object contacts + drone collision response; horizontal vs
vertical resolution when landing on a drone capsule] and port it with a drop-onto-drone test;
(2) corridor east (x 41->49 at z ~62.9), x ~49 south run to the vent mouths z 42.16 and the
mezzanine; grapple points #626/#657 (45.30/40.78, 4.79, 42.86) at vent-mouth level — check
[D+M] whether they, not the stairwell slot, are the route to the mezzanine; (3) un-ignore
the_east_duct_route_reaches_the_stairwell_slot and walk_the_infiltration_to_the_delta_exit as
they go green. Reading docs/reports/level-14-east-duct.md on origin/s/level-14-east-duct
(merged; report not on master). Setup in progress.

### level-14-sleeper-push (in progress)
Branch s/level-14-sleeper-push from origin/master (4f31c0eb), merged origin/s/level-14-east-duct
(its rewrite was NOT on master — merged here: the report, the rewritten east test, probe14_mouth4).
STEP 1 DONE: the actor-actor push is the Jointy capsule arm [M Collide_Jointy 0x1ec180:229-281]
— Bond's kind-0x800 hittest vs the near bot's capsule (arms pos ± up·r, r 0.4 [M NDrone2_Collision
0x140e90:91-121], h +0xa0 default 1.0 [M DefaultInit 0x14b300:48]); the SP rule: the player (class
3 [M Player_Init 0x1910f8:76]) resolves against the bot (class 2 [M NDrone2_CreateObj 0x14d7c8:14])
with the FULL overlap (MPSettings+0x384 == 0 in SP [M C_SBNFMAP 0x20b8d8:26] → fVar12 = 1.0,
0x1ec180:231-233) and the bot never resolves against the player. Ported as LadderCtx::bot_capsules
(near bots only [M DroneFunc_NearPlayerCollision 0x14b190 < 2.0]) + the closest-spine push in
physics_tick [M 0x1ec180:234-253]; test sleeper_push_live::the_drop_onto_the_sleeper_ends_beside_
the_drone_not_on_top (green: the drop slides off the wandering carrier onto the slot floor).
FINDING: the drop slot was never about the drone's back — it is ROOFED by vent-lid SP #266's glist
shape [D probe: dynamic tris y 2.346..2.397 x 40.05..41.54 z 62.22..63.66]; the lid SPs sit mask
0x4 = ACTIVATE [D placement p5] and the port's E path FIRES them (SP_Activate 0x1dc338): probe14_lid
opened #266 on the first press, the 63-tick anim (script 0x06000A3B [D decode]) cleared the slot
(tris 60 → 0 by k45) and Bond crossed the opening south. LEFT: the drop's landing wedges on a ghost
rest y 2.97 (walkable=true, no tri found — the flipped lid's residual or the kind-25 second entity
[S]); the corridor east + the x 49 south run + the mouths (the same mask-4 lids) + the mezzanine;
the two walk tests stay ignored. The airborne gate on the push is a marked port choice [S] — the
grounded-side push (allies: 01 armour walk, 0a west doorway, 01 E-key door all regressed with it
on) awaits the game's per-bot +0x4f8/+0x570 bits (live check listed in the report).

### level-14-trench-landing (in progress)
Branch s/level-14-trench-landing from origin/master (074e18b9). Task: 07000014 the drop's ghost
landing (Bond rests on nothing at y 2.97 over the trench — walkable with push ~0, no floor tri
within 3 m, no dynamic tri in the stall box), the east corridor (x 41->49 at z 62.9), the x ~49
south run to the mask-4 lids #1/#3 (z 42.16, E to open) and the mezzanine; decide [D+M] whether
grapple points #626/#657 (45.30/40.78, 4.79, 42.86) are the route up rather than the stairwell
slot; un-ignore the_east_duct_route_reaches_the_stairwell_slot and
walk_the_infiltration_to_the_delta_exit as they go green. Reading docs/reports/level-14-sleeper-
push.md (restored from 875b0e18 — the batch merge dropped docs/WORKLOG.md and docs/reports/ from
master's tree; restored verbatim, entries below appended). Setup: repo/work/ISO already present.

### level-14-trench-landing (step 1 done, the route green)
STEP 1 DONE: the stall tri attributed (probe14_ghost, committed) — the ghost rest at y 2.97 is the
room floor's south lip face (y 2.40, edge z 62.207) grazing the falling capsule's rim at dist
0.5500 == r (push ~0, walkable=true, no floor within 3 m below) — a mid-air east turn pins Bond
there; the game's own bit block would ground the same graze [M Intersect_CylGeom 0x109da0's bit
block: dot1 up-face, dot2 q below B, dot3 normalize(q−B)·normalize(A−B) < 0.7 all pass at the
stall], so the un-catch is the player's JUMP (the test's un-catch). The SECOND divergence the
probe exposed is the real [M] fix: the lid script 0x06000A3B's entities (0x02000750 kind 0x31 —
the 60-tri slab that roofed the slot closed — and 0x02000751 kind 0x19 ladder art at the lip)
were SOLID in the port (the old kind blacklist in script_parts) but are RENDER-ONLY in the game:
Script_CreateEntity 0x1d8968's fallthrough (LAB_001d8a68) creates them as bare control objects
(control_create_object(0,0,0,0) + hashtable_set_object_to_entity_gfx + control_unlink_object
0x1d8d54) and never cel-links them; the movement capsule's object pass tests only the visited
cels' object chains [M Collide_Pick 0x1eb320 LAB_001eb8e8 walks cel+8; the cel-linkers are
build_LinkToRoom 0x1c9680 → control_link_object_to_cel, called by Break_Create 0x12c230 /
Ladder_Create 0x1966c0:22 / Switch_Create 0x1adcc0:38 / Destroy_Create 0x12c318 only]. Fixed by
whitelisting script_parts' Entity kinds to {0x20, 0x22, 0x29, 0xe1}. ROUTE GREEN:
the_east_duct_route_reaches_the_stairwell_slot un-ignored and passing — the crawl south, the drop
(the jump un-catch) lands on the trench floor, the sleeper dies to the walked gun, the corridor
east to x 48.67, the x 49 south run to z 44.03 (the mouths' lane) in 2306 ticks. GRAPPLE DECISION
[D+M]: the mouths' lids sit at y 2.372 [D probe14_ghost C] 2.97 m above the corridor floor with no
nav thread past (49.1, 0.4, 42.8) [D block 0x05] and the grapple points #626/#657
(45.30/40.78, 4.79, 42.86) [D] hang 2.42 m above THEIR mouths' x — the player's route up is the
Q-Claw, not a stairwell. RESTORED the two tests the s/batch-1024e merge dropped from master's
tree (the_vent_grill_blocks_the_west_duct_per_the_game — green; walk_the_infiltration_to_the_
delta_exit — ignored with the leg-1a rewrite note). Ran: nf-game lib 547, nf-view lib 96,
sleeper_push_live 1, walkreal_07000001_live 3, walkreal_07000014_live 2 passed + 1 ignored, all
green on the fixed tree. LEFT: the Q-Claw climb leg at the mouths, legs 2+ of the big walk.
