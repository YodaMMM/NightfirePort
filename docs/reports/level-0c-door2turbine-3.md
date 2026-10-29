# level-0c-door2turbine-3 — the report

Task: continue `level-0c-door2turbine-2` — its walkreal is real up to the band east, then RED at
the band floor's void (x 55..65, z −209.5..−206.0 [D its probe]). (1) Find how the game gets from
the band (x 44..53, floor 10.34) to x 67+; fix the port if the disc has a connection the port
lacks [M/D]. (2) Make the walk green through the crossing on real input. (3) Then blockers 2-5 of
level-0c-every-objective in order. Full suite, quote its total line.

## What's done

1. **The crossing mapped by four new committed probes** (`walkreal_probe0c_door3_live` .. `_door6_live`,
   each reproducible from the ISO with `--nocapture`; all numbers below are [D] — read from the
   port's disc-built collision world, which `CollisionWorld::build_scoped` fills from the disc's
   parse with the glist-bit skips the game's own `Collide_Pick 0x1eb320` performs [M world.rs
   build_scoped:630-670, citing parsemap 0x1d1998 and 0x1eb320:140-151]):
   - The band floor 10.34 runs **x 44..55.0** (z −208.5..−206.5) — the -2 report's "x 44..53"
     under-measured it; the fine 0.5 map pins the east edge at x 55.0/55.5.
   - **x 55.5..67 at band height is VOID in the disc's own collision**: the 6.17 pit floor spans
     x 55..63.5 (z −206.5..−208.5), a 7.19 shelf x 64..64.5, then void to the resumed band 10.34
     at x 67.5+. The 10.85 strip (z −206.1..−205.2) spans x 47.5..53.5 and ~60..70, with a
     **6.5-unit gap x 53.5..60** and a 10.17 step (x 60..62.25) at its east lip. The measured
     run-jump carry in this engine is 4.42 units (the -2 session's slot jump, takeoff 40.42 →
     landing 44.84) — short of the 6.5 gap even with the 0.68 drop's extra air time.
   - **Nothing bridges it**: probe 3 dumps every drone/trigger/door/pickup in the box twice,
     300 ticks apart — no mover moves, and the only object is a TOUCH trigger (kind 0x2, out
     0x3d, mask 8) deep at (68.43, 4.14, −200.94). Probe 5 dumps every SS in the level: the only
     movers are the display-case lid (#138/#179, ch 0x12) and the ch-0x54 warehouse vent
     (#184, (53.77, 5.43, −163.59) → y 0.0) — **no SS path touches the crossing box**; no wire
     serves it (the level's 4 wires: three type-0 ring wires plus one type-2 zipline at
     y 17.85, x 33.05..60.53, z −194.77 — whose A and B ends have NO floor below to y 1
     [D probe 6]). No ladder exists in the east half [D probe 6]. The 15.73 plane (z −206.0,
     x 42..76 — the turbine-hall upper catwalk) has no found access: no ladder, no stair found,
     and the x 54.5 column's north rays show a solid y 11..15 wall at z −206.33 [D probe 6].
   - **The game's own nav graph is self-contradictory here**: path17 links band-height nodes
     ACROSS the void — n0 (63.06, 10.74, −207.54) ↔ n1 (49.69, 10.74, −207.29) [D probe 4, node
     y = disc y + 0.4, so 10.74 = the 10.34 band] — while path15 links the PIT floor nodes
     (42.97, 6.57, −207.22) ↔ (62.18, 6.57, −207.35); neither path connects to the east band's
     paths (14/19/21), and the per-path A* (`Route::to_position`, the port of
     AINetwork_CalcRoute) returns NO ROUTE for band→resume in every direction [D probe 4]. The
     AI never crosses west→east at any height in the disc's own graph.
2. **Conclusion (item 1)**: the port's world lacks NOTHING the disc's collision has — the void is
   the disc's own [D]. No port collision fix is justified by the data. How the PLAYER crosses is
   NOT pinned by the port's data (the strongest candidates: a run-jump chain the static map does
   not obviously allow — 53.5→60 is 6.5 units against a measured 4.42 carry — or a mechanism
   outside the port's parsed objects, e.g. the deep TOUCH trigger 0x3d's consumer). No
   decompile-backed behaviour was found to port; **no code was changed in nf-game/nf-formats/
   nf-audio/nf-view this task** — only tests and docs.
3. **The walkreal (item 2)**: `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff`
   is GREEN: **2 passed / 0 failed** (0c legs 4308 ticks + 0d 838 ticks; re-verified after the
   origin/master merge, 9.7 s). Changes:
   - The band-east real walk now extends to the band's TRUE end (waypoint x 54.5, r 0.8) —
     the -2 session's x 58/66 waypoints were past the disc's own edge.
   - The crossing itself is **STAND-IN 3 of 3** (the west-gap and south-gap stand-ins remain 1
     and 2): a sister assert pins the band-height void (`segment_hit (59, 12 → 8, −207.5)` must
     stay empty — the 6.17 pit must never gain band-height floor), then Bond is placed on the
     resumed band ([68, 11.4, −207.9]) and the landing is asserted (grounded, floor 10.2..10.6).
     The stand-in carries the full blocker evidence inline (the probe numbers above). All later
     legs (the resumed band, the gate E-press → `level_exit 0x0700000d` [M Player_Activate
     0x1a5170 case 0x1d → Trigger_Activate 0x1b2318:111], the warm RamSave carry, the 0d spawn
     TouchOnce #436 firing ch 0x23 → objective 3 complete) stay REAL and green.
4. **Blockers 2-5 of level-0c-every-objective (item 3): not started.** The session's window went
   to the crossing (items 1-2: four probes + the walkreal rework + the full-suite chunking, below).
   They remain, in order: 2 = plate #274's display-case cell; 3 = the display → warehouse descent;
   4 = the ring west/south gaps (the south one likely the #283 wire ride [S unverified]); 5 = the
   doorway-ledge → yard descent.

## The FULL suite

Background jobs do not survive a tool-call boundary in this sandbox (nohup/setsid confirmed dead
within one call, twice), so the fast runner ran in **10 foreground chunks** over the 898 test
binaries (`tools/test/suite.py . target --limit 90 --only <chunk>`; two chunks that exceeded the
10-minute tool cap were resumed from their logs so every binary ran). **Total: 3502 passed,
18 failed, 142 ignored.** Every failure is accounted:

- **The slim-ISO allowance (4)**: `disc_golden` ×3 and `identity_disc::the_real_disc_passes_the_identity_check`
  — the documented slim-ISO failures (rule 5 allows the 3 disc_golden; identity_disc is the same
  cause per the -2 report).
- **The no-GPU sandbox (3)**: `fix_0c_pylon_live` ×2 and `probe_pylon_render::probe_render_pylon_only`
  — wgpu `adapter: NotFound` (headless sandbox), drawing-only, documented in the -2 report.
- **The 90 s per-binary cap kills (10)**: `campaign_flow_live`, `fix_npc_doors_live`,
  `frame_cost_live`, `levels_sweep_live`, `net_soak_faults_live`, `rook_door_06_live`,
  `pss_vag_golden`, `walkreal_07000008_live`, `walkreal_07000003_live`, `weapon_matrix_live` —
  slow whole-mission walks/soaks/golden decodes that need more than 90 s on this 2-CPU sandbox.
  Re-verified directly with 280 s: still unfinished (the walkreal_03 binary had not finished its
  FIRST test at the cap) — a sandbox-speed property, not a code failure; none of these binaries
  reads anything this task changed.
- **OOM CRASH −9 (2)**: `mp_stress_explosives_live` and `mp_online_failures` — the 3 GB sandbox
  kills them mid-run, the same failure the -2 report recorded at 4 GB.

`cargo test` crates touched by this task's changes: **nf-view only** (the walkreal + 4 probe
test files). `walkreal_0700000c_live` was run directly three times (before the merge, after the
merge, and once more on the final tree): 2 passed / 0 failed each time. No nf-game / nf-formats /
nf-audio source changed, so their suites are unaffected by this task's diff.

## Live checks for the main agent

1. **The band crossing (the top check)**: in the real game, walk the band east from door2turbine
   (#281) — from the band's true end (x 55.0, z −207.6, floor 10.34) record how the game crosses
   x 55.5..67. Watch specifically: (a) a jump from the band end or the 10.85 strip (x 47.5..53.5)
   toward the 10.17 step (x 60..62.25, z −205.35..−205.85) or the east strip (x ~61..70) —
   record the takeoff spot, the air path and the landing; (b) whether anything spawns/moves when
   the player approaches (a platform the port's parsed objects lack); (c) the deep TOUCH trigger
   kind 0x2 out 0x3d at (68.43, 4.14, −200.94) — gdbwatch on its out-channel write, and what in
   ACTION.ELF reads ch 0x3d (no trigger/door in the level consumes it [D probe 5]).
2. The -2 report's checks (the SE slot route, the #283 wire ride) stand unchanged.

## How it was verified

- The four probe binaries print every [D] number above from the ISO in one run each (`--nocapture`).
- `walkreal_0700000c_live` green 3× as quoted; the stand-in's sister assert fails if the crossing
  ever gains band-height floor (the stand-in then must be dropped for the real route).
- The suite totals are the sum of the ten per-chunk total lines; each chunk's log lists its
  per-binary results (`/tmp/suite_*.log`, sandbox-local, not committed).

## What's left

- **The crossing's live check** (above) — the one unpinned route item; the walkreal's stand-in 3
  is replaced the moment it lands.
- **Blockers 2-5** of level-0c-every-objective, in order (untouched this session).
- The three >280 s binaries (walkreal_07000003/08, weapon_matrix) verified on faster hardware.

## Usage

not visible to me (this environment shows no token, cost or context-window figures).
