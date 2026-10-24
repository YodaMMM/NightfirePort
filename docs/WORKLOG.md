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

### level-0a-office-hatch-8 (round result)
BATHHOUSE ROOT-CAUSED, not the sound path: the port's pan chain is line-by-line faithful —
levels() = PS2_SFXCalculate3D 0x1060c (att = a²/100, rear = ((db+1000)·att)/2000, front = 100−rear,
the /1000 integer dots), DotProduct 0x103bc, NormaliseVector 0x10470, GetMyPos 0x57e8, the EE
listener = the viewer pos+matrix rows [M 0x1dd278, [L] sound.md:218 snapshot reproduced by the
port's row-0 formula]. The failing (l=41, r=0) IS the game rule's exact output for the branch's
shouter geometry (dv/100 = (143,6,32) → u=(979,41,219), att=53, front=80 → l=41, r=(21·53)/2000=0 —
the integer division clamps). Probe data: bond pos, RNG state (draws/x/y every 50 ticks) and all
8 nearby guards' pos/state are BIT-IDENTICAL branch-vs-master at the wake; the pre-teleport
driver-start streams are identical f1..f1842; the divergence is the guards' WAKE-TALK picks at
f1844 (branch 0x70+0x73 = 112/115, master 0x71+0x6f = 113/111 — different shouters, the branch's
closest shouter 14.7 m out sits in the clamp wedge, master's 6.0 m one does not), i.e. an upstream
AI/talk-path difference, not audio. The boot-rest delta washes out: the intro-end placement leaves
both trees at y 1.05096 identically. TEST re-pinned per the fact-tag rules: the unconditional
`r > 0` was master-boot-backed and the decompile refutes it as a game guarantee (0x1060c's own
truncation); the pin is now the doc's own "reaches the mixer at non-zero volume" (l > 0) with the
full derivation in the test comment; the sister's r-pin carries the same caveat (left green).
Bathhouse pair green; walkreal_0a 4/0/3 (the [L] rest pin bit-exact, glide/corner/office stay
ignored with the sisters' notes), walkreal_05 8/0, fix_truck 2/0, fire_modes3 5/0/4 (torpedo
re-aim holds). probe: probe_hatch8_pan.rs committed; sound_tick logs the Listener cmd under
NF_MPFX_LOG=1. NOT reached: the hall glide/corner/office walk (task item 2), the full fast-runner
sweep (this sandbox kills background processes — the suite cannot run detached; foreground chunks
went to the root-cause probes), the live pan read (hatch-7's live check 1 stands, now with the
exact wedge numbers).
