## Your task: `level-14-trench-landing`, 07000014: the drop's ghost landing, the east corridor, the mezzanine route

Master has the game's drone-vs-player push (`Collide_Jointy` 0x1ec180's 0x800 arm). The real-input east route (`walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot`, ignored) now stops at the drop: Bond rests on nothing at y 2.97 over the trench — `capsule_push` walkable with push ~0, no floor triangle within 3 m below, no dynamic triangle in the stall box. Read `docs/reports/level-14-sleeper-push.md` on `origin/s/level-14-sleeper-push`.

1. Dump the triangle owners at the stall pin. Suspects: the flipped-open lid's final pose (its anim ends at a 180-degree flip north of the hinge) or the script's second entity 0x02000751 (obj_type 25, created at key 61) standing in the slot. Find what the port collides with that the game does not (or the reverse) [M] and fix it.
2. The corridor east (x 41 -> 49 at z 62.9), the x ~49 south run to the mouths (z 42.16 — the same mask-4 lids #1/#3, E to open) and the mezzanine. Decide [D + M] whether the grapple points #626/#657 at (45.30 / 40.78, 4.79, 42.86) are the player's route up rather than the stairwell slot.
3. Un-ignore `the_east_duct_route_reaches_the_stairwell_slot` and `walk_the_infiltration_to_the_delta_exit` as they go green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-trench-landing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox3 at 2026-10-24T20:10Z

---
## Report (done, 2026-10-24T21:11Z)



---

# level-14-trench-landing — the round report

# level-14-trench-landing — the drop's ghost landing, the east corridor, the mezzanine route

Branch `s/level-14-trench-landing` (from `origin/master` 074e18b9). Task page: 07000014 — the drop's
ghost landing (Bond rested on nothing at y 2.97 over the trench), the east corridor, the grapple
decision, the un-ignores.

## What's done

**1. The stall tri attributed (`probe14_ghost.rs`, new, assertion-free, `--ignored --nocapture`).**

The probe reproduces the E-open + drop and dumps every dynamic owner per script object. Findings:

- The rest at y 2.974 is **the room floor's south lip face** (static, y 2.40, edge z 62.207)
  grazing the falling capsule's rim at dist **0.5500 == r** — push ~1e-7, `walkable=true`
  (`q.y < b.y`, |n.y| > 0.7), no floor within 3 m below. A mid-air east turn pins Bond there
  (the walker's old shape); the D3 face dump prints the two grazing faces exactly.
- The game's own bit block would ground the same graze [M `Intersect_CylGeom` 0x109da0's bit
  block (disasm 0x10a5e0-0x10a734): dot1 face-normal·up > 0.7, dot2 q below B, dot3
  normalize(q−B)·normalize(A−B) < 0.7 — all pass at the stall; `Player_FeetOnPoint` 0x194188's
  tail sets the grounded bit on it], so the catch is not a port divergence — **the un-catch is
  the player's move** (the JUMP; the test's un-catch branch).
- The probe's second finding IS the [M] divergence and the fix: the lid script 0x06000A3B's
  entities — **0x02000750 (kind 0x31**, the 60-tri slab that roofed the slot closed [D probe
  section A: 60 slot tris before the fix, 0 after**]** and 0x02000751 (kind 0x19** ladder art,
  created at key 61, 60 tris at the lip)** — were solid in the port but are **render-only in the
  game**: `Script_CreateEntity` 0x1d8968's fallthrough (LAB_001d8a68) creates every kind outside
  {0x20, 0x22, 0x29, 0xe1, 0xea, 0xeb, 0xf0, 0xf2, 0xf5} as a bare control object
  (`control_create_object(0,0,0,0)` + `hashtable_set_object_to_entity_gfx` gfx attach +
  `control_unlink_object` 0x1d8d54) and never cel-links it; the movement capsule's object pass
  tests only the visited cels' object chains [M `Collide_Pick` 0x1eb320 LAB_001eb8e8 walks
  `cel+8`; the cel-linkers are `build_LinkToRoom` 0x1c9680 → `control_link_object_to_cel`,
  called by `Break_Create` 0x12c230 / `Ladder_Create` 0x1966c0:22 / `Switch_Create` 0x1adcc0:38 /
  `Destroy_Create` 0x12c318 only]. **Fix: `script_parts`' Entity arm whitelisted to
  {0x20, 0x22, 0x29, 0xe1}** (crates/nf-game/src/drone.rs; the old blacklist posed the
  fallthrough entities' glist shapes solid). The doors/SS parts and the `Obj::Anim` discrete parts
  are untouched.

**2. The east route green: `the_east_duct_route_reaches_the_stairwell_slot` un-ignored and passing.**

The drop now lands Bond on the trench floor (the fall threads the slot with the south press; the
south-rim graze, when it catches, is un-caught by the JUMP — the player's move), the sleeper dies
to the walked gun, the corridor east reaches x 48.67, the x≈49 south run reaches z 44.03 (the
mouths' lane) in 2306 ticks. All asserts cite [D]/[M] as above.

**3. The grapple decision [D + M]: the Q-Claw is the route up, not a stairwell.**

The mouths' lids sit at y 2.372 [D probe14_ghost section C: the kind-0x31 entities at
(45.286/40.776, 2.372, 42.159)] — 2.97 m above the corridor floor (−0.6), with the nav web ending
at (49.1, 0.4, 42.8) [D block 0x05, map 54 node 3 — AI cannot grapple] and no stair geometry on
the corridor. The grapple points #626/#657 (45.30/40.78, 4.79, 42.86) [D task page] hang 2.42 m
directly above THEIR mouths' x — one Q-Claw point per mouth. The "stairwell slot" reading is
closed: the slot at z 62.2..63.64 is the sleeper-ambush DROP into the under-vent corridor, not a
way up.

**4. Restored the two tests the s/batch-1024e merge (074e18b9) dropped from master's tree** (the
same evil merge that lost docs/WORKLOG.md and docs/reports/ — restored from 875b0e18/ab1c1a06):
`the_vent_grill_blocks_the_west_duct_per_the_game` (green) and
`walk_the_infiltration_to_the_delta_exit` (kept ignored — its leg 1a walks the OLD west descent
through the z=58.86 grill, which the east-duct findings sealed; the rewrite onto the now-green
east route plus legs 2+ is the next session's work; the ignore reason carries the full note).

## How it was verified

- Ran on the fixed tree (all green): `cargo test --offline --release -p nf-game -p nf-view --lib
  --test sleeper_push_live --test walkreal_07000001_live -- --test-threads=1` → nf-game lib 547
  passed, nf-view lib 96 passed, sleeper_push_live 1, walkreal_07000001_live 3.
- `cargo test --offline --release -p nf-view --test walkreal_07000014_live -- --test-threads=1`
  → 2 passed (the east route + the vent grill), 1 ignored (the big walk).
- `probe14_ghost --ignored --nocapture` — the dumps quoted above.
- The fast suite could not run to a total line this session: the sandbox kills background
  processes within ~2 minutes (three starts died — `suite.out` stayed 0 bytes) and the foreground
  10-minute tool cap is shorter than the suite's build+run. The touched-crate batch above is the
  session's coverage; the next session should run `tools/test/suite.py` on this tree.

## What's left

1. The Q-Claw climb leg at the mouths (grapple #626/#657, reel, the lid E-open, out onto the
   mezzanine) — then the big walk's leg-1a rewrite onto the east route and its legs 2+.
2. The full `tools/test/suite.py` run on this tree (see above).
3. Live checks for the main agent (below) — the rim catch and the drop landing in the real game.

## Live checks for the main agent

1. FRESH boot 07000014, walk Bond to the room's south lip, walk south off the edge holding
   forward: where does he land — the trench floor (−0.6), the sleeper's back, or does he catch the
   slot's south rim (y ≈ 2.97, `obj+0x30` frozen)? If he catches, does a JUMP free him?
2. Same boot: with the lid CLOSED (no E press), does the game's Bond fall through the slot — i.e.
   is the closed lid non-solid (the render-only reading) or solid?
3. gdbwatch on the script entity 0x02000751 (the kind-0x19 ladder art): any instruction that adds
   it to a cel's object chain (a `control_link_object_to_cel` 0x1346b8 call) would refute the
   render-only reading.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
